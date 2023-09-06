use async_trait::async_trait;
use rdkafka::config::ClientConfig;
use rdkafka::message::{Header, OwnedHeaders};
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::error::Error;
use std::time::Duration;

use crate::models::user::User;

#[async_trait]
pub trait EventService: Send + Sync {
    async fn user_created(&self, user: User) -> Result<(), Box<dyn Error>>;
}

pub struct KafkaEventService {
    producer: FutureProducer,
}

impl KafkaEventService {
    pub fn new(kafka_url: String) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", kafka_url)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");

        KafkaEventService { producer: producer }
    }
}

#[async_trait]
impl EventService for KafkaEventService {
    async fn user_created(&self, user: User) -> Result<(), Box<dyn Error>> {
        let json_string = serde_json::to_string(&user).unwrap();

        // The send operation on the topic returns a future, which will be
        // completed once the result or failure from Kafka is received.
        let delivery_status = self
            .producer
            .send(
                FutureRecord::to("user_created")
                    .payload(&format!("{}", json_string))
                    .key(&format!("user created"))
                    .headers(OwnedHeaders::new().insert(Header {
                        key: "header_key",
                        value: Some("header_value"),
                    })),
                Duration::from_secs(120),
            )
            .await;

        // This loop will wait until all delivery statuses have been received.
        println!("Future completed. Result: {:?}", delivery_status);
        match delivery_status {
            Err((err, _)) => return Err(Box::new(err)),
            Ok(_) => return Ok(()),
        }
    }
}
