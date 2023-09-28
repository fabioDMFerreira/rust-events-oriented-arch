use std::sync::Arc;

use log::{debug, error};
use tokio::{sync::mpsc, task};
use utils::news::{models::news::News, services::news_service::NewsService};

use crate::scrapper::FeedsScrapper;

#[derive(Clone)]
pub struct NewsIngestor {
    pub news_service: Arc<dyn NewsService>,
    pub feeds_scrapper: Arc<dyn FeedsScrapper>,
}

impl NewsIngestor {
    pub fn new(
        news_service: Arc<dyn NewsService>,
        feeds_scrapper: Arc<dyn FeedsScrapper>,
    ) -> NewsIngestor {
        NewsIngestor {
            news_service,
            feeds_scrapper,
        }
    }

    pub async fn ingest(&self) {
        debug!("start scrapping feeds");
        let result = self.news_service.list_feeds().await;

        let feeds = match result {
            Ok(feeds) => feeds,
            Err(err) => {
                log::error!("failed to list feeds: {}", err);
                return;
            }
        };

        const BUFFER_SIZE: usize = 10;
        let (tx, mut rx) = mpsc::channel::<Vec<News>>(BUFFER_SIZE);

        let feeds_scrapper = self.feeds_scrapper.clone();

        task::spawn(async move {
            let result = feeds_scrapper.scrap_all(feeds, tx).await;
            if let Err(err) = result {
                error!("failed scrapping feeds: {}", err);
            }
        });

        while let Some(news) = rx.recv().await {
            for news_item in news {
                if let Err(err) = self.news_service.insert_news(&news_item).await {
                    error!("failed inserting news: {}", err);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use utils::{
        error::{CommonError, DATABASE_ERROR_CODE},
        news::{models::feed::Feed, services::news_service::MockNewsService},
    };

    use crate::scrapper::MockFeedsScrapper;

    use super::*;

    #[tokio::test]
    async fn test_news_ingestor_ingest_success() {
        let mut news_service = MockNewsService::new();
        let mut feeds_scrapper = MockFeedsScrapper::new();

        // Set up the mock behaviors for the NewsService and FeedsScrapper
        news_service.expect_list_feeds().returning(|| {
            Ok(vec![Feed {
                author: "coingraph".to_string(),
                title: "Coingraph".to_string(),
                url: "".to_string(),
                id: uuid::Uuid::new_v4(),
            }])
        });
        feeds_scrapper.expect_scrap_all().returning(|_, _| Ok(()));

        let news_service = Arc::new(news_service);
        let feeds_scrapper = Arc::new(feeds_scrapper);

        let news_ingestor = NewsIngestor::new(news_service, feeds_scrapper);
        news_ingestor.ingest().await;
    }

    #[tokio::test]
    async fn test_news_ingestor_ingest_list_feeds_error() {
        let mut news_service = MockNewsService::new();
        let feeds_scrapper = Arc::new(MockFeedsScrapper::new());

        // Set up the mock behavior for the NewsService when list_feeds fails
        news_service.expect_list_feeds().returning(|| {
            Err(CommonError {
                message: "db is down".to_string(),
                code: DATABASE_ERROR_CODE,
            })
        });

        let news_service = Arc::new(news_service);

        let news_ingestor = NewsIngestor::new(news_service, feeds_scrapper);
        news_ingestor.ingest().await;
    }
}
