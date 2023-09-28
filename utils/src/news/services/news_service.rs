use crate::error::CommonError;
use async_trait::async_trait;
use log::{info, warn};
use mockall::automock;
use std::sync::Arc;

use crate::news::{
    models::{feed::Feed, news::News},
    repositories::{
        feed_repository::FeedRepository, news_repository::NewsRepository,
        subscription_repository::SubscriptionRepository,
    },
};

use super::events_service::EventService;

#[automock]
#[async_trait]
pub trait NewsService: Send + Sync {
    async fn list_feeds(&self) -> Result<Vec<Feed>, CommonError>;
    async fn insert_news(&self, news: &News) -> Result<News, CommonError>;
}

pub struct Service {
    pub feed_repo: Arc<dyn FeedRepository>,
    pub news_repo: Arc<dyn NewsRepository>,
    pub subscriptions_repo: Arc<dyn SubscriptionRepository>,
    pub events_service: Arc<dyn EventService>,
}

impl Service {
    pub fn new(
        feed_repo: Arc<dyn FeedRepository>,
        news_repo: Arc<dyn NewsRepository>,
        subscriptions_repo: Arc<dyn SubscriptionRepository>,
        events_service: Arc<dyn EventService>,
    ) -> Self {
        Service {
            feed_repo,
            news_repo,
            subscriptions_repo,
            events_service,
        }
    }
}

#[async_trait]
impl NewsService for Service {
    async fn insert_news(&self, news: &News) -> Result<News, CommonError> {
        let db_news = self
            .news_repo
            .find_by_fields(Some(news.title.clone()), Some(news.feed_id))?;

        return match db_news {
            None => {
                let news = self.news_repo.create(news)?;
                info!(
                    "News with title {} of feed {} inserted!",
                    news.title, news.feed_id
                );
                self.events_service.news_created(&news).await?;
                Ok(news)
            }
            Some(news) => {
                warn!("news already exists: {:?}", news);
                Ok(news)
            }
        };
    }

    async fn list_feeds(&self) -> Result<Vec<Feed>, CommonError> {
        self.feed_repo.list().map_err(|err| err.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        error::DATABASE_ERROR_CODE,
        news::{
            models::news::News,
            repositories::{
                feed_repository::MockFeedRepository, news_repository::MockNewsRepository,
                subscription_repository::MockSubscriptionRepository,
            },
            services::events_service::MockEventService,
        },
    };

    use super::*;
    use crate::error::DatabaseError;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_insert_news_success() {
        // Arrange
        let mut news_repo = MockNewsRepository::new();
        let mut events_service = MockEventService::new();
        let feeds_repo = MockFeedRepository::new();
        let subscriptions_repo = MockSubscriptionRepository::new();

        let news = News {
            title: "Test News".to_string(),
            feed_id: uuid::Uuid::new_v4(),
            id: uuid::Uuid::new_v4(),
            author: "author 1".to_string(),
            url: "".to_string(),
            publish_date: None,
        };

        let inserted_news = news.clone();
        news_repo
            .expect_find_by_fields()
            .with(eq(Some(news.title.clone())), eq(Some(news.feed_id)))
            .times(1)
            .returning(move |_, _| Ok(None));

        news_repo
            .expect_create()
            .with(eq(news.clone()))
            .times(1)
            .return_once(move |_| Ok(inserted_news.clone()));

        let inserted_news = news.clone();
        events_service
            .expect_news_created()
            .with(eq(inserted_news.clone()))
            .times(1)
            .returning(|_| Ok(()));

        let service = Service::new(
            Arc::new(feeds_repo),
            Arc::new(news_repo),
            Arc::new(subscriptions_repo),
            Arc::new(events_service),
        );

        // Act
        let result = service.insert_news(&news).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().title, "Test News");
    }

    #[tokio::test]
    async fn test_insert_news_already_exists() {
        // Arrange
        let mut news_repo = MockNewsRepository::new();
        let events_service = MockEventService::new();
        let feeds_repo = MockFeedRepository::new();
        let subscriptions_repo = MockSubscriptionRepository::new();

        let news = News {
            title: "Test News".to_string(),
            feed_id: uuid::Uuid::new_v4(),
            id: uuid::Uuid::new_v4(),
            author: "author 1".to_string(),
            url: "".to_string(),
            publish_date: None,
        };
        let news_cloned = news.clone();
        news_repo
            .expect_find_by_fields()
            .with(eq(Some(news.title.clone())), eq(Some(news.feed_id)))
            .times(1)
            .returning(move |_, _| Ok(Some(news_cloned.clone())));

        let service = Service::new(
            Arc::new(feeds_repo),
            Arc::new(news_repo),
            Arc::new(subscriptions_repo),
            Arc::new(events_service),
        );

        // Act
        let result = service.insert_news(&news).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_feeds_success() {
        // Arrange
        let mut feed_repo = MockFeedRepository::new();
        let expected_feeds = vec![
            Feed {
                id: uuid::Uuid::new_v4(),
                title: "Feed 1".to_string(),
                author: "author1".to_string(),
                url: "".to_string(),
            },
            Feed {
                id: uuid::Uuid::new_v4(),
                title: "Feed 2".to_string(),
                author: "author1".to_string(),
                url: "".to_string(),
            },
        ];
        let expected_feeds_cloned = expected_feeds.clone();
        feed_repo
            .expect_list()
            .times(1)
            .return_once(move || Ok(expected_feeds_cloned.clone()));

        let service = Service::new(
            Arc::new(feed_repo),
            Arc::new(MockNewsRepository::new()),
            Arc::new(MockSubscriptionRepository::new()),
            Arc::new(MockEventService::new()),
        );

        // Act
        let result = service.list_feeds().await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_feeds);
    }

    #[tokio::test]
    async fn test_list_feeds_error() {
        // Arrange
        let mut feed_repo = MockFeedRepository::new();
        let expected_error = DatabaseError {
            message: "Failed to list feeds".to_string(),
        };

        feed_repo
            .expect_list()
            .times(1)
            .return_once(move || Err(expected_error));

        let service = Service::new(
            Arc::new(feed_repo),
            Arc::new(MockNewsRepository::new()),
            Arc::new(MockSubscriptionRepository::new()),
            Arc::new(MockEventService::new()),
        );

        // Act
        let result = service.list_feeds().await;

        // Assert
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CommonError {
                message: "Failed to list feeds".to_string(),
                code: DATABASE_ERROR_CODE
            }
        );
    }
}
