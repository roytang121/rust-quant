use crate::model::constants::Exchanges;
use async_trait::async_trait;
use std::future::Future;

pub mod simple_message_bus;

#[async_trait]
pub trait MessageBus {
    // async fn publish<T: Serialize>(&mut self, channel: &str, message: &T) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct MessageBusUtils {}
impl MessageBusUtils {
    pub async fn publish_async(
        sender: &tokio::sync::mpsc::Sender<PublishPayload>,
        payload: PublishPayload,
    ) -> Result<(), SendError<PublishPayload>> {
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

use serde::Serialize;
use tokio::sync::mpsc::error::SendError;
