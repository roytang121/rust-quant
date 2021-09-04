use crate::model::constants::Exchanges;
use crate::model::{OrderRequest, OrderSide, OrderStatus, OrderType, OrderUpdate, OrderFill};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_camel_case_types)]
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
#[allow(non_camel_case_types)]
pub enum FtxOrderType {
    limit,
    market,
}
#[derive(Deserialize, Serialize, Debug)]
#[allow(non_camel_case_types)]
pub enum FtxOrderStatus {
    new,
    open,
    closed,
}
#[derive(Deserialize, Serialize, Debug)]
#[allow(non_camel_case_types)]
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
    pub bids: Vec<[f64; 2]>,
    pub asks: Vec<[f64; 2]>,
    pub checksum: u64,
    pub time: f64,
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_camel_case_types, non_snake_case)]
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
    pub createdAt: String,
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
#[allow(non_camel_case_types, non_snake_case)]
pub struct FtxOrderFill {
    pub fee: f64,
    pub feeRate: f64,
    pub future: Option<String>,
    pub id: i64,
    pub liquidity: String,
    pub market: String,
    pub orderId: i64,
    pub tradeId: i64,
    pub price: f64,
    pub side: FtxOrderSide,
    pub size: f64,
    pub time: String,
    #[serde(rename = "type")]
    pub type_: String,
}
impl FtxOrderFill {
    pub fn to_order_fill(&self) -> OrderFill {
        OrderFill {
            exchange: Exchanges::FTX,
            fee: self.fee,
            fee_rate: self.feeRate,
            id: self.id,
            liquidity: self.liquidity.clone(),
            market: self.market.clone(),
            orderId: self.orderId,
            tradeId: self.tradeId,
            price: self.price,
            side: match self.side {
                FtxOrderSide::buy => OrderSide::Buy,
                FtxOrderSide::sell => OrderSide::Sell,
            },
            size: self.size,
            time: self.time.clone(),
            type_: self.type_.clone(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_camel_case_types, non_snake_case)]
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
