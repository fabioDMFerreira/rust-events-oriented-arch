#[macro_use]
extern crate log;

use actix::Actor;
use actix_web::HttpServer;
use users::app;
use users::config::Config;
use utils::http::websockets::ws_server::WebsocketServer;
use utils::logger::init_logger;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::init();

    init_logger(config.logs_path.clone());

    let server_port = config.server_port.clone();

    // http server
    info!("Starting API server in port {}", config.server_port.clone());

    let ws_server = WebsocketServer::new().start();

    HttpServer::new(move || app::setup_app(&config, ws_server.clone()))
        .bind(format!("0.0.0.0:{}", server_port))?
        .run()
        .await
}
