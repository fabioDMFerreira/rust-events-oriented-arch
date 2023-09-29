use async_trait::async_trait;
use log::{debug, info};
use std::sync::Arc;
use utils::{
    error::{CommonError, DatabaseError, SerializationError},
    http::websockets::{ws_sender::WebsocketServerSender, ws_server::SessionMessage},
    news::{models::news::News, repositories::subscription_repository::SubscriptionRepository},
    pipeline::processor::Processor,
};

pub struct NewsWebsocketProcessor<'a> {
    websocket_server: &'a dyn WebsocketServerSender,
    subscription_repo: Arc<dyn SubscriptionRepository>,
}

impl<'a> NewsWebsocketProcessor<'a> {
    pub fn new(
        websocket_server: &'a dyn WebsocketServerSender,
        subscription_repo: Arc<dyn SubscriptionRepository>,
    ) -> Self {
        NewsWebsocketProcessor {
            websocket_server,
            subscription_repo,
        }
    }
}

#[async_trait]
impl<'a> Processor for NewsWebsocketProcessor<'a> {
    async fn process(&self, payload: &str) -> Result<(), CommonError> {
        info!("Received message: {}", payload);
        let news: News = serde_json::from_str(payload).map_err(|err| {
            SerializationError::new(format!("failed to convert JSON string: {}", err).as_str())
        })?;

        match self.subscription_repo.list_by_feed(news.feed_id) {
            Ok(subscriptions) => {
                for s in subscriptions {
                    debug!("sending message to socket {}", s.user_id);
                    // TODO: send messages concurrently
                    self.websocket_server
                        .do_send(SessionMessage {
                            id: s.user_id.to_string(),
                            message: payload.to_string(),
                        })
                        .await?;
                }

                Ok(())
            }
            Err(err) => Err(DatabaseError::new(
                format!(
                    "failed getting subscriptions from database: {}",
                    err.message
                )
                .as_str(),
            )
            .into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use std::str::FromStr;
    use utils::{
        http::websockets::ws_sender::MockWebsocketServerSender,
        news::models::subscription::Subscription,
        news::repositories::subscription_repository::MockSubscriptionRepository,
    };

    #[tokio::test]
    async fn test_process_success() {
        // Create mock objects
        let mut mock_websocket_server = MockWebsocketServerSender::new();
        let mut mock_subscription_repo = MockSubscriptionRepository::new();

        // Define expected inputs and outputs
        let payload = r#"{"id":"f130494f-711e-4bb3-940a-3d50bb65e521","author":"author1","url":"","title":"news test","feed_id":"63a0ae94-1ad8-45fd-acc6-9c68f58e28af","publish_date":"2023-05-13"}"#;
        let subscriptions = vec![
            Subscription {
                feed_id: uuid::Uuid::from_str("63a0ae94-1ad8-45fd-acc6-9c68f58e28af").unwrap(),
                user_id: uuid::Uuid::from_str("9454decf-b36d-436e-96e1-f31a9a2f3d68").unwrap(),
            },
            Subscription {
                feed_id: uuid::Uuid::from_str("63a0ae94-1ad8-45fd-acc6-9c68f58e28af").unwrap(),
                user_id: uuid::Uuid::from_str("9537e337-241e-4d3c-8776-b43fc1050010").unwrap(),
            },
        ];

        // Set up expectations
        mock_websocket_server
            .expect_do_send()
            .times(2)
            .withf(move |message| {
                message.id == "9454decf-b36d-436e-96e1-f31a9a2f3d68"
                    || message.id == "9537e337-241e-4d3c-8776-b43fc1050010"
            })
            .return_const(Ok(()));
        mock_subscription_repo
            .expect_list_by_feed()
            .with(eq(uuid::Uuid::from_str(
                "63a0ae94-1ad8-45fd-acc6-9c68f58e28af",
            )
            .unwrap()))
            .return_const(Ok(subscriptions));

        // Create the processor instance
        let processor =
            NewsWebsocketProcessor::new(&mock_websocket_server, Arc::new(mock_subscription_repo));

        // Perform the test
        let result = processor.process(payload).await;

        // Check the result
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_failed_deserialization() {
        // Create mock objects
        let mock_websocket_server = MockWebsocketServerSender::new();
        let mock_subscription_repo = MockSubscriptionRepository::new();

        // Define expected input
        let payload = "invalid json";

        // Create the processor instance
        let processor =
            NewsWebsocketProcessor::new(&mock_websocket_server, Arc::new(mock_subscription_repo));

        // Perform the test
        let result = processor.process(payload).await;

        // Check the result
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Error: failed to convert JSON string: expected value at line 1 column 1, Code: 6"
        );
    }

    #[tokio::test]
    async fn test_process_failed_database() {
        // Create mock objects
        let mut mock_websocket_server = MockWebsocketServerSender::new();
        let mut mock_subscription_repo = MockSubscriptionRepository::new();

        // Define expected inputs and outputs
        let payload = r#"{"id":"f130494f-711e-4bb3-940a-3d50bb65e521","author":"author1","url":"","title":"news test","feed_id":"63a0ae94-1ad8-45fd-acc6-9c68f58e28af","publish_date":"2023-05-13"}"#;

        let error_message = "Failed to fetch subscriptions";

        // Set up expectations
        mock_websocket_server.expect_do_send().times(0); // No calls to do_send() expected
        mock_subscription_repo
            .expect_list_by_feed()
            .with(eq(uuid::Uuid::from_str(
                "63a0ae94-1ad8-45fd-acc6-9c68f58e28af",
            )
            .unwrap()))
            .return_const(Err(DatabaseError::new(error_message)));

        // Create the processor instance
        let processor =
            NewsWebsocketProcessor::new(&mock_websocket_server, Arc::new(mock_subscription_repo));

        // Perform the test
        let result = processor.process(payload).await;

        // Check the result
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            format!(
                "Error: failed getting subscriptions from database: {}, Code: 1",
                error_message
            ),
        );
    }
}
