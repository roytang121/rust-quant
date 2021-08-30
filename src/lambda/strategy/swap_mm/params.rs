use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SwapMMStrategyParams {
    depth_symbol: String,
    hedge_symbol: String,
}
