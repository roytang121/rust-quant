use crate::lambda::param_service::Stated;
use crate::lambda::LambdaState;
use std::any::Any;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SwapMMInitParams {
    pub depth_symbol: String,
    pub hedge_symbol: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SwapMMStrategyParams {
    pub min_level: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SwapMMStrategyStateStruct {
    pub state: LambdaState,
    pub bid_px: Option<f64>,
    pub ask_px: Option<f64>,
    pub bid_level: Option<i32>,
    pub ask_level: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StrategyStateEnum {
    None,
    SwapMM(SwapMMStrategyStateStruct)
}
impl ToString for StrategyStateEnum {
    fn to_string(&self) -> String {
        match self {
            StrategyStateEnum::None => {
                "{}".to_string()
            }
            StrategyStateEnum::SwapMM(value) => {
                serde_json::to_string(value).unwrap()
            }
        }
    }
}