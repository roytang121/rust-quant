use crate::core::config::ConfigStore;
use crate::pubsub::{MessageBus, PublishPayload};
use async_trait::async_trait;
use futures_util::StreamExt;
use redis::Msg;
use serde::Serialize;

use tokio::sync::mpsc::error::SendError;

pub type MessageBusSender = tokio::sync::mpsc::Sender<PublishPayload>;

pub struct RedisBackedMessageBus {
    pub conn: redis::aio::Connection,
    pub publish_tx: tokio::sync::mpsc::Sender<PublishPayload>,
    publish_rx: tokio::sync::mpsc::Receiver<PublishPayload>,
}

impl MessageBus for RedisBackedMessageBus {}

impl RedisBackedMessageBus {
    pub async fn new() -> anyhow::Result<RedisBackedMessageBus> {
        let cfg = ConfigStore::load();
        let redis_client = redis::Client::open(cfg.redis_url)?;
        let conn = redis_client.get_async_connection().await?;
        let (tx, rx) = tokio::sync::mpsc::channel::<PublishPayload>(200);
        let instance = RedisBackedMessageBus {
            conn,
            publish_tx: tx,
            publish_rx: rx,
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

    pub async fn publish<T: Serialize>(
        &mut self,
        channel: &str,
        message: &T,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // let mut conn = &mut self.conn;
        let packed = Self::pack_json(message)?;
        redis::cmd("PUBLISH")
            .arg(channel)
            .arg(packed)
            .query_async::<redis::aio::Connection, i32>(&mut self.conn)
            .await?;
        Ok(())
    }

    pub async fn publish_async(
        sender: &tokio::sync::mpsc::Sender<PublishPayload>,
        payload: PublishPayload,
    ) -> Result<(), SendError<PublishPayload>> {
        sender.send(payload).await
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
            pubsub.subscribe(channel).await?
        }
        let mut stream = pubsub.on_message();
        while let Some(msg) = stream.next().await {
            let msg: Msg = msg;
            let mut payload = msg.get_payload::<String>()?;
            consumer.consume(payload.as_mut_str()).await?;
        }
        Err(anyhow!("subscribe_channels uncaught error"))
    }

    pub async fn subscribe(&mut self) -> anyhow::Result<()> {
        let rx = &mut self.publish_rx;
        while let Some(msg) = rx.recv().await {
            redis::cmd("PUBLISH")
                .arg(msg.channel.as_str())
                .arg(msg.payload.as_str())
                .query_async::<redis::aio::Connection, i32>(&mut self.conn)
                .await?;
        }
        Err(anyhow!("simple_message_bus uncaught error"))
    }
}

#[async_trait]
pub trait MessageConsumer {
    async fn consume(&self, msg: &mut str) -> anyhow::Result<()>;
}

#[async_trait]
pub trait TypedMessageConsumer<T> {
    async fn consume(&self, msg: T) -> anyhow::Result<()>;
}
