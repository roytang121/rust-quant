use crate::model::constants::Exchanges;
use async_trait::async_trait;
use std::future::Future;

pub mod simple_message_bus;
pub use std::str::FromStr;

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
impl SubscribeMarketDepthRequest {
    pub fn new(exchange: Exchanges, market: &str) -> SubscribeMarketDepthRequest {
        SubscribeMarketDepthRequest {
            exchange,
            market: market.to_string(),
        }
    }
    pub fn from_token(token: &str) -> SubscribeMarketDepthRequest {
        let splitted: Vec<&str> = token.split(".").collect();
        let market = splitted.get(0).expect("Invalid token.").to_owned();
        let exchange = splitted.get(1).expect("Invalid token.").to_owned();
        let exchange = Exchanges::from_str(exchange).expect("Unknown exchange");
        SubscribeMarketDepthRequest {
            exchange: exchange,
            market: market.to_string(),
        }
    }
}

use serde::Serialize;
use tokio::sync::mpsc::error::SendError;
