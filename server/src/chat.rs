use std::collections::HashMap;
use std::ops::Add;
use std::time::{Duration, Instant};

use actix::prelude::*;
use actix::Actor;
use actix_web::{get, rt, web, Error, HttpRequest, HttpResponse, Scope};
use actix_web_actors::ws::{self, WebsocketContext};
use futures_util::{StreamExt as _, TryFutureExt};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::types::chrono;
use uuid::Uuid;

use crate::auth::{get_userid, guard_auth};
use crate::core::{ActionResult, AppError, AppResponse, AppState, DBPool};
use crate::utils::base64_field;

#[derive(FromRow, Serialize)]
struct ChatItem {
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
) -> AppResponse<Vec<ChatItem>> {
    AppResponse::wrap_async(|| async {
        guard_auth(&session, true)?;

        let user_id = get_userid(&session).unwrap();

        sqlx::query_as!(
            ChatItem,
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
    SendChat(SendChatRequest),
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum WsReponseMessage {
    ReceiveChat(ChatItem),
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "conversation_type", rename_all = "lowercase")]
pub enum ConversationType {
    Text,
    File,
    Image,
}

async fn send_chat(
    router_addr: Addr<ChatRouter>,
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
            ) RETURNING id, created_At"#,
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

        router_addr.do_send(RouteChat {
            chat_item: ChatItem {
                id: data.id,
                created_at: data.created_at,
                sender_id,
                receiver_id: Some(request.receiver_id),
                sender_key: request.sender_key,
                receiver_key: request.receiver_key,
                cipher: request.message_cipher,
            },
        });

        Ok(ActionResult::success())
    })
    .await
}

struct WsConnection {
    router_addr: Addr<ChatRouter>,
    hb: Instant,
    user_id: Uuid,
    db_pool: DBPool,
}

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

impl WsConnection {
    fn new(router_addr: Addr<ChatRouter>, user_id: Uuid, db_pool: DBPool) -> WsConnection {
        WsConnection {
            router_addr: router_addr,
            user_id: user_id,
            db_pool: db_pool,
            hb: Instant::now(),
        }
    }

    fn hb(&self, ctx: &mut WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                ctx.stop();
            }

            ctx.ping(b"PING");
        });
    }
}

impl Actor for WsConnection {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        self.router_addr
            .send(Connect {
                addr: ctx.address().recipient(),
                user_id: self.user_id,
            })
            .into_actor(self)
            .then(|res, _, ctx| {
                match res {
                    Ok(_) => (),
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.router_addr.do_send(Disconnect {
            user_id: self.user_id,
        });
        Running::Stop
    }
}

impl Handler<RouteChat> for WsConnection {
    type Result = ();

    fn handle(&mut self, msg: RouteChat, ctx: &mut Self::Context) -> Self::Result {
        let data = WsReponseMessage::ReceiveChat(msg.chat_item);
        let json = serde_json::to_string(&data).unwrap();

        ctx.text(json);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                let json = serde_json::from_str::<WsRequestMessage>(&text);

                match json {
                    Err(_) => ctx.text("Error"),
                    Ok(v) => match v {
                        WsRequestMessage::SendChat(data) => {
                            let router_addr = self.router_addr.clone();
                            let pool = self.db_pool.clone();
                            let user_id = self.user_id.clone();
                            let fut =
                                async move { send_chat(router_addr, &pool, user_id, data).await };
                            ctx.spawn(fut.into_actor(self).map(|res, _, ctx| {
                                match res.0 {
                                    Ok(_) => ctx.text("Success"),
                                    Err(_) => ctx.text("Fail"),
                                }
                                ()
                            }));
                        }
                    },
                }
            }
            Ok(ws::Message::Binary(bin)) => {
                ctx.binary(bin);
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                ctx.stop();
            }
            Ok(ws::Message::Nop) => {}
            Err(e) => {
                eprintln!("Unexpected error! {e}");
            }
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct Connect {
    addr: Recipient<RouteChat>,
    user_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
struct Disconnect {
    user_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
struct RouteChat {
    chat_item: ChatItem,
}

pub struct ChatRouter {
    sessions: HashMap<Uuid, Recipient<RouteChat>>,
}

impl Default for ChatRouter {
    fn default() -> Self {
        Self {
            sessions: Default::default(),
        }
    }
}

impl Actor for ChatRouter {
    type Context = Context<Self>;
}

impl Handler<Connect> for ChatRouter {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Self::Context) -> Self::Result {
        self.sessions.insert(msg.user_id, msg.addr);
    }
}

impl Handler<Disconnect> for ChatRouter {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
        self.sessions.remove(&msg.user_id);
    }
}

impl Handler<RouteChat> for ChatRouter {
    type Result = ();

    fn handle(&mut self, msg: RouteChat, _: &mut Self::Context) -> Self::Result {
        if let Some(receiver_id) = msg.chat_item.receiver_id {
            if let Some(addr) = self.sessions.get(&receiver_id) {
                addr.do_send(msg);
            }
        }
    }
}

async fn ws(
    req_session: actix_session::Session,
    router_addr: web::Data<Addr<ChatRouter>>,
    data: web::Data<AppState>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    guard_auth(&req_session, true)?;

    let user_id = get_userid(&req_session).unwrap();
    let db_pool = data.db_pool.clone();
    let ws_connection = WsConnection::new(router_addr.get_ref().clone(), user_id, db_pool);

    let res = ws::start(ws_connection, &req, stream)?;

    Ok(res)
}

pub fn scope() -> Scope {
    web::scope("/chat")
        .service(get)
        .route("ws", web::get().to(ws))
}
