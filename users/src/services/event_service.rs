use async_trait::async_trait;
use mockall::automock;
use rdkafka::producer::FutureProducer;
use utils::{broker, error::BrokerError};

use crate::models::user::User;

#[automock]
#[async_trait]
pub trait EventService: Send + Sync {
    async fn user_created(&self, user: User) -> Result<(), BrokerError>;
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
    async fn user_created(&self, user: User) -> Result<(), BrokerError> {
        let json_string = serde_json::to_string(&user).unwrap();

        let delivery_status = broker::send_message_to_topic(
            self.producer.clone(),
            String::from("user_created"),
            json_string,
        );

        match delivery_status.await {
            Err((err, _)) => {
                return Err(BrokerError {
                    message: err.to_string(),
                })
            }
            Ok(_) => return Ok(()),
        }
    }
}
