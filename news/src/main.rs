extern crate log;

use news::{
    app::App,
    config::Config,
    repositories::{feed_repository::FeedRepository, news_repository::NewsRepository},
};
use std::{error::Error, sync::Arc, thread};
use tokio_cron_scheduler::{Job, JobScheduler};
use utils::{db::connect_db, logger::init_logger};

#[tokio::main]
async fn main() {
    let config = Config::init();

    init_logger(config.logs_path.clone());

    let db_pool = connect_db(config.database_url);

    let feed_repository = Arc::new(FeedRepository::new(Arc::new(db_pool.clone())));
    let news_repository = Arc::new(NewsRepository::new(Arc::new(db_pool.clone())));

    let app = App::new(feed_repository, news_repository);

    if let Err(err) = setup_cronjobs(&app).await {
        panic!("failed setup cronjobs: {}", err);
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
