use std::sync::Arc;
use actix_web::web::to;
use futures::StreamExt;
use log::info;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::Message;
use crate::cache_data::data_cache::{GlobalCache};
use crate::cache_data::dto::{ InvalidCache};

pub struct MqConsumer {
    pub consumer: Arc<StreamConsumer>,
    pub cache_manager: Arc<GlobalCache>,
}

pub trait MqConsumerFunc {
    fn invalid_by_signal(&self, topic: &str);
}

impl MqConsumer {
    pub fn new(consumer: StreamConsumer, cache_manager: Arc<GlobalCache>) -> MqConsumer {
        MqConsumer {
            consumer: Arc::new(consumer),
            cache_manager,
        }
    }
}

impl MqConsumerFunc for MqConsumer {
    fn invalid_by_signal(&self, topic: &str) {
        self.consumer.subscribe(&[topic]).expect("cannot subscribe topic");

        let manager = self.cache_manager.clone();
        let consumer_clone = self.consumer.clone();

        tokio::spawn(async move {
            while let Some(message) = consumer_clone.stream().next().await {
                let rs = message.unwrap();
                let value = rs.payload().unwrap();

                let invalid: InvalidCache = serde_json::from_slice(value).expect("failed to deser JSON to User");
                info!("value from kafka {:?}", invalid);
                if invalid.id.is_some() {
                    let id = invalid.id.unwrap();
                    manager.invalid(id).await;
                }
            }
        });
    }
}
