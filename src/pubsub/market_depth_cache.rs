use std::collections::HashMap;
use crate::model::market_data_model::MarketDepth;
use redis::Msg;
use std::time::Duration;
use std::cell::Cell;
use tokio_stream::StreamExt;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::model::constants::Exchanges;
use dashmap::DashMap;

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

    pub async fn subscribe(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let redis = redis::Client::open("redis://localhost:10400").unwrap();
        let conn = redis.get_async_connection().await.unwrap();
        let mut pubsub = conn.into_pubsub();
        pubsub.subscribe("marketdepth:FTX:ETH-PERP").await.unwrap();
        pubsub.subscribe("marketdepth:FTX:BTC-PERP").await.unwrap();
        pubsub.subscribe("marketdepth:FTX:BTC/USD").await.unwrap();
        pubsub.subscribe("marketdepth:FTX:ETH/USD").await.unwrap();
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