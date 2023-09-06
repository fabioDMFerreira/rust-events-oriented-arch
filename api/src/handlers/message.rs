use crate::actors::ws_server::{SessionMessage, WebsocketServer};
use actix::Addr;
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct MessagePayload {
    message: String,
    user_id: String,
}

#[post("/messages")]
async fn create_message(
    ws_server: Data<Addr<WebsocketServer>>,
    payload: Json<MessagePayload>,
) -> HttpResponse {
    let MessagePayload { message, user_id } = payload.into_inner();

    ws_server.do_send(SessionMessage {
        id: user_id,
        message: message,
    });

    HttpResponse::Ok().finish()
}
