extern crate log;

use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::{error::Error as ActixError, web, App as ActixApp, HttpServer};
use log::info;
use news::handlers::subscriptions::{create_subscription, delete_subscription, get_subscriptions};
use news::repositories::feed_repository::FeedDieselRepository;
use news::repositories::news_repository::NewsDieselRepository;
use news::repositories::subscription_repository::{
    SubscriptionRepository, SubscriptionsDieselRepository,
};
use std::thread;
use std::{error::Error, sync::Arc};
use tokio_cron_scheduler::{Job, JobScheduler};
use utils::http::middlewares::jwt_auth::JwtMiddlewareConfig;
use utils::{db::connect_db, http::utils::build_server, logger::init_logger};

use news::{
    app::App,
    config::Config,
    handlers::feeds::get_feeds,
    handlers::news::get_news,
    repositories::{feed_repository::FeedRepository, news_repository::NewsRepository},
};

#[tokio::main]
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

    let app = App::new(feed_repository.clone(), news_repository.clone());

    info!("Setting up cronjobs");

    if let Err(err) = setup_cronjobs(&app).await {
        panic!("failed setup cronjobs: {}", err);
    };

    let server_port = config.server_port.clone();

    info!("Starting API server in port {}", server_port.clone());

    let server_result = HttpServer::new(move || {
        setup_http_server(
            &config,
            feed_repository.clone(),
            news_repository.clone(),
            subscription_repository.clone(),
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
) -> ActixApp<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = ActixError,
    >,
> {
    let jwt_config: Arc<dyn JwtMiddlewareConfig> = Arc::new(config.clone());

    build_server(config.cors_origin.clone())
        .app_data(web::Data::from(feed_repo.clone()))
        .app_data(web::Data::from(news_repo.clone()))
        .app_data(web::Data::from(subscription_repo.clone()))
        .app_data(web::Data::new(config.clone()))
        .app_data(web::Data::from(jwt_config.clone()))
        .service(get_news)
        .service(get_feeds)
        .service(get_subscriptions)
        .service(create_subscription)
        .service(delete_subscription)
}
