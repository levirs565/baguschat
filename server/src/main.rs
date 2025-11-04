use std::env;
use std::fmt::Display;

use actix_cors::Cors;
use actix_session::{storage::RedisSessionStore, Session, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::dev::ServiceResponse;
use actix_web::http::header::{self, TryIntoHeaderValue};
use actix_web::http::StatusCode;
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::web::JsonConfig;
use actix_web::ResponseError;
use actix_web::{
    body::BoxBody, http::header::ContentType, post, web, App, HttpRequest, HttpResponse,
    HttpServer, Responder,
};
use base64::prelude::*;
use blake2::Blake2b512;
use num_bigint::BigUint;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::{
    postgres::{PgDatabaseError, PgPoolOptions},
    Pool, Postgres,
};
use srp::client::SrpClient;
use srp::groups::{G_2048, G_8192};
use srp::server::SrpServer;

fn deserialize_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    BASE64_STANDARD.decode(s).map_err(serde::de::Error::custom)
}

fn serialize_base64<S>(v: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let s = BASE64_STANDARD.encode(v);
    serializer.serialize_str(&s)
}

struct AppState {
    db_pool: Pool<Postgres>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum AppError {
    BadRequest { message: String },
    Internal,
    NotFound,
    DuplicateUsername,
    UserNotFound,
    SRPNotStarted,
    InvalidCredential,
}

#[derive(Serialize)]
struct AppErrorResponde {
    error: AppError,
}

impl Display for AppError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match &self {
            AppError::NotFound => actix_web::http::StatusCode::NOT_FOUND,
            _ => actix_web::http::StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let mut res = HttpResponse::new(self.status_code());

        res.headers_mut().insert(
            header::CONTENT_TYPE,
            ContentType::json().try_into_value().unwrap(),
        );

        res.set_body(BoxBody::new(
            serde_json::to_string(&AppErrorResponde {
                error: self.clone(),
            })
            .unwrap(),
        ))
    }
}

pub struct AppResult<T>(pub Result<T, AppError>);

impl<T> AppResult<T> {
    pub fn ok(value: T) -> Self {
        AppResult(Ok(value))
    }

    pub fn err(error: AppError) -> Self {
        AppResult(Err(error))
    }
}

impl<T> Responder for AppResult<T>
where
    T: Serialize,
{
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        match &self.0 {
            Ok(value) => {
                let body = serde_json::to_string(value).unwrap();

                HttpResponse::Ok()
                    .content_type(ContentType::json())
                    .body(body)
            }
            Err(e) => HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(serde_json::to_string(&AppErrorResponde { error: e.clone() }).unwrap()),
        }
    }
}

#[derive(Deserialize)]
struct SignupRequest {
    username: String,
    #[serde(deserialize_with = "deserialize_base64")]
    srp_salt: Vec<u8>,
    #[serde(deserialize_with = "deserialize_base64")]
    srp_verifier: Vec<u8>,
    #[serde(deserialize_with = "deserialize_base64")]
    private_key_encrypted: Vec<u8>,
    #[serde(deserialize_with = "deserialize_base64")]
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
    #[serde(deserialize_with = "deserialize_base64")]
    srp_client_public_key: Vec<u8>,
}

#[derive(Serialize)]
struct SRPHelloResponse {
    #[serde(serialize_with = "serialize_base64")]
    srp_salt: Vec<u8>,
    #[serde(serialize_with = "serialize_base64")]
    srp_server_public_key: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct SRPSession {
    username: String,
    #[serde(
        serialize_with = "serialize_base64",
        deserialize_with = "deserialize_base64"
    )]
    salt: Vec<u8>,
    #[serde(
        serialize_with = "serialize_base64",
        deserialize_with = "deserialize_base64"
    )]
    verifier: Vec<u8>,
    #[serde(
        serialize_with = "serialize_base64",
        deserialize_with = "deserialize_base64"
    )]
    client_public_key: Vec<u8>,
    #[serde(
        serialize_with = "serialize_base64",
        deserialize_with = "deserialize_base64"
    )]
    server_private_key: Vec<u8>,
    #[serde(
        serialize_with = "serialize_base64",
        deserialize_with = "deserialize_base64"
    )]
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
    #[serde(deserialize_with = "deserialize_base64")]
    srp_evidence: Vec<u8>,
}

#[derive(Serialize)]
struct SRPAuthResponse {
    #[serde(serialize_with = "serialize_base64")]
    srp_evidence: Vec<u8>,
}

#[post("auth")]
async fn srp_auth(
    session: Session,
    data: web::Data<AppState>,
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

                AppResult::ok(SRPAuthResponse {
                    srp_evidence: verifier.proof().to_vec(),
                })
            }
        },
    }
}

