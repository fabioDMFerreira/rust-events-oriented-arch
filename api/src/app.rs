use std::sync::Arc;

use actix::Actor;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::{error::Error, web, App};
use utils::http_server::logger;
use utils::{broker, db, http_server::cors};

use crate::actors::ws_server::WebsocketServer;
use crate::config::Config;
use crate::handlers::auth::{login_handler, logout_handler, me_handler};
use crate::handlers::health::get_health;
use crate::handlers::index::get_index;
use crate::handlers::message::create_message;
use crate::handlers::user::{create_user, delete_user, get_user_by_id, get_users, update_user};
use crate::handlers::ws::get_ws;
use crate::repositories::user_repository::{UserDieselRepository, UserRepository};
use crate::services::event_service::KafkaEventService;
use crate::services::user_service::{UserService, UserServiceImpl};

pub fn setup_app(
    config: &Config,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = Error,
    >,
> {
    let db_connection = db::connect_db(config.database_url.clone());
    let kafka_producer = broker::create_producer(config.kafka_url.clone());

    let ws_server = WebsocketServer::new().start();

    // repositories
    let user_repo: Arc<dyn UserRepository> =
        Arc::new(UserDieselRepository::new(Arc::new(db_connection.clone())));

    // services
    let events_service = Arc::new(KafkaEventService::new(kafka_producer.clone()));
    let user_service: Arc<dyn UserService> =
        Arc::new(UserServiceImpl::new(user_repo, events_service));

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
}