use crate::model::constants::{Exchanges, PublishChannel};
use crate::model::market_data_model::MarketDepth;
use crate::pubsub::SubscribeMarketDepthRequest;
use dashmap::DashMap;
use redis::Msg;
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio_stream::StreamExt;

pub struct MarketDepthCache {
    pub cache: Arc<DashMap<String, MarketDepth>>,
    tx: tokio::sync::mpsc::Sender<String>,
    rx: tokio::sync::mpsc::Receiver<String>,
}

impl MarketDepthCache {
    pub fn new() -> MarketDepthCache {
        let (tx, rx) = tokio::sync::mpsc::channel::<String>(1000);
        MarketDepthCache {
            cache: Arc::new(DashMap::new()),
            tx: tx,
            rx: rx,
        }
    }

    pub async fn subscribe(
        &mut self,
        market_depth_requests: &[SubscribeMarketDepthRequest],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let redis = redis::Client::open("redis://localhost:10400").unwrap();
        let conn = redis.get_async_connection().await.unwrap();
        let mut pubsub = conn.into_pubsub();
        for request in market_depth_requests {
            pubsub
                .subscribe(format!(
                    "{}:{}:{}",
                    PublishChannel::MarketDepth.to_string(),
                    request.exchange.to_string(),
                    request.market
                ))
                .await
                .unwrap();
        }
        // let mut stream = Box::pin(pubsub.on_message().throttle(Duration::from_millis(50)));
        let mut stream = pubsub.on_message();
        let tx_clone = self.tx.clone();
        let stream_reader = async move {
            while let Some(msg) = stream.next().await {
                let m: Msg = msg;
                let value = m.get_payload::<String>().unwrap();
                tx_clone.send(value).await;
            }
        };
        let printer = async move {
            while let Some(mut msg) = self.rx.recv().await {
                let md = simd_json::from_str::<MarketDepth>(msg.as_mut_str()).unwrap();
                // log::info!("{:?}", md);
                self.cache.insert(md.market.to_owned(), md);
            }
        };

        tokio::select! {
            _ = stream_reader => {},
            _ = printer => {},
        }
        Ok(())
    }
}
