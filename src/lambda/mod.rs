mod engine;
mod lambda;
mod lambda_instance;
mod param_service;

pub use engine::engine;
pub use lambda_instance::{LambdaInstance, LambdaInstanceConfig, LambdaParams, GenericLambdaInstanceConfig};

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
