use actix_session::Session;
use actix_web::{Scope, get, web};
use serde::Serialize;
use uuid::Uuid;

use crate::auth::guard_auth;
use crate::core::{AppError, AppResponse, AppState};
use crate::utils::base64_field;

#[derive(Serialize)]
struct GetUserResponse {
    username: String,
    #[serde(with = "base64_field")]
    public_key: Vec<u8>,
}

#[get("{id}")]
async fn get(
    session: Session,
    data: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> AppResponse<GetUserResponse> {
    AppResponse::wrap_async(|| async {
        guard_auth(&session, true)?;

        let user_id = path.into_inner();

        let data = sqlx::query!(
            "SELECT username, public_key FROM Users WHERE id = $1",
            user_id
        )
        .fetch_one(&data.db_pool)
        .await
        .map_err(|_| AppError::UserNotFound)?;

        Ok(GetUserResponse {
            username: data.username,
            public_key: data.public_key,
        })
    })
    .await
}

pub fn scope() -> Scope {
    web::scope("/user").service(get)
}
