use crate::core::config::ConfigStore;
use crate::pubsub::{MessageBus, PublishPayload};
use async_trait::async_trait;
use futures_util::StreamExt;
use redis::{AsyncCommands, Msg, RedisError};
use serde::Serialize;

use std::sync::Arc;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::RwLock;

pub type MessageBusSender = tokio::sync::mpsc::Sender<PublishPayload>;

pub struct RedisBackedMessageBus {
    pub client: Arc<redis::Client>,
    pub publish_conn: redis::aio::MultiplexedConnection,
    pub publish_tx: tokio::sync::mpsc::Sender<PublishPayload>,
    publish_rx: RwLock<tokio::sync::mpsc::Receiver<PublishPayload>>,
}

impl MessageBus for RedisBackedMessageBus {}

impl RedisBackedMessageBus {
    pub async fn new() -> anyhow::Result<RedisBackedMessageBus> {
        let cfg = ConfigStore::load();
        let redis_client = redis::Client::open(cfg.redis_url)?;
        let publish_conn = redis_client.get_multiplexed_async_connection().await?;
        let (tx, rx) = tokio::sync::mpsc::channel::<PublishPayload>(200);
        let instance = RedisBackedMessageBus {
            client: Arc::new(redis_client),
            publish_conn,
            publish_tx: tx,
            publish_rx: RwLock::new(rx),
        };
        Ok(instance)
    }

    fn pack_value<T: Serialize>(value: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buf = Vec::new();
        value.serialize(&mut serde_json::Serializer::new(&mut buf))?;
        Ok(buf)
    }

    pub fn pack_json<T: Serialize>(value: &T) -> anyhow::Result<String> {
        Ok(serde_json::to_string(&value)?)
    }

    pub async fn publish<T: Serialize>(&self, channel: &str, message: &T) -> anyhow::Result<()> {
        let packed = Self::pack_json(message)?;
        let mut conn = self.publish_conn.clone();
        conn.publish::<&str, &str, i32>(channel.clone(), packed.as_str())
            .await?;
        Ok(())
    }

    /// spawning new tokio task for each publish message
    pub fn publish_spawn<T: 'static + Serialize + Send + Sync>(
        &self,
        channel: String,
        message: T,
    ) -> Result<(), SendError<PublishPayload>> {
        // sender.send(payload).await
        let mut conn = self.publish_conn.clone();
        tokio::spawn(async move {
            let packed = Self::pack_json(&message).unwrap();
            let result = conn
                .publish::<&str, &str, i32>(channel.as_str(), packed.as_str())
                .await;
            match result {
                Ok(_) => {}
                Err(err) => error!("redis publish error: {}", err),
            }
        });
        Ok(())
    }

    pub async fn subscribe_channels<T>(channels: Vec<&str>, consumer: &T) -> anyhow::Result<()>
    where
        T: MessageConsumer,
    {
        let cfg = ConfigStore::load();
        let redis_client = redis::Client::open(cfg.redis_url)?;
        let conn = redis_client.get_async_connection().await?;
        let mut pubsub = conn.into_pubsub();
        for channel in channels {
            log::info!("subscribing channel {}", channel);
            pubsub.subscribe(channel).await?
        }
        let mut stream = pubsub.on_message();
        while let Some(msg) = stream.next().await {
            let msg: Msg = msg;
            let mut payload = msg.get_payload_bytes();
            consumer.consume(payload).await?;
        }
        Err(anyhow!("subscribe_channels uncaught error"))
    }

    pub async fn subscribe(&self) -> anyhow::Result<()> {
        log::info!("redis_message_bus subscribing...");
        let mut rx = self.publish_rx.write().await;
        let mut conn = self.publish_conn.clone();
        while let Some(msg) = rx.recv().await {
            // let time_start = chrono::Utc::now().timestamp_nanos();
            conn.publish::<&str, &str, i32>(msg.channel.as_str(), msg.payload.as_str())
                .await;
            // let time_end = chrono::Utc::now().timestamp_nanos();
            // log::info!("INFO: {}ms", (time_end - time_start) as f64 * 0.000001);
        }
        Err(anyhow!("simple_message_bus uncaught error"))
    }
}

#[async_trait]
pub trait MessageConsumer {
    async fn consume(&self, msg: &[u8]) -> anyhow::Result<()>;
}

#[async_trait]
pub trait TypedMessageConsumer<T> {
    async fn consume(&self, msg: T) -> anyhow::Result<()>;
}
