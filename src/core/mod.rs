use crate::pubsub::simple_message_bus::MessageBusSender;

pub mod config;

#[async_trait]
pub trait OrderGateway {
    fn new(message_bus_sender: MessageBusSender) -> Self;
    async fn subscribe(&self) -> Result<(), Box<dyn std::error::Error>>;
}
