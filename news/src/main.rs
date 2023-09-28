extern crate log;

use actix::{Actor, Addr};
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::{error::Error as ActixError, web, App as ActixApp, HttpServer};
use log::info;
use news::handlers::subscriptions::{create_subscription, delete_subscription, get_subscriptions};
use news::news_websocket_processor::NewsWebsocketProcessor;
use rdkafka::consumer::{Consumer, StreamConsumer};
use std::sync::Arc;
use std::thread;
use utils::broker;
use utils::http::services::auth_service::{AuthService, JwtAuthService};
use utils::http::websockets::ws_handler::get_ws;
use utils::http::websockets::ws_sender::WsSenderWrapper;
use utils::http::websockets::ws_server::WebsocketServer;
use utils::news::events::NEWS_CREATED_EVENT;
use utils::news::repositories::feed_repository::{FeedDieselRepository, FeedRepository};
use utils::news::repositories::news_repository::{NewsDieselRepository, NewsRepository};
use utils::news::repositories::subscription_repository::{
    SubscriptionRepository, SubscriptionsDieselRepository,
};
use utils::pipeline::consumer::KafkaConsumer;
use utils::pipeline::data_pipeline::DataPipeline;
use utils::{db::connect_db, http::utils::build_server, logger::init_logger};

use news::{config::Config, handlers::feeds::get_feeds, handlers::news::get_news};

#[actix_web::main]
async fn main() {
    let config = Config::init();

    init_logger(config.logs_path.clone());

    let db_pool = connect_db(config.database_url.clone());

    let feed_repository: Arc<dyn FeedRepository> =
        Arc::new(FeedDieselRepository::new(Arc::new(db_pool.clone())));
    let news_repository: Arc<dyn NewsRepository> =
        Arc::new(NewsDieselRepository::new(Arc::new(db_pool.clone())));
    let subscription_repository: Arc<dyn SubscriptionRepository> = Arc::new(
        SubscriptionsDieselRepository::new(Arc::new(db_pool.clone())),
    );

    let server_port = config.server_port.clone();

    info!("Starting API server in port {}", server_port.clone());

    let ws_server = WebsocketServer::new().start();
    let consumer = broker::create_consumer(config.kafka_url.to_string());

    setup_news_created_pipeline(consumer, &ws_server, subscription_repository.clone());

    let server_result = HttpServer::new(move || {
        setup_http_server(
            &config,
            feed_repository.clone(),
            news_repository.clone(),
            subscription_repository.clone(),
            ws_server.clone(),
        )
    })
    .bind(format!("0.0.0.0:{}", server_port.clone()));

    match server_result {
        Ok(server) => {
            if let Err(err) = server.run().await {
                panic!("failed running server: {}", err)
            }
        }
        Err(err) => {
            panic!("failed building server: {}", err)
        }
    }

    thread::park();
}

fn setup_http_server(
    config: &Config,
    feed_repo: Arc<dyn FeedRepository>,
    news_repo: Arc<dyn NewsRepository>,
    subscription_repo: Arc<dyn SubscriptionRepository>,
    ws_server: Addr<WebsocketServer>,
) -> ActixApp<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = ActixError,
    >,
> {
    let auth_service: Arc<dyn AuthService> =
        Arc::new(JwtAuthService::new(config.jwt_secret.clone()));

    build_server(config.cors_origin.clone())
        .app_data(web::Data::from(feed_repo.clone()))
        .app_data(web::Data::from(news_repo.clone()))
        .app_data(web::Data::from(subscription_repo.clone()))
        .app_data(web::Data::new(config.clone()))
        .app_data(web::Data::new(ws_server.clone()))
        .app_data(web::Data::from(auth_service.clone()))
        .service(get_news)
        .service(get_feeds)
        .service(get_subscriptions)
        .service(create_subscription)
        .service(delete_subscription)
        .service(get_ws)
}

fn setup_news_created_pipeline(
    consumer: StreamConsumer,
    ws_server: &Addr<WebsocketServer>,
    subscription_repo: Arc<dyn SubscriptionRepository>,
) {
    let ws_sender = WsSenderWrapper::new(ws_server.clone());

    actix_rt::spawn(async move {
        consumer
            .subscribe(&[NEWS_CREATED_EVENT])
            .expect("Error subscribing to topic");
        let processor = NewsWebsocketProcessor::new(&ws_sender, subscription_repo);
        let consumer = KafkaConsumer::new(consumer);
        let pipeline = DataPipeline::new(&consumer, &processor);

        pipeline.start().await;
    });
}
