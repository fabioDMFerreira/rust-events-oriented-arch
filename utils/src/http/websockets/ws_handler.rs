use std::{sync::Arc, time::Instant};

use crate::http::websockets::{ws_server::WebsocketServer, ws_session::WebsocketSession};
use actix::prelude::*;
use actix_web::{get, web, Error as ActixError, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::http::middlewares::jwt_auth::JwtMiddlewareConfig;

#[get("/connect-ws")]
pub async fn get_ws(
    r: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<WebsocketServer>>,
    config: web::Data<dyn JwtMiddlewareConfig + 'static>,
) -> Result<HttpResponse, ActixError> {
    let config = Arc::clone(&config);

    ws::start(
        WebsocketSession {
            id: String::from("0"),
            hb: Instant::now(),
            addr: srv.get_ref().clone(),
            authenticated: false,
            config: config.clone(),
        },
        &r,
        stream,
    )
}
