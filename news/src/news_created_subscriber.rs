use crate::{
    config::Config, models::news::News,
    repositories::subscription_repository::SubscriptionRepository,
};
use actix::Addr;
use log::{debug, error, info};
use rdkafka::consumer::Consumer;
use rdkafka::consumer::StreamConsumer;
use rdkafka::Message;
use std::{error::Error, sync::Arc};
use utils::{
    broker,
    http::websockets::ws_server::{SessionMessage, WebsocketServer},
};

pub async fn setup_news_created_subscriber(
    config: &Config,
    ws: Addr<WebsocketServer>,
    subscription_repo: Arc<dyn SubscriptionRepository>,
) -> Result<(), Box<dyn Error>> {
    let consumer: StreamConsumer = broker::create_consumer(config.kafka_url.clone());

    consumer
        .subscribe(&["news_created"])
        .expect("Error subscribing to topic");

    // Consume messages from the subscribed topic
    loop {
        match consumer.recv().await {
            Ok(message) => {
                let payload = message.payload_view::<str>();
                match payload {
                    Some(Ok(payload)) => {
                        info!("Received message: {}", payload);
                        let news: News =
                            serde_json::from_str(payload).expect("Failed to convert JSON string");

                        let result = subscription_repo.list_by_feed(news.feed_id);

                        match result {
                            Ok(subscriptions) => {
                                for s in subscriptions {
                                    debug!("sending message to socket {}", s.user_id);
                                    ws.do_send(SessionMessage {
                                        id: s.user_id.to_string(),
                                        message: payload.to_string(),
                                    });
                                }
                            }
                            Err(err) => {
                                error!(
                                    "failed getting subscriptions from database: {}",
                                    err.message
                                );
                            }
                        }
                    }
                    Some(Err(_)) => error!("Error deserializing message payload"),
                    None => error!("Empty message payload"),
                }
            }
            Err(_) => error!("Error deserializing message payload"),
        }
    }
}
