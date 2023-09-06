#[macro_use]
extern crate log;

use std::sync::Arc;

use actix::Actor;
use actix_web::{web, App, HttpServer};
use utils::http_server::logger;
use utils::logger::init_logger;
use utils::{broker, db, http_server::cors};

use api::actors::ws_server::WebsocketServer;
use api::config::Config;
use api::handlers::auth::{login_handler, logout_handler, me_handler};
use api::handlers::health::get_health;
use api::handlers::index::get_index;
use api::handlers::message::create_message;
use api::handlers::user::{create_user, delete_user, get_user_by_id, get_users, update_user};
use api::handlers::ws::get_ws;
use api::repositories::user_repository::{UserDieselRepository, UserRepository};
use api::services::event_service::KafkaEventService;
use api::services::user_service::{UserService, UserServiceImpl};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // infra
    init_logger();
    let config = Config::init();
    let db_connection = db::connect_db(config.database_url.clone());
    let kafka_producer = broker::create_producer(config.kafka_url.clone());

    // repositories
    let user_repo: Arc<dyn UserRepository> =
        Arc::new(UserDieselRepository::new(Arc::new(db_connection.clone())));

    // services
    let events_service = Arc::new(KafkaEventService::new(kafka_producer.clone()));
    let user_service: Arc<dyn UserService> =
        Arc::new(UserServiceImpl::new(user_repo, events_service));

    // http server
    info!("Starting API server in port {}", config.server_port.clone());

    let ws_server = WebsocketServer::new().start();

    let server_port = config.server_port.clone();

    HttpServer::new(move || {
        App::new()
            .wrap(cors(config.cors_origin.clone()))
            .wrap(logger())
            .app_data(web::Data::new(db_connection.clone()))
            .app_data(web::Data::from(user_service.clone()))
            .app_data(web::Data::new(ws_server.clone()))
            .app_data(web::Data::new(config.clone()))
            .service(get_index)
            .service(get_health)
            .service(get_ws)
            .service(get_users)
            .service(get_user_by_id)
            .service(create_user)
            .service(update_user)
            .service(delete_user)
            .service(login_handler)
            .service(logout_handler)
            .service(me_handler)
            .service(create_message)
    })
    .bind(format!("0.0.0.0:{}", server_port))?
    .run()
    .await
}
