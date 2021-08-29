use crate::model::constants::{Exchanges, PublishChannel};
use crate::model::{
    CancelOrderRequest, Instrument, OrderRequest, OrderStatus, OrderUpdate, OrderUpdateCache,
};
use crate::pubsub::simple_message_bus::{MessageConsumer, RedisBackedMessageBus};
use crate::pubsub::PublishPayload;
use async_trait::async_trait;
use dashmap::DashMap;
use futures_util::StreamExt;
use redis::Msg;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::mpsc::error::SendError;
use uuid::Uuid;

#[async_trait]
pub trait OrderGateway {
    fn new() -> Self;
    async fn subscribe(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct OrderUpdateService {
    pub cache: OrderUpdateCache,
}

impl OrderUpdateService {
    pub fn new() -> OrderUpdateService {
        OrderUpdateService {
            cache: Arc::new(DashMap::new()),
        }
    }

    pub async fn subscribe(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut sub =
            RedisBackedMessageBus::subscribe(vec![PublishChannel::OrderUpdate.as_ref()], self);
        sub.await
    }
}

#[async_trait]
impl MessageConsumer for OrderUpdateService {
    async fn consume(&self, msg: &mut str) -> Result<(), Box<dyn std::error::Error>> {
        let order_update = simd_json::from_str::<OrderUpdate>(msg)?;
        log::info!("{:?}", order_update);
        if order_update.has_cache_key() {
            let cache_key = order_update.cache_key();
            match order_update.status {
                OrderStatus::Closed => {
                    self.cache.remove(&cache_key);
                }
                _ => {
                    self.cache.insert(cache_key, order_update);
                }
            }
        } else {
            log::info!(
                "Received order_update with empty clientId: {:?}",
                order_update
            );
        }
        Ok(())
    }
}

impl OrderRequest {
    pub async fn send_order(
        order_update_cache: &OrderUpdateCache,
        message_bus_sender: &tokio::sync::mpsc::Sender<PublishPayload>,
        order_request: OrderRequest,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload = PublishPayload {
            channel: PublishChannel::OrderRequest.to_string(),
            payload: RedisBackedMessageBus::pack_json(&order_request)?,
        };
        let pending_order_update = OrderUpdate {
            exchange: order_request.exchange.clone(),
            id: -1,
            client_id: Option::from(order_request.client_id),
            market: order_request.market,
            type_: order_request.type_,
            side: order_request.side,
            size: order_request.size,
            price: order_request.price,
            reduceOnly: false,
            ioc: order_request.ioc,
            postOnly: order_request.post_only,
            status: OrderStatus::PendingNew,
            filledSize: 0.0,
            remainingSize: 0.0,
            avgFillPrice: None,
        };
        order_update_cache.insert(pending_order_update.cache_key(), pending_order_update);
        message_bus_sender.send(payload).await?;
        Ok(())
    }
    pub async fn cancel_order(
        order_update_cache: &OrderUpdateCache,
        message_bus_sender: &tokio::sync::mpsc::Sender<PublishPayload>,
        client_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut order_update) = order_update_cache.get_mut(client_id) {
            order_update.status = OrderStatus::PendingCancel
        }
        let cancel_order_request = CancelOrderRequest {
            exchange: Exchanges::FTX,
            client_id: client_id.to_string(),
        };
        let payload = PublishPayload {
            channel: PublishChannel::CancelOrder.to_string(),
            payload: RedisBackedMessageBus::pack_json(&cancel_order_request)?,
        };
        message_bus_sender.send(payload).await?;
        Ok(())
    }
}
