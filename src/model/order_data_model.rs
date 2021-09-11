use crate::model::constants::{Exchanges, PublishChannel};

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use crate::pubsub::simple_message_bus::RedisBackedMessageBus;
use crate::pubsub::PublishPayload;
use uuid::Uuid;

pub type OrderCacheKey = String;
pub type OrderUpdateCacheInner = DashMap<OrderCacheKey, OrderUpdate>;

#[derive(Deserialize, Serialize, Debug, strum_macros::Display, Clone, PartialOrd, PartialEq)]
pub enum OrderType {
    Limit,
    Market,
}
#[derive(Deserialize, Serialize, Debug, strum_macros::Display, Clone, PartialOrd, PartialEq)]
pub enum OrderStatus {
    New,
    Open,
    Closed,
    Failed,
    PendingNew,
    PendingCancel,
}
#[derive(Deserialize, Serialize, Debug, strum_macros::Display, Clone, PartialOrd, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}
impl OrderSide {
    pub fn flip_side(side: &OrderSide) -> OrderSide {
        match side {
            OrderSide::Buy => OrderSide::Sell,
            OrderSide::Sell => OrderSide::Buy,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[allow(non_camel_case_types, non_snake_case)]
pub struct OrderUpdate {
    pub exchange: Exchanges,
    pub id: i64,
    pub client_id: Option<String>,
    pub market: String,
    #[serde(rename = "type")]
    pub type_: OrderType,
    pub side: OrderSide,
    pub size: f64,
    pub price: f64,
    pub reduceOnly: bool,
    pub ioc: bool,
    pub postOnly: bool,
    pub status: OrderStatus,
    pub filledSize: f64,
    pub remainingSize: f64,
    pub avgFillPrice: Option<f64>,
}
impl OrderUpdate {
    pub fn cache_key(&self) -> String {
        match &self.client_id {
            None => {
                panic!("Error getting OrderUpdate Key");
            }
            Some(cid) => return Clone::clone(cid),
        }
    }
    pub fn has_cache_key(&self) -> bool {
        match self.client_id {
            None => false,
            Some(_) => true,
        }
    }
}
impl Default for OrderUpdate {
    fn default() -> Self {
        OrderUpdate {
            exchange: Exchanges::Unknown,
            id: 0,
            client_id: None,
            market: "".to_string(),
            type_: OrderType::Limit,
            side: OrderSide::Buy,
            size: 0.0,
            price: 0.0,
            reduceOnly: false,
            ioc: false,
            postOnly: false,
            status: OrderStatus::New,
            filledSize: 0.0,
            remainingSize: 0.0,
            avgFillPrice: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_camel_case_types, non_snake_case)]
pub struct OrderFill {
    pub exchange: Exchanges,
    pub fee: f64,
    pub fee_rate: f64,
    pub id: i64,
    pub liquidity: String,
    pub market: String,
    pub orderId: i64,
    pub tradeId: i64,
    pub price: f64,
    pub side: OrderSide,
    pub size: f64,
    pub time: String,
    #[serde(rename = "type")]
    pub type_: String,
}
impl Default for OrderFill {
    fn default() -> Self {
        OrderFill {
            exchange: Exchanges::Unknown,
            fee: 0.0,
            fee_rate: 0.0,
            id: 0,
            liquidity: "".to_string(),
            market: "".to_string(),
            orderId: 0,
            tradeId: 0,
            price: 0.0,
            side: OrderSide::Buy,
            size: 0.0,
            time: "".to_string(),
            type_: OrderType::Limit.to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OrderRequest {
    pub exchange: Exchanges,
    pub market: String,
    pub side: OrderSide,
    pub price: f64,
    pub size: f64,
    pub type_: OrderType,
    pub ioc: bool,
    pub post_only: bool,
    pub client_id: Option<String>,
}
impl OrderRequest {
    pub fn generate_client_id(&mut self) -> &Option<String> {
        self.client_id = Option::from(format!(
            "{}:{}:{}:{}",
            &self.exchange.to_string(),
            &self.market,
            &self.side.to_string(),
            Uuid::new_v4().to_string(),
        ));
        return &self.client_id;
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CancelOrderRequest {
    pub exchange: Exchanges,
    pub client_id: String,
}

impl OrderRequest {
    pub async fn send_order(
        order_update_cache: &OrderUpdateCacheInner,
        message_bus_sender: &tokio::sync::mpsc::Sender<PublishPayload>,
        order_request: OrderRequest,
    ) -> anyhow::Result<()> {
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
        order_update_cache: &OrderUpdateCacheInner,
        message_bus_sender: &tokio::sync::mpsc::Sender<PublishPayload>,
        client_id: &str,
    ) -> anyhow::Result<()> {
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
