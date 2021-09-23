use crate::lambda::LambdaState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SwapMMInitParams {
    pub depth_symbol: String,
    pub hedge_symbol: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SwapMMStrategyParams {
    pub state: LambdaState,
    pub min_level: i64,
    pub min_basis: f64,
    pub base_size: f64,
    pub target_acc_size: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SwapMMStrategyStateStruct {
    pub target_bid_px: Option<f64>,
    pub target_ask_px: Option<f64>,
    pub target_bid_level: Option<i64>,
    pub target_ask_level: Option<i64>,
    pub open_bid_px: Option<f64>,
    pub open_ask_px: Option<f64>,
    pub open_bid_level: Option<i64>,
    pub open_ask_level: Option<i64>,
    pub open_bid_cnt: Option<usize>,
    pub open_ask_cnt: Option<usize>,
    pub enable_buy: bool,
    pub enable_sell: bool,
    pub depth_bid_px: Option<f64>,
    pub depth_ask_px: Option<f64>,
    pub bid_basis_bp: Option<f64>,
    pub ask_basis_bp: Option<f64>,
}
