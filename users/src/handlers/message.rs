use crate::actors::ws_server::{SessionMessage, WebsocketServer};
use actix::Addr;
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
struct MessagePayload {
    #[validate(required, length(min = 1))]
    message: Option<String>,
    #[validate(required, length(min = 1))]
    user_id: Option<String>,
}

#[post("/messages")]
async fn create_message(
    ws_server: Data<Addr<WebsocketServer>>,
    payload: Option<Json<MessagePayload>>,
) -> HttpResponse {
    if payload.is_none() {
        return HttpResponse::BadRequest().body("empty body");
    }

    let payload = payload.unwrap().into_inner();
    if let Err(err) = payload.validate() {
        return HttpResponse::BadRequest().json(err);
    }

    let MessagePayload { message, user_id } = payload;

    ws_server.do_send(SessionMessage {
        id: user_id.unwrap(),
        message: message.unwrap(),
    });

    HttpResponse::Ok().finish()
}
