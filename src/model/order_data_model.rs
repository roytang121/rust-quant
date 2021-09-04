use crate::model::constants::Exchanges;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use uuid::Uuid;

pub type OrderCacheKey = String;
pub type OrderUpdateCacheInner = DashMap<OrderCacheKey, OrderUpdate>;

#[derive(Deserialize, Serialize, Debug, strum_macros::Display, Clone)]
pub enum OrderType {
    Limit,
    Market,
}
#[derive(Deserialize, Serialize, Debug, strum_macros::Display, Clone)]
pub enum OrderStatus {
    New,
    Open,
    Closed,
    PendingNew,
    PendingCancel,
}
#[derive(Deserialize, Serialize, Debug, strum_macros::Display, Clone)]
pub enum OrderSide {
    Buy,
    Sell,
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
    pub fn generate_client_id(&mut self) {
        self.client_id = Option::from(format!(
            "{}:{}:{}:{}",
            &self.exchange.to_string(),
            &self.market,
            &self.side.to_string(),
            Uuid::new_v4().to_string(),
        ))
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CancelOrderRequest {
    pub exchange: Exchanges,
    pub client_id: String,
}
