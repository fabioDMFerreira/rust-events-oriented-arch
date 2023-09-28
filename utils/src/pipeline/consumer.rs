use async_trait::async_trait;
use rdkafka::consumer::StreamConsumer;
use rdkafka::Message;

use crate::error::{BrokerError, CommonError};

#[async_trait]
pub trait Consumer: Send + Sync {
    async fn consume(&self) -> Result<String, CommonError>;
}

pub struct KafkaConsumer {
    consumer: StreamConsumer,
}

impl KafkaConsumer {
    pub fn new(consumer: StreamConsumer) -> Self {
        KafkaConsumer { consumer }
    }
}

#[async_trait]
impl Consumer for KafkaConsumer {
    async fn consume(&self) -> Result<String, CommonError> {
        return match self.consumer.recv().await {
            Ok(message) => match message.payload_view::<str>() {
                Some(Ok(payload)) => Ok(payload.to_string()),
                Some(Err(err)) => Err(BrokerError {
                    message: format!("Error deserializing message payload: {}", err),
                }
                .into()),
                None => Err(BrokerError {
                    message: "Empty message payload".to_string(),
                }
                .into()),
            },
            Err(_) => Err(BrokerError {
                message: "Error deserializing message payload".to_string(),
            }
            .into()),
        };
    }
}
