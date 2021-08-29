use serde::{Deserialize, Serialize};
use crate::model::{OrderUpdate, OrderType, OrderSide, OrderStatus, OrderRequest};
use crate::model::constants::Exchanges;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug)]
pub enum WebSocketResponseType {
    error,
    subscribed,
    unsubscribed,
    info,
    partial,
    update,
    pong,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum FtxOrderType {
    limit,
    market,
}
#[derive(Deserialize, Serialize, Debug)]
pub enum FtxOrderStatus {
    new,
    open,
    closed,
}
#[derive(Deserialize, Serialize, Debug)]
pub enum FtxOrderSide {
    buy,
    sell,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct WebSocketResponse<DataType> {
    pub channel: Option<String>,
    pub market: Option<String>,
    #[serde(rename = "type")]
    pub type_: WebSocketResponseType,
    pub code: Option<i32>,
    pub msg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<DataType>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OrderBookData {
    pub action: WebSocketResponseType,
    pub bids: Vec<[f32; 2]>,
    pub asks: Vec<[f32; 2]>,
    pub checksum: u64,
    pub time: f64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FtxOrderData {
    pub id: i64,
    pub clientId: Option<String>,
    pub market: String,
    #[serde(rename = "type")]
    pub type_: FtxOrderType,
    pub side: FtxOrderSide,
    pub size: f64,
    pub price: f64,
    pub reduceOnly: bool,
    pub ioc: bool,
    pub postOnly: bool,
    pub status: FtxOrderStatus,
    pub filledSize: f64,
    pub remainingSize: f64,
    pub avgFillPrice: Option<f64>,
    // pub createdAt: String,
}

impl FtxOrderData {
    pub fn to_order_update(&self) -> OrderUpdate {
        OrderUpdate {
            exchange: Exchanges::FTX,
            id: self.id.clone(),
            client_id: self.clientId.clone(),
            market: self.market.clone(),
            type_: match self.type_ {
                FtxOrderType::limit => OrderType::Limit,
                FtxOrderType::market => OrderType::Market,
            },
            side: match self.side {
                FtxOrderSide::buy => OrderSide::Buy,
                FtxOrderSide::sell => OrderSide::Sell,
            },
            size: self.size,
            price: self.price,
            reduceOnly: self.reduceOnly,
            ioc: self.ioc,
            postOnly: self.postOnly,
            status: match self.status {
                FtxOrderStatus::new => OrderStatus::New,
                FtxOrderStatus::open => OrderStatus::Open,
                FtxOrderStatus::closed => OrderStatus::Closed,
            },
            filledSize: self.filledSize,
            remainingSize: self.remainingSize,
            avgFillPrice: self.avgFillPrice,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FtxPlaceOrder {
    pub market: String,
    pub side: FtxOrderSide,
    pub price: f64,
    #[serde(rename(deserialize = "type"))]
    pub type_: FtxOrderType,
    pub size: f64,
    pub reduceOnly: bool,
    pub ioc: bool,
    pub postOnly: bool,
    pub clientId: Option<String>,
}
impl FtxPlaceOrder {
    pub fn from_order_request(or: OrderRequest) -> Self {
        FtxPlaceOrder {
            market: or.market.clone(),
            side: match or.side {
                OrderSide::Buy => FtxOrderSide::buy,
                OrderSide::Sell => FtxOrderSide::sell,
            },
            price: or.price,
            type_: match or.type_ {
                OrderType::Limit => FtxOrderType::limit,
                OrderType::Market => FtxOrderType::market,
            },
            size: or.size,
            reduceOnly: false,
            ioc: or.ioc,
            postOnly: or.post_only,
            clientId: Option::from(or.client_id),
        }
    }
}