use std::env;

use actix_cors::Cors;
use actix_session::{storage::RedisSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::middleware::ErrorHandlers;
use actix_web::web::JsonConfig;
use actix_web::{App, HttpServer, web};
use sqlx::postgres::PgPoolOptions;

use crate::core::{AppError, AppState, global_error_handler};
mod core;
mod auth;
mod user;
mod chat;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
            .service(crate::auth::scope())
            .service(crate::user::scope())
            .service(crate::chat::scope())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
