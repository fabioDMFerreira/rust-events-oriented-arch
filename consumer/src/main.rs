#[macro_use]
extern crate log;

use chrono::Local;
use std::env;
use std::fs::File;
use std::io::Write;

use env_logger::Env;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;

#[tokio::main]
async fn main() {
    let target = Box::new(File::create("/var/log/app/stdout.log").expect("Can't create file"));

    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .target(env_logger::Target::Pipe(target))
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {} {}:{} {}",
                Local::now().format("%b %d %H:%M:%S"),
                record.level(),
                record.file_static().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .init();

    let kafka_url = env::var("KAFKA_URL").unwrap_or_else(|_| {
        panic!("KAFKA_URL environment variable is not set");
    });

    let topic = "user_created";

    debug!(
        "consumer subscribing to topic {} in server {:?}",
        topic,
        kafka_url.as_str()
    );

    // Set up the Kafka consumer configuration
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", kafka_url)
        .set("group.id", "<your-consumer-group-id>")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Consumer creation error");

    // Subscribe to a topic
    consumer
        .subscribe(&[topic])
        .expect("Error subscribing to topic");

    debug!("subscribed {}", topic);

    // Consume messages from the subscribed topic
    loop {
        match consumer.recv().await {
            Ok(message) => match message.payload_view::<str>() {
                Some(Ok(payload)) => info!("Received message: {}", payload),
                Some(Err(_)) => error!("Error deserializing message payload"),
                None => error!("Empty message payload"),
            },
            Err(e) => error!("Error receiving message: {:?}", e),
        }
    }
}
