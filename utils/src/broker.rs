use std::time::Duration;

use rdkafka::consumer::Consumer;
use rdkafka::consumer::StreamConsumer;
use rdkafka::message::BorrowedMessage;
use rdkafka::message::OwnedHeaders;
use rdkafka::{
    error::KafkaError,
    message::OwnedMessage,
    producer::{FutureProducer, FutureRecord},
    ClientConfig,
};

pub type KafkaProducer = FutureProducer;

pub fn create_producer(kafka_url: String) -> KafkaProducer {
    ClientConfig::new()
        .set("bootstrap.servers", kafka_url)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error")
}

pub async fn send_message_to_topic(
    producer: KafkaProducer,
    topic: String,
    message: String,
) -> Result<(i32, i64), (KafkaError, OwnedMessage)> {
    producer
        .send(
            FutureRecord::to(topic.as_str())
                .payload(&message)
                .key(topic.as_str())
                .headers(OwnedHeaders::new()),
            Duration::from_secs(120),
        )
        .await
}

pub fn create_consumer(kafka_url: String) -> StreamConsumer {
    ClientConfig::new()
        .set("bootstrap.servers", kafka_url)
        .set("group.id", "<your-consumer-group-id>")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Consumer creation error")
}

pub async fn subscribe(
    consumer: StreamConsumer,
    topic: String,
    processor: fn(BorrowedMessage<'_>),
) {
    // Subscribe to a topic
    consumer
        .subscribe(&[topic.as_str()])
        .expect("Error subscribing to topic");

    // Consume messages from the subscribed topic
    loop {
        match consumer.recv().await {
            Ok(message) => processor(message),
            Err(e) => println!("Error receiving message: {:?}", e),
        }
    }
}
