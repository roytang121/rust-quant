use crate::model::constants::{Exchanges, PublishChannel};
use crate::model::market_data_model::MarketDepth;
use crate::pubsub::simple_message_bus::{MessageConsumer, RedisBackedMessageBus};
use crate::pubsub::SubscribeMarketDepthRequest;
use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use redis::Msg;
use std::cell::Cell;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use std::borrow::Cow;
use rocket::http::ext::IntoOwned;
use std::ops::Deref;

type Cache = DashMap<String, MarketDepth>;

pub struct MarketDepthCache {
    pub cache: Arc<Cache>,
    tx: tokio::sync::mpsc::Sender<String>,
    rx: tokio::sync::mpsc::Receiver<String>,
}

impl MarketDepthCache {
    pub fn new() -> MarketDepthCache {
        let (tx, rx) = tokio::sync::mpsc::channel::<String>(1000);
        MarketDepthCache {
            cache: Arc::new(DashMap::new()),
            tx,
            rx,
        }
    }

    /// get a clone of MarketDepth and immediate releasing the ref
    /// it is costly in terms of memory allocation to clone a MarketDepth but yields a better performance for not locking a reference to Map
    pub fn get(&self, key: &str) -> Option<MarketDepth> {
        return match self.cache.get(key) {
            None => None,
            Some(md) => {
                let now = chrono::Utc::now().timestamp_millis();
                if now - md.timestamp > 1000 {
                    // ref must be dropped before calling remove to prevent deadlock
                    drop(md);
                    self.cache.remove(key);
                    return None;
                }
                Some(md.value().clone())
            }
        };
    }

    pub async fn subscribe(&self, market_depth_requests: &[SubscribeMarketDepthRequest]) -> Result<(), Box<dyn Error>> {
        let channels: Vec<String> = market_depth_requests
            .iter()
            .map(|request| {
                format!(
                    "{}:{}:{}",
                    PublishChannel::MarketDepth.to_string(),
                    request.exchange.to_string(),
                    request.market
                )
            })
            .collect();

        let channels = channels.as_slice().iter().map(AsRef::as_ref).collect();

        RedisBackedMessageBus::subscribe(channels, self).await
    }
}

#[async_trait::async_trait]
impl MessageConsumer for MarketDepthCache {
    async fn consume(&self, msg: &mut str) -> Result<(), Box<dyn Error>> {
        match serde_json::from_str::<MarketDepth>(msg) {
            Ok(md) => {
                self.cache.insert(md.market.to_string(), md);
            }
            Err(err) => {
                error!("{}", err);
            }
        }
        Ok(())
    }
}
