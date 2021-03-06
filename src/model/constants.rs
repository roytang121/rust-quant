use serde::{Deserialize, Serialize};

use strum_macros::AsRefStr;
pub use strum_macros::EnumString;

#[derive(
    Serialize, Deserialize, Debug, EnumString, strum_macros::Display, Clone, PartialOrd, PartialEq,
)]
pub enum Exchanges {
    FTX,
    BINANCE,
    OKEX,
    SIM,
    Unknown,
}

#[derive(
    Serialize,
    Deserialize,
    Debug,
    strum_macros::Display,
    EnumString,
    AsRefStr,
    Clone,
    PartialOrd,
    PartialEq,
)]
pub enum PublishChannel {
    OrderUpdate,
    OrderFill,
    OrderRequest,
    MarketDepth,
    CancelOrder,
    StrategyStates,
    StrategyParams,
    UpdateParam,
}
