use crate::core::{AppError, AppResult, AppState};
use crate::utils::base64_field;
use actix_session::Session;
use actix_web::Scope;
use actix_web::{post, web};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::postgres::PgDatabaseError;
use srp::groups::G_2048;
use srp::server::SrpServer;

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

#[post("signup")]
async fn signup(
    data: web::Data<AppState>,
    request: web::Json<SignupRequest>,
) -> AppResult<ActionResult> {
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
                    return AppResult::err(AppError::DuplicateUsername);
                }
            }
        }
    }

    AppResult::ok(ActionResult::success())
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

#[post("hello")]
async fn srp_hello(
    session: Session,
    data: web::Data<AppState>,
    request: web::Json<SRPHelloRequest>,
) -> AppResult<SRPHelloResponse> {
    match sqlx::query!(
        r"
        SELECT srp_salt, srp_verifier FROM Users WHERE username = $1
    ",
        request.username
    )
    .fetch_one(&data.db_pool)
    .await
    {
        Err(_) => AppResult::err(AppError::UserNotFound),
        Ok(data) => {
            let srp = SrpServer::<Sha256>::new(&G_2048);
            let mut b = [0u8; 64];
            rand::rng().fill_bytes(&mut b);
            let b_pub = srp.compute_public_ephemeral(&b, &data.srp_verifier);

            session.clear();
            if let Err(e) = session.insert(
                "srp",
                SRPSession {
                    username: request.username.clone(),
                    salt: data.srp_salt.clone(),
                    verifier: data.srp_verifier,
                    client_public_key: request.srp_client_public_key.clone(),
                    server_private_key: b.to_vec(),
                    server_public_key: b_pub.clone(),
                },
            ) {
                eprintln!("Fail to set SRPSession {e}");
                return AppResult::err(AppError::Internal);
            };

            AppResult::ok(SRPHelloResponse {
                srp_salt: data.srp_salt,
                srp_server_public_key: b_pub,
            })
        }
    }
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
) -> AppResult<SRPAuthResponse> {
    match session.get::<SRPSession>("srp") {
        Err(_) => AppResult::err(AppError::SRPNotStarted),
        Ok(srp_session_option) => match srp_session_option {
            None => AppResult::err(AppError::SRPNotStarted),
            Some(srp_session) => {
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

                if let Err(_) = verifier.verify_client(request.srp_evidence.as_slice()) {
                    return AppResult::err(AppError::InvalidCredential);
                }

                session.clear();

                AppResult::ok(SRPAuthResponse {
                    srp_evidence: verifier.proof().to_vec(),
                })
            }
        },
    }
}

pub fn scope() -> Scope {
    web::scope("/auth")
        .service(signup)
        .service(srp_hello)
        .service(srp_auth)
}