pub fn global_error_handler<B>(
    res: ServiceResponse<B>,
) -> actix_web::Result<ErrorHandlerResponse<B>> {
    println!("Called 2");
    let status = res.status();

    let (req, res) = res.into_parts();
    let err = match status {
        StatusCode::NOT_FOUND => AppError::NotFound,
        _ => AppError::Internal,
    };

    let res =
        res.set_body(serde_json::to_string(&AppErrorResponde { error: err.clone() }).unwrap());

    let res = ServiceResponse::new(req, res)
        .map_into_boxed_body()
        .map_into_right_body();

    Ok(ErrorHandlerResponse::Response(res))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // let username = "levirs";
    // let password = "1234567890";
    // let client = SrpClient::<Sha256>::new(&G_2048);
    // let server = SrpServer::<Sha256>::new(&G_2048);

    // let salt = hex::decode("92f501c7897ccc82689ced20a8455bcb224bbe253b808ac360e01c845385c286").unwrap();
    // // let mut salt = [0u8; 32];
    // // rand::rng().fill_bytes(&mut salt);
    // println!("Salt {}", hex::encode(&salt));

    // let v = client.compute_verifier(username.as_bytes(), password.as_bytes(), &salt);
    //  println!("v {}", hex::encode(&v));

    // let a = hex::decode("468219397831e044fc81ee8f47994681ec08eb590a62658de7ab6de92fc98963").unwrap();
    // // let mut a = [0u8; 32];
    // // rand::rng().fill_bytes(&mut a);
    // println!("a {}", hex::encode(&a));
    // let a_pub = client.compute_public_ephemeral(&a);
    // println!("a_pub {}", hex::encode(&a_pub));

    // let b = hex::decode("afaf79d41b080ce60571ad66132798bdc1e71397eb821188ac764175fd99934a").unwrap();
    // // let mut b = [0u8; 32];
    // // rand::rng().fill_bytes(&mut b);
    // println!("b {}", hex::encode(&b));
    // let b_pub = server.compute_public_ephemeral(&b, &v);
    // println!("b_pub {}", hex::encode(&b_pub));

    // // let private_key = (username, password, salt);
    // let client_verifier = client.process_reply_rfc5054(&a, username.as_bytes(), password.as_bytes(), &salt, &b_pub).unwrap();

    // let server_verifier = server.process_reply_rfc5054(username.as_bytes(), &salt, &b, v.as_slice(), &a_pub).unwrap();
    // println!("client_key {}", hex::encode(client_verifier.key()));
    // println!("client_proof: {}", hex::encode(client_verifier.proof()));


    // let identity_hash =  SrpClient::<Sha256>::compute_identity_hash(username.as_bytes(), password.as_bytes());
    // let x =  SrpClient::<Sha256>::compute_x(&identity_hash, &salt);
    // println!("id_hash: {}", hex::encode(identity_hash));
    // println!("x: {}", hex::encode(x.to_bytes_be()));
    // server_verifier.verify_client(client_verifier.proof()).unwrap();

    // // let private_key = (username, password, salt);
    // // let verifier = client.process_reply(&a, username, password, salt, b_pub);
    // return Ok(());

    if let Err(e) = dotenvy::dotenv() {
        eprintln!("Warning: fail to load .env: {e:?}")
    }

    let url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL is not set");
    let session_key = env::var("SESSION_KEY").expect("SESSION_KEY is not set");

    println!("Connecting to database..");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("Connect to database failed");

    let app_data = web::Data::new(AppState { db_pool: pool });

    println!("Connecting to Redis");
    let redis_store = RedisSessionStore::new(redis_url).await.unwrap();

    println!("Starting server...");
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            .allow_any_method()
            .allow_any_header()
            .supports_credentials()
            .max_age(3600);
        App::new()
            .wrap(cors)
            .wrap(
                ErrorHandlers::default()
                    .handler(actix_web::http::StatusCode::NOT_FOUND, global_error_handler)
                    .handler(
                        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
                        global_error_handler,
                    )
                    .default_handler(global_error_handler),
            )
            .app_data(JsonConfig::default().error_handler(|err, _| {
                AppError::BadRequest {
                    message: format!("{}", err),
                }
                .into()
            }))
            .wrap(
                SessionMiddleware::builder(redis_store.clone(), Key::from(session_key.as_bytes()))
                    .cookie_secure(true)
                    .build(),
            )
            .app_data(app_data.clone())
            .service(srp_hello)
            .service(signup)
            .service(srp_auth)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
