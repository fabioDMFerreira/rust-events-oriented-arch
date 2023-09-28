use crate::{broker, error::BrokerError, news::events::NEWS_CREATED_EVENT};
use async_trait::async_trait;
use mockall::automock;
use rdkafka::producer::FutureProducer;

use crate::news::models::news::News;

#[automock]
#[async_trait]
pub trait EventService: Send + Sync {
    async fn news_created(&self, news: &News) -> Result<(), BrokerError>;
}

pub struct KafkaEventService {
    producer: FutureProducer,
}

impl KafkaEventService {
    pub fn new(producer: FutureProducer) -> Self {
        KafkaEventService { producer }
    }
}

#[async_trait]
impl EventService for KafkaEventService {
    async fn news_created(&self, news: &News) -> Result<(), BrokerError> {
        let json_string = serde_json::to_string(news).map_err(|err| BrokerError {
            message: err.to_string(),
        })?;

        broker::send_message_to_topic(self.producer.clone(), NEWS_CREATED_EVENT, json_string)
            .await
            .map_err(|err| BrokerError {
                message: err.0.to_string(),
            })?;

        Ok(())
    }
}
