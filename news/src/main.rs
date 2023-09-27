extern crate log;

use actix::{Actor, Addr};
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::{error::Error as ActixError, web, App as ActixApp, HttpServer};
use log::info;
use news::handlers::subscriptions::{create_subscription, delete_subscription, get_subscriptions};
use news::news_created_subscriber::setup_news_created_subscriber;
use news::repositories::feed_repository::FeedDieselRepository;
use news::repositories::news_repository::NewsDieselRepository;
use news::repositories::subscription_repository::{
    SubscriptionRepository, SubscriptionsDieselRepository,
};
use std::thread;
use std::{error::Error, sync::Arc};
use tokio_cron_scheduler::{Job, JobScheduler};
use utils::broker::{self};
use utils::http::services::auth_service::JwtAuthService;
use utils::http::websockets::ws_handler::get_ws;
use utils::http::websockets::ws_server::WebsocketServer;
use utils::{db::connect_db, http::utils::build_server, logger::init_logger};

use news::{
    app::App,
    config::Config,
    handlers::feeds::get_feeds,
    handlers::news::get_news,
    repositories::{feed_repository::FeedRepository, news_repository::NewsRepository},
};

#[actix_web::main]
async fn main() {
    let config = Config::init();

    init_logger(config.logs_path.clone());

    let kafka_producer = broker::create_producer(config.kafka_url.clone());

    let db_pool = connect_db(config.database_url.clone());

    let feed_repository: Arc<dyn FeedRepository> =
        Arc::new(FeedDieselRepository::new(Arc::new(db_pool.clone())));
    let news_repository: Arc<dyn NewsRepository> =
        Arc::new(NewsDieselRepository::new(Arc::new(db_pool.clone())));
    let subscription_repository: Arc<dyn SubscriptionRepository> = Arc::new(
        SubscriptionsDieselRepository::new(Arc::new(db_pool.clone())),
    );

    let app = App::new(
        feed_repository.clone(),
        news_repository.clone(),
        kafka_producer,
    );

    info!("Setting up cronjobs");

    if let Err(err) = setup_cronjobs(&app).await {
        panic!("failed setup cronjobs: {}", err);
    };

    let server_port = config.server_port.clone();

    info!("Starting API server in port {}", server_port.clone());

    let ws_server = WebsocketServer::new().start();

    let config_clone = config.clone();
    let ws_server_clone = ws_server.clone();
    let subscription_repo_clone = subscription_repository.clone();

    actix_rt::spawn(async move {
        setup_news_created_subscriber(&config_clone, ws_server_clone, subscription_repo_clone).await
    });

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

async fn setup_cronjobs(app: &App) -> Result<(), Box<dyn Error>> {
    let app = app.clone();

    let sched = JobScheduler::new().await?;

    let scrap_news_job = Job::new_async("0 * * * * *", move |_uuid, _l| {
        let app = app.clone();
        Box::pin(async move {
            app.scrap_feeds().await;
        })
    })?;
    sched.add(scrap_news_job).await?;

    sched.start().await?;

    Ok(())
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
    let jwt_config = Arc::new(config.clone());
    let auth_service = Arc::new(JwtAuthService::new(config.jwt_secret.clone()));

    build_server(config.cors_origin.clone())
        .app_data(web::Data::from(feed_repo.clone()))
        .app_data(web::Data::from(news_repo.clone()))
        .app_data(web::Data::from(subscription_repo.clone()))
        .app_data(web::Data::new(config.clone()))
        .app_data(web::Data::from(jwt_config.clone()))
        .app_data(web::Data::new(ws_server.clone()))
        .app_data(web::Data::new(auth_service))
        .service(get_news)
        .service(get_feeds)
        .service(get_subscriptions)
        .service(create_subscription)
        .service(delete_subscription)
        .service(get_ws)
}
