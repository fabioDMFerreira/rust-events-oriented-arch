use std::{sync::Arc, time::Instant};

use actix::prelude::*;
use actix_web::{get, web, Error as ActixError, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::{
    actors::{ws_server::WebsocketServer, ws_session::WebsocketSession},
    config::Config,
};

#[get("/ws")]
pub async fn get_ws(
    r: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<WebsocketServer>>,
    config: web::Data<Config>,
) -> Result<HttpResponse, ActixError> {
    ws::start(
        WebsocketSession {
            id: String::from("0"),
            hb: Instant::now(),
            addr: srv.get_ref().clone(),
            authenticated: false,
            config: Arc::new(config.get_ref().clone()),
        },
        &r,
        stream,
    )
}
