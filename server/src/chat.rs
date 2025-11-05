use actix_web::{get, rt, web, Error, HttpRequest, HttpResponse, Scope};
use actix_ws::AggregatedMessage;
use futures_util::{StreamExt as _, TryFutureExt};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::types::chrono;
use uuid::Uuid;

use crate::auth::{get_userid, guard_auth};
use crate::core::{ActionResult, AppError, AppResponse, AppState, DBPool};
use crate::utils::base64_field;

#[derive(FromRow, Serialize)]
struct GetResponseItem {
    id: Uuid,
    created_at: chrono::NaiveDateTime,
    sender_id: Uuid,
    receiver_id: Option<Uuid>,
    #[serde(with = "base64_field")]
    sender_key: Vec<u8>,
    #[serde(with = "base64_field")]
    receiver_key: Vec<u8>,
    #[serde(with = "base64_field")]
    cipher: Vec<u8>,
}

#[get("")]
async fn get(
    session: actix_session::Session,
    data: web::Data<AppState>,
) -> AppResponse<Vec<GetResponseItem>> {
    AppResponse::wrap_async(|| async {
        guard_auth(&session, true)?;

        let user_id = get_userid(&session).unwrap();

        sqlx::query_as!(
            GetResponseItem,
            r#"
            SELECT 
                conversations.id, 
                created_at, 
                sender_id,
                receiver_id,
                sender_key,
                receiver_key,
                cipher
            FROM conversations
            JOIN conversations_text ON conversations.id = conversations_text.id
            WHERE conversations.receiver_id = $1 OR conversations.sender_id = $1
            ORDER BY conversations.created_at DESC
            "#,
            user_id
        )
        .fetch_all(&data.db_pool)
        .await
        .map_err(|_| AppError::Internal)
    })
    .await
}

#[derive(Deserialize)]
struct SendChatRequest {
    receiver_id: Uuid,
    #[serde(with = "base64_field")]
    sender_key: Vec<u8>,
    #[serde(with = "base64_field")]
    receiver_key: Vec<u8>,
    #[serde(with = "base64_field")]
    message_cipher: Vec<u8>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum WsRequestMessage {
    SendChatRequest(SendChatRequest),
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "conversation_type", rename_all = "lowercase")]
pub enum ConversationType {
    Text,
    File,
    Image,
}

async fn send_chat(
    db_pool: &DBPool,
    sender_id: Uuid,
    request: SendChatRequest,
) -> AppResponse<ActionResult> {
    AppResponse::wrap_async(|| async {
        let mut tx = db_pool.begin().await.map_err(|_| AppError::Internal)?;

        let data = sqlx::query!(
            r#"INSERT INTO conversations(
                sender_id,
                receiver_id,
                sender_key,
                receiver_key,
                contet_type
            ) VALUES (
                $1, $2, $3, $4, $5
            ) RETURNING id"#,
            sender_id,
            request.receiver_id,
            request.sender_key,
            request.receiver_key,
            ConversationType::Text as ConversationType
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| AppError::Internal)
        .unwrap();

        sqlx::query!(
            r#"INSERT INTO conversations_text(
                id,
                cipher
            ) VALUES (
                $1, $2
            )"#,
            data.id,
            request.message_cipher
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| AppError::Internal)?;

        tx.commit().await.map_err(|_| AppError::Internal)?;

        Ok(ActionResult::success())
    })
    .await
}

async fn ws(
    req_session: actix_session::Session,
    data: web::Data<AppState>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    guard_auth(&req_session, true)?;

    let user_id = get_userid(&req_session).unwrap();
    let db_pool = data.db_pool.clone();
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        .max_continuation_size(2_usize.pow(20));

    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    let json = serde_json::from_str::<WsRequestMessage>(&text);

                    match json {
                        Err(_) => session.text("Error").await.unwrap(),
                        Ok(v) => match v {
                            WsRequestMessage::SendChatRequest(data) => {
                                let res = send_chat(&db_pool, user_id, data).await;
                                match res.0 {
                                    Ok(_) => session.text("Success"),
                                    Err(_) => session.text("Fail"),
                                }
                                .await
                                .unwrap()
                            }
                        },
                    }
                }
                Ok(AggregatedMessage::Binary(bin)) => {
                    session.binary(bin).await.unwrap();
                }

                Ok(AggregatedMessage::Ping(msg)) => {
                    session.pong(&msg).await.unwrap();
                }

                _ => {}
            }
        }
    });

    Ok(res)
}

pub fn scope() -> Scope {
    web::scope("/chat")
        .service(get)
        .route("ws", web::get().to(ws))
}
