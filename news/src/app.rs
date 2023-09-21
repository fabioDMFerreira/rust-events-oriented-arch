use std::sync::Arc;

use crate::{
    models::news::News,
    repositories::{feed_repository::FeedRepository, news_repository::NewsRepository},
    scrapper::Scrapper,
};
use log::{debug, error, info};
use tokio::{sync::mpsc, task};

#[derive(Clone)]
pub struct App {
    pub feed_repo: Arc<FeedRepository>,
    pub news_repo: Arc<NewsRepository>,
}

impl App {
    pub fn new(feed_repo: Arc<FeedRepository>, news_repo: Arc<NewsRepository>) -> App {
        App {
            feed_repo,
            news_repo,
        }
    }

    pub async fn scrap_feeds(&self) {
        debug!("start scrapping feeds");
        let result = self.feed_repo.list();

        let feeds = result.unwrap();

        const BUFFER_SIZE: usize = 10;
        let (tx, mut rx) = mpsc::channel::<Vec<News>>(BUFFER_SIZE);

        task::spawn(async move {
            let mut scrapper = Scrapper::new(feeds);
            let result = scrapper.scrap_all(tx).await;
            if let Err(err) = result {
                error!("failed scrapping feeds: {}", err);
            }
        });

        while let Some(news) = rx.recv().await {
            for news in news {
                let db_news = self
                    .news_repo
                    .find_by_fields(Some(news.title.clone()), Some(news.feed_id.clone()));

                if let Ok(db_news) = db_news {
                    if let None = db_news {
                        let result = self.news_repo.create(&news);
                        if let Err(err) = result {
                            error!("failed creating new {:?}: {}", news, err);
                        } else {
                            info!(
                                "News with title {} of feed {} inserted!",
                                news.title, news.feed_id
                            );
                        }
                    }
                }
            }
        }
    }
}
