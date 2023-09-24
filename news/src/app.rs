use std::sync::Arc;

use crate::{
    models::news::News,
    repositories::{feed_repository::FeedRepository, news_repository::NewsRepository},
    scrapper::Scrapper,
};
use log::{debug, error, info};
use tokio::{sync::mpsc, task};
use utils::{
    broker::{self, KafkaProducer},
    error::CommonError,
};

#[derive(Clone)]
pub struct App {
    pub feed_repo: Arc<dyn FeedRepository>,
    pub news_repo: Arc<dyn NewsRepository>,
    pub kafka_producer: KafkaProducer,
}

impl App {
    pub fn new(
        feed_repo: Arc<dyn FeedRepository>,
        news_repo: Arc<dyn NewsRepository>,
        kafka_producer: KafkaProducer,
    ) -> App {
        App {
            feed_repo,
            news_repo,
            kafka_producer,
        }
    }

    pub async fn scrap_feeds(&self) {
        debug!("start scrapping feeds");
        let result = self.feed_repo.list();

        let feeds = result.unwrap();

        const BUFFER_SIZE: usize = 10;
        let (tx, mut rx) = mpsc::channel::<Vec<News>>(BUFFER_SIZE);

        task::spawn(async move {
            let result = Scrapper::scrap_all(feeds, tx).await;
            if let Err(err) = result {
                error!("failed scrapping feeds: {}", err);
            }
        });

        while let Some(news) = rx.recv().await {
            for news in news {
                let db_news = self
                    .news_repo
                    .find_by_fields(Some(news.title.clone()), Some(news.feed_id));

                if let Ok(None) = db_news {
                    let result = self.news_repo.create(&news);
                    match result {
                        Ok(news) => {
                            info!(
                                "News with title {} of feed {} inserted!",
                                news.title, news.feed_id
                            );
                            let _ = broker::send_message_to_topic(
                                self.kafka_producer.clone(),
                                "news_created".to_string(),
                                serde_json::to_string(&news).unwrap(),
                            )
                            .await;
                        }
                        Err(err) => {
                            error!("failed creating new {:?}: {}", news, CommonError::from(err));
                        }
                    }
                }
            }
        }
    }
}
