pub use engine::LambdaEngine;
pub use lambda_instance::{GenericLambdaInstanceConfig, LambdaInstanceConfig, LambdaParams};

mod engine;
mod lambda_instance;
mod param_service;

pub mod strategy {
    pub mod swap_mm;
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub enum LambdaRegistry {
        SwapMM,
        LatencyMM,
    }
    impl Default for LambdaRegistry {
        fn default() -> Self {
            panic!("Should not call default for LambdaRegistry")
        }
    }
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
