use serde::{Deserialize, Serialize};
use std::convert::AsRef;
use strum_macros::AsRefStr;

#[derive(Serialize, Deserialize, Debug, strum_macros::Display, Clone, PartialOrd, PartialEq)]
pub enum Exchanges {
    FTX,
    BINANCE,
    OKEX,
}

#[derive(
    Serialize, Deserialize, Debug, strum_macros::Display, AsRefStr, Clone, PartialOrd, PartialEq,
)]
pub enum PublishChannel {
    OrderUpdate,
    OrderRequest,
    MarketDepth,
    CancelOrder,
}
