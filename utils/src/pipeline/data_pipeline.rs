use log::error;

use super::consumer::Consumer;
use super::processor::Processor;

pub struct DataPipeline<'a> {
    consumer: &'a dyn Consumer,
    processor: &'a dyn Processor,
}

impl<'a> DataPipeline<'a> {
    pub fn new(consumer: &'a dyn Consumer, processor: &'a dyn Processor) -> Self {
        DataPipeline {
            consumer,
            processor,
        }
    }
}

impl<'a> DataPipeline<'a> {
    pub async fn start(&self) {
        loop {
            match self.consumer.consume().await {
                Ok(message) => {
                    if let Err(err) = self.processor.process(&message).await {
                        error!("failed processing message {}: {}", message, err);
                    }
                }
                Err(err) => error!("failed consuming message: {}", err),
            }
        }
    }
}
