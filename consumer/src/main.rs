#[macro_use]
extern crate log;

use std::env;
use utils::broker;
use utils::logger::init_logger;

use rdkafka::consumer::StreamConsumer;
use rdkafka::message::{BorrowedMessage, Message};

#[tokio::main]
async fn main() {
    let kafka_url = env::var("KAFKA_URL").unwrap_or_else(|_| {
        panic!("KAFKA_URL environment variable is not set");
    });
    let logs_path = env::var("LOGS_PATH").unwrap_or_else(|_| String::from(""));

    init_logger(logs_path);

    let topic = "user_created";

    debug!(
        "consumer subscribing to topic {} in server {:?}",
        topic,
        kafka_url.as_str()
    );

    // Set up the Kafka consumer configuration
    let consumer: StreamConsumer = broker::create_consumer(kafka_url.clone());

    let processor = |msg: BorrowedMessage<'_>| match msg.payload_view::<str>() {
        Some(Ok(payload)) => info!("Received message: {}", payload),
        Some(Err(_)) => error!("Error deserializing message payload"),
        None => error!("Empty message payload"),
    };

    broker::subscribe(consumer, topic.to_string(), processor).await;
}
