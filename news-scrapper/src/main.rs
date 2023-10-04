use news_scrapper::{
    config::Config,
    http_fetcher::HttpFetcher,
    news_ingestor::NewsIngestor,
    scrapper::{RssFetcher, RssScrapper},
};
use std::{error::Error, sync::Arc, thread};
use tokio_cron_scheduler::{Job, JobScheduler};
use utils::{
    broker,
    db::connect_db,
    logger::init_logger,
    news::{
        repositories::{
            feed_repository::{FeedDieselRepository, FeedRepository},
            news_repository::{NewsDieselRepository, NewsRepository},
            subscription_repository::{SubscriptionRepository, SubscriptionsDieselRepository},
        },
        services::{
            events_service::{EventService, KafkaEventService},
            news_service::{NewsService, Service},
        },
    },
};

#[tokio::main]
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
    let events_service: Arc<dyn EventService> = Arc::new(KafkaEventService::new(kafka_producer));

    let service: Arc<dyn NewsService> = Arc::new(Service::new(
        feed_repository.clone(),
        news_repository.clone(),
        subscription_repository.clone(),
        events_service.clone(),
    ));

    let rss_fetcher: Arc<dyn RssFetcher> = Arc::new(HttpFetcher::default());

    let feeds_scrapper = Arc::new(RssScrapper::new(rss_fetcher.clone()));

    let ingestor = NewsIngestor::new(service, feeds_scrapper);

    if let Err(err) = setup_cronjobs(&ingestor).await {
        panic!("failed setup cronjobs: {}", err);
    };

    thread::park();
}

async fn setup_cronjobs(ingestor: &NewsIngestor) -> Result<(), Box<dyn Error>> {
    let ingestor = ingestor.clone();

    let sched = JobScheduler::new().await?;

    let scrap_news_job = Job::new_async("0 * * * * *", move |_uuid, _l| {
        let ingestor = ingestor.clone();
        Box::pin(async move {
            ingestor.ingest().await;
        })
    })?;
    sched.add(scrap_news_job).await?;

    sched.start().await?;

    Ok(())
}
