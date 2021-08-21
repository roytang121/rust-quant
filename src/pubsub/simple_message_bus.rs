use crate::pubsub::MessageBus;
use serde::Serialize;

pub struct RedisBackedMessageBus {
    conn: redis::aio::Connection,
}

impl MessageBus for RedisBackedMessageBus {}

impl RedisBackedMessageBus {
    pub async fn new() -> Result<RedisBackedMessageBus, Box<dyn std::error::Error>> {
        let redis_client = redis::Client::open("redis://localhost:10400")?;
        let conn = redis_client.get_async_connection().await?;
        let instance = RedisBackedMessageBus { conn: conn };
        Ok(instance)
    }

    fn pack_value<T: Serialize>(value: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buf = Vec::new();
        value.serialize(&mut serde_json::Serializer::new(&mut buf))?;
        Ok(buf)
    }

    fn pack_json<T: Serialize>(value: &T) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string(&value)?)
    }

    pub async fn publish<T: Serialize>(
        &mut self,
        channel: &str,
        message: &T,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let conn = &mut self.conn;
        let packed = Self::pack_json(message)?;
        redis::cmd("PUBLISH")
            .arg(channel)
            .arg(packed)
            .query_async::<redis::aio::Connection, i32>(conn)
            .await?;
        Ok(())
    }
}
