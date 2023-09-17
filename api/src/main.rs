#[macro_use]
extern crate log;

use actix_web::HttpServer;
use api::app;
use utils::logger::init_logger;

use api::config::Config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::init();

    init_logger(config.logs_path.clone());

    let server_port = config.server_port.clone();

    // http server
    info!("Starting API server in port {}", config.server_port.clone());

    HttpServer::new(move || app::setup_app(&config))
        .bind(format!("0.0.0.0:{}", server_port))?
        .run()
        .await
}
