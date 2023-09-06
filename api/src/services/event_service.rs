use async_trait::async_trait;
use rdkafka::producer::FutureProducer;
use std::error::Error;
use utils::broker;

use crate::models::user::User;

#[async_trait]
pub trait EventService: Send + Sync {
    async fn user_created(&self, user: User) -> Result<(), Box<dyn Error>>;
}

pub struct KafkaEventService {
    producer: FutureProducer,
}

impl KafkaEventService {
    pub fn new(producer: FutureProducer) -> Self {
        KafkaEventService { producer: producer }
    }
}

#[async_trait]
impl EventService for KafkaEventService {
    async fn user_created(&self, user: User) -> Result<(), Box<dyn Error>> {
        let json_string = serde_json::to_string(&user).unwrap();

        let delivery_status = broker::send_message_to_topic(
            self.producer.clone(),
            String::from("user_created"),
            json_string,
        );

        match delivery_status.await {
            Err((err, _)) => return Err(Box::new(err)),
            Ok(_) => return Ok(()),
        }
    }
}
