use std::future::Future;
use crate::model::constants::Exchanges;
use async_trait::async_trait;

pub mod simple_message_bus;
pub(crate) mod market_depth_cache;

#[async_trait]
pub trait MessageBus {
    // async fn publish<T: Serialize>(&mut self, channel: &str, message: &T) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct MessageBusUtils {}
impl MessageBusUtils {
    pub async fn publish_async(sender: &tokio::sync::mpsc::Sender<PublishPayload>, payload: PublishPayload) -> Result<(), SendError<PublishPayload>> {
        sender.send(payload).await
    }
}

#[derive(Debug)]
pub struct PublishPayload {
    pub(crate) channel: String,
    pub(crate) payload: String,
}

pub struct SubscribeMarketDepthRequest {
    pub exchange: Exchanges,
    pub market: String,
}

pub use market_depth_cache::{MarketDepthCache};
use serde::Serialize;
use tokio::sync::mpsc::error::SendError;