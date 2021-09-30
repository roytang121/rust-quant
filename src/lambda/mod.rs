mod engine;
mod lambda;
mod lambda_instance;
mod param_service;

pub use engine::{LambdaEngine};
pub use lambda::{ SimpleHedger, Lambda };
pub use lambda_instance::{
    GenericLambdaInstanceConfig, LambdaInstance, LambdaInstanceConfig, LambdaParams,
};

pub mod strategy {
    pub mod swap_mm;
}

#[derive(Debug, Serialize, Deserialize, Clone, strum_macros::Display)]
pub enum LambdaState {
    Init,
    Live,
    Paused,
    Stopped,
    AutoPaused,
}
impl Default for LambdaState {
    fn default() -> Self {
        LambdaState::Init
    }
}

pub type LambdaStateCache = dashmap::DashMap<String, serde_json::Value>;
