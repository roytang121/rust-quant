use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SwapMMStrategyParams {
    pub(crate) depth_symbol: String,
    pub(crate) hedge_symbol: String,
}
