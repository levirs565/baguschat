use std::env;

use actix::Actor;
use actix_cors::Cors;
use actix_session::{storage::RedisSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::middleware::ErrorHandlers;
use actix_web::web::JsonConfig;
use actix_web::{web, App, HttpServer};
use aws_sdk_s3::Config;
use sqlx::postgres::PgPoolOptions;

use crate::chat::ChatRouter;
use crate::core::{global_error_handler, AppError, AppState};
mod auth;
mod chat;
mod core;
mod user;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if let Err(e) = dotenvy::dotenv() {
        eprintln!("Warning: fail to load .env: {e:?}")
    }

    let port = env::var("PORT").unwrap_or(String::from("8080"));
    let url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL is not set");
    let session_key = env::var("SESSION_KEY").expect("SESSION_KEY is not set");
    let s3_bucket = env::var("S3_BUCKET").expect("S3_BUCKET is not set");
    let cors_origin = env::var("CORS_ORIGIN").expect("CORS_ORIGIN is not set");

    println!("Connecting to database..");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("Connect to database failed");

    println!("Connecting to Redis");
    let redis_store = RedisSessionStore::new(redis_url).await.unwrap();

    println!("Connecting to S3");
    let s3_config = aws_config::load_from_env().await;
    let s3_config: Config = (&s3_config).into();
    let s3_config = (&s3_config).to_builder().force_path_style(true).build();
    let s3_client = aws_sdk_s3::Client::from_conf(s3_config);

    let app_data = web::Data::new(AppState {
        db_pool: pool,
        s3: s3_client,
        s3_bucket: s3_bucket,
    });

    println!("Starting server...");

    let chat_router = web::Data::new(ChatRouter::default().start());

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&cors_origin)
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
            .app_data(chat_router.clone())
            .service(crate::auth::scope())
            .service(crate::user::scope())
            .service(crate::chat::scope())
    })
    .bind(("0.0.0.0", port.parse().unwrap()))?
    .run()
    .await
}
