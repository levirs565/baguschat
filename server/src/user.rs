use actix_session::Session;
use actix_web::{get, web, Scope};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::guard_auth;
use crate::core::{AppError, AppResponse, AppState};
use crate::utils::base64_field;

#[derive(Serialize)]
struct GetUserResponse {
    id: Uuid,
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
            "SELECT username, public_key FROM Users WHERE id = $1 LIMIT 5",
            user_id
        )
        .fetch_one(&data.db_pool)
        .await
        .map_err(|_| AppError::UserNotFound)?;

        Ok(GetUserResponse {
            id: user_id,
            username: data.username,
            public_key: data.public_key,
        })
    })
    .await
}

#[derive(Deserialize)]
struct ListQuery {
    username: String,
}

#[get("")]
async fn get_list(
    session: Session,
    data: web::Data<AppState>,
    info: web::Query<ListQuery>,
) -> AppResponse<Vec<GetUserResponse>> {
    AppResponse::wrap_async(|| async {
        guard_auth(&session, true)?;

        let filter = format!("{}%", info.username);

        let data = sqlx::query!(
            "SELECT id, username, public_key FROM Users WHERE username LIKE $1",
            filter
        )
        .fetch_all(&data.db_pool)
        .await
        .map_err(|_| AppError::UserNotFound)?;

        let mapped = data.iter().map(|raw| GetUserResponse {
            id: raw.id,
            username: raw.username.clone(),
            public_key: raw.public_key.clone(),
        });

        Ok(mapped.collect())
    })
    .await
}

pub fn scope() -> Scope {
    web::scope("/user").service(get).service(get_list)
}
