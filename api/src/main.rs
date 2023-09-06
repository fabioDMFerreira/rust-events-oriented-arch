#[macro_use]
extern crate log;

use std::fs::File;
use std::io::Write;
use std::sync::Arc;

use actix::Actor;
use actix_cors::Cors;
use actix_web::http::header;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use chrono::Local;
use env_logger::Env;

use api::actors::ws_server::WebsocketServer;
use api::config::Config;
use api::core::db;
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
    let target = Box::new(File::create("/var/log/app/stdout.log").expect("Can't create file"));

    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .target(env_logger::Target::Pipe(target))
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {} {}:{} {}",
                Local::now().format("%b %d %H:%M:%S"),
                record.level(),
                record.file_static().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .init();

    info!("Starting API server in port 8000");

    let config = Config::init();

    let db_connection = db::connect_db(config.database_url.clone());

    let user_repo: Arc<dyn UserRepository> =
        Arc::new(UserDieselRepository::new(Arc::new(db_connection.clone())));

    let events_service = Arc::new(KafkaEventService::new(config.kafka_url.clone()));

    let user_service: Arc<dyn UserService> =
        Arc::new(UserServiceImpl::new(user_repo, events_service));

    let chat_server = WebsocketServer::new().start();

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ])
            .supports_credentials();

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .app_data(web::Data::new(db_connection.clone()))
            .app_data(web::Data::from(user_service.clone()))
            .app_data(web::Data::new(chat_server.clone()))
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
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
