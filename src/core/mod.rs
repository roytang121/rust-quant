use crate::pubsub::simple_message_bus::MessageBusSender;

pub mod config;

#[async_trait]
pub trait OrderGateway {
    async fn subscribe(&self) -> anyhow::Result<()>;
}
