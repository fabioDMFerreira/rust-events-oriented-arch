use std::{sync::Arc, time::Instant};

use crate::http::{
    services::auth_service::AuthService,
    websockets::{ws_server::WebsocketServer, ws_session::WebsocketSession},
};
use actix::prelude::*;
use actix_web::{get, web, Error as ActixError, HttpRequest, HttpResponse};
use actix_web_actors::ws;

#[get("/connect-ws")]
pub async fn get_ws(
    r: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<WebsocketServer>>,
    auth_service: web::Data<dyn AuthService + 'static>,
) -> Result<HttpResponse, ActixError> {
    ws::start(
        WebsocketSession {
            id: String::from("0"),
            hb: Instant::now(),
            addr: srv.get_ref().clone(),
            authenticated: false,
            auth_service: Arc::clone(&auth_service),
        },
        &r,
        stream,
    )
}
