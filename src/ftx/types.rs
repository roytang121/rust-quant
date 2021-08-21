use serde::{Deserialize, Serialize};

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
    pub bids: Vec<Vec<f32>>,
    pub asks: Vec<Vec<f32>>,
    pub checksum: u64,
    pub time: f64,
}
