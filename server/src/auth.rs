use crate::core::{AppError, AppResponse, AppResult, AppState};
use crate::utils::base64_field;
use actix_session::Session;
use actix_web::Scope;
use actix_web::{get, post, web};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::postgres::PgDatabaseError;
use srp::groups::G_2048;
use srp::server::SrpServer;
use uuid::Uuid;

#[derive(Deserialize)]
struct SignupRequest {
    username: String,
    #[serde(with = "base64_field")]
    srp_salt: Vec<u8>,
    #[serde(with = "base64_field")]
    srp_verifier: Vec<u8>,
    #[serde(with = "base64_field")]
    private_key_encrypted: Vec<u8>,
    #[serde(with = "base64_field")]
    public_key: Vec<u8>,
}

#[derive(Serialize)]
struct ActionResult {
    success: bool,
}

impl ActionResult {
    fn success() -> Self {
        return ActionResult { success: true };
    }
}

impl<T> From<AppError> for AppResponse<T> {
    fn from(err: AppError) -> Self {
        AppResponse(Err(err))
    }
}

#[post("signup")]
async fn signup(
    session: Session,
    data: web::Data<AppState>,
    request: web::Json<SignupRequest>,
) -> AppResponse<ActionResult> {
    AppResponse::wrap_async(|| async {
        guard_auth(&session, false)?;

        if let Err(e) = sqlx::query!(
            r#"
        INSERT INTO 
            Users(username, srp_salt, srp_verifier, public_key, private_key_encrypted)
        VALUES ($1, $2, $3, $4, $5)
        "#,
            request.username,
            request.srp_salt,
            request.srp_verifier,
            request.public_key,
            request.private_key_encrypted,
        )
        .execute(&data.db_pool)
        .await
        {
            if let Some(db_err) = e.as_database_error() {
                if let Some(pg_err) = db_err.try_downcast_ref::<PgDatabaseError>() {
                    if pg_err.code() == "23505" {
                        return Err(AppError::DuplicateUsername);
                    }
                }
            }
        }

        Ok(ActionResult::success())
    })
    .await
}

#[derive(Deserialize)]
struct SRPHelloRequest {
    username: String,
    #[serde(with = "base64_field")]
    srp_client_public_key: Vec<u8>,
}

#[derive(Serialize)]
struct SRPHelloResponse {
    #[serde(with = "base64_field")]
    srp_salt: Vec<u8>,
    #[serde(with = "base64_field")]
    srp_server_public_key: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct SRPSession {
    userid: Uuid,
    username: String,
    #[serde(with = "base64_field")]
    salt: Vec<u8>,
    #[serde(with = "base64_field")]
    verifier: Vec<u8>,
    #[serde(with = "base64_field")]
    client_public_key: Vec<u8>,
    #[serde(with = "base64_field")]
    server_private_key: Vec<u8>,
    #[serde(with = "base64_field")]
    server_public_key: Vec<u8>,
}

const SRP_SESSION_KEY: &str = "srp";
const USER_SESSION_KEY: &str = "user";

#[post("hello")]
async fn srp_hello(
    session: Session,
    data: web::Data<AppState>,
    request: web::Json<SRPHelloRequest>,
) -> AppResponse<SRPHelloResponse> {
    AppResponse::wrap_async(|| async {
        guard_auth(&session, false)?;

        let data = sqlx::query!(
            r"
            SELECT id, srp_salt, srp_verifier FROM Users WHERE username = $1
        ",
            request.username
        )
        .fetch_one(&data.db_pool)
        .await
        .map_err(|_| AppError::UserNotFound)?;
        let srp = SrpServer::<Sha256>::new(&G_2048);
        let mut b = [0u8; 64];
        rand::rng().fill_bytes(&mut b);
        let b_pub = srp.compute_public_ephemeral(&b, &data.srp_verifier);

        session.clear();
        session
            .insert(
                SRP_SESSION_KEY,
                SRPSession {
                    userid: data.id,
                    username: request.username.clone(),
                    salt: data.srp_salt.clone(),
                    verifier: data.srp_verifier,
                    client_public_key: request.srp_client_public_key.clone(),
                    server_private_key: b.to_vec(),
                    server_public_key: b_pub.clone(),
                },
            )
            .map_err(|e| {
                eprintln!("Fail to set SRPSession {e}");
                AppError::Internal
            })?;

        Ok(SRPHelloResponse {
            srp_salt: data.srp_salt,
            srp_server_public_key: b_pub,
        })
    })
    .await
}

#[derive(Deserialize)]
struct SRPAuthRequest {
    #[serde(with = "base64_field")]
    srp_evidence: Vec<u8>,
}

#[derive(Serialize)]
struct SRPAuthResponse {
    #[serde(with = "base64_field")]
    srp_evidence: Vec<u8>,
}

#[post("auth")]
async fn srp_auth(
    session: Session,
    request: web::Json<SRPAuthRequest>,
) -> AppResponse<SRPAuthResponse> {
    AppResponse::wrap_async(|| async {
        let srp_session = session
            .get::<SRPSession>(SRP_SESSION_KEY)
            .map_err(|_| AppError::SRPNotStarted)?
            .ok_or(AppError::SRPNotStarted)?;

        let srp = SrpServer::<Sha256>::new(&G_2048);
        let verifier = srp
            .process_reply_rfc5054(
                srp_session.username.as_bytes(),
                &srp_session.salt,
                &srp_session.server_private_key,
                &srp_session.verifier,
                &srp_session.client_public_key,
            )
            .unwrap();

        verifier
            .verify_client(request.srp_evidence.as_slice())
            .map_err(|_| AppError::InvalidCredential)?;

        session.clear();
        session
            .insert(USER_SESSION_KEY, srp_session.userid)
            .map_err(|e| {
                eprintln!("Fail saving user session key: {e}");
                AppError::Internal
            })?;

        Ok(SRPAuthResponse {
            srp_evidence: verifier.proof().to_vec(),
        })
    })
    .await
}

#[post("logout")]
async fn logout(session: Session) -> AppResponse<ActionResult> {
    AppResponse::wrap_async(|| async {
        guard_auth(&session, true)?;

        session.clear();
        Ok(ActionResult::success())
    })
    .await
}

#[derive(Serialize)]
struct GetStateResponseUser {
    username: String,
}

#[derive(Serialize)]
struct GetStateResponse {
    user: Option<GetStateResponseUser>,
}

#[get("state")]
async fn get_state(session: Session, data: web::Data<AppState>) -> AppResponse<GetStateResponse> {
    AppResponse::wrap_async(|| async {
        match get_userid(&session) {
            None => Ok(GetStateResponse { user: None }),
            Some(user_id) => {
                let data = sqlx::query!("SELECT username FROM Users WHERE id = $1", user_id)
                    .fetch_one(&data.db_pool)
                    .await
                    .map_err(|_| AppError::Internal)?;
                return Ok(GetStateResponse {
                    user: Some(GetStateResponseUser {
                        username: data.username,
                    }),
                });
            }
        }
    })
    .await
}

pub fn get_userid(session: &Session) -> Option<Uuid> {
    match session.get::<Uuid>(USER_SESSION_KEY) {
        Ok(option) => option,
        Err(_) => None,
    }
}

pub fn guard_auth(session: &Session, logged_in: bool) -> AppResult<()> {
    let has_logged_in = get_userid(session).is_some();
    if logged_in != has_logged_in {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

pub fn scope() -> Scope {
    web::scope("/auth")
        .service(signup)
        .service(srp_hello)
        .service(srp_auth)
        .service(logout)
        .service(get_state)
}
