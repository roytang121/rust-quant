use async_trait::async_trait;
use dashmap::DashMap;

use crate::model::constants::PublishChannel;
use crate::model::{OrderStatus, OrderUpdate};
use crate::pubsub::simple_message_bus::{MessageConsumer, RedisBackedMessageBus};

use std::sync::Arc;

type Cache = Arc<DashMap<String, OrderUpdate>>;

#[derive(Debug)]
pub struct OrderUpdateCache {
    pub cache: Cache,
}

impl OrderUpdateCache {
    pub fn new() -> OrderUpdateCache {
        OrderUpdateCache {
            cache: Arc::new(DashMap::new()),
        }
    }

    pub async fn subscribe(&self) -> anyhow::Result<()> {
        RedisBackedMessageBus::subscribe_channels(vec![PublishChannel::OrderUpdate.as_ref()], self)
            .await
    }

    pub fn accept_order_update(cache: Cache, mut order_update: OrderUpdate) {
        // let mut order_update = serde_json::from_slice::<OrderUpdate>(msg.as_slice()).unwrap();
        log::debug!("{:?}", order_update);
        if order_update.has_cache_key() {
            let cache_key = order_update.cache_key();
            match order_update.status {
                OrderStatus::Closed | OrderStatus::Failed => {
                    cache.remove(&cache_key);
                }
                _ => {
                    if cache.contains_key(cache_key.as_str()) {
                        let cached_order = cache
                            .get(cache_key.as_str())
                            .expect("Failed to get cached_order");
                        match cached_order.status {
                            OrderStatus::PendingCancel => match order_update.status {
                                OrderStatus::New | OrderStatus::Open => {
                                    order_update.status = OrderStatus::PendingCancel;
                                    drop(cached_order);
                                    cache.insert(cache_key, order_update);
                                    return ();
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                    cache.insert(cache_key, order_update);
                }
            }
        } else {
            log::info!(
                "Received order_update with empty clientId: {:?}",
                order_update
            );
        }
    }
}

#[async_trait]
impl MessageConsumer for OrderUpdateCache {
    async fn consume(&self, msg: &[u8]) -> anyhow::Result<()> {
        let order_update = serde_json::from_slice::<OrderUpdate>(msg)?;
        // log::debug!("{:?}", msg);
        let cache = self.cache.clone();
        tokio::spawn(async move { Self::accept_order_update(cache, order_update) });
        Ok(())
    }
}
