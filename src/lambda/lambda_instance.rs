use crate::lambda::param_service::LambdaStrategyParamService;
use crate::lambda::strategy::swap_mm::params::{StrategyStateEnum, SwapMMStrategyParams};
use crate::lambda::LambdaStateCache;
use confy::ConfyError;
use rocket::tokio::sync::mpsc::error::SendError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::{Borrow, BorrowMut};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot::{Receiver, Sender};
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct LambdaParams {
    pub book: String,
    pub market_depths: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct LambdaInstanceConfig {
    pub name: String,
    pub lambda_params: LambdaParams,
    pub init_params: Value,
    pub strategy_params: Value,
}

impl LambdaInstanceConfig {
    pub fn load(instance_name: &str) -> Self {
        let mut config: LambdaInstanceConfig = confy::load_path(format!("./instance/{}.toml", instance_name)).unwrap();
        if config.name != instance_name {
            panic!("config name != instance_name")
        }
        config.name = instance_name.to_string();
        config
    }
    pub fn save(&self) -> Result<(), ConfyError> {
        let config = LambdaInstanceConfig {
            name: self.name.clone(),
            lambda_params: self.lambda_params.clone(),
            init_params: self.init_params.clone(),
            strategy_params: self.strategy_params.clone(),
        };
        confy::store_path(format!("./instance/{}.toml", self.name), config)
    }
}

pub type LambdaStrategyParamsRequestSender = tokio::sync::mpsc::Sender<LambdaStrategyParamsRequest>;

pub struct LambdaInstance {
    pub name: String,
    pub lambda_params: LambdaParams,
    pub init_params: Value,
    pub strategy_params: RefCell<Value>,
    pub strategy_params_request_sender: LambdaStrategyParamsRequestSender,
    strategy_params_receiver: RwLock<tokio::sync::mpsc::Receiver<LambdaStrategyParamsRequest>>,
    pub state_cache: RefCell<StrategyStateEnum>,
}

#[derive(Debug)]
pub enum RequestType {
    ParamSnapshot {
        result: tokio::sync::oneshot::Sender<Value>,
    },
    UpdateParam {
        value: Value,
    },
    SetState {
        value: StrategyStateEnum,
    },
    StateSnapshot {
        result: tokio::sync::oneshot::Sender<StrategyStateEnum>,
    },
}

#[derive(Debug)]
pub struct LambdaStrategyParamsRequest(RequestType);

impl LambdaStrategyParamsRequest {
    pub async fn request_strategy_params_snapshot(
        sender: &LambdaStrategyParamsRequestSender,
    ) -> anyhow::Result<Value> {
        let (tx, rx) = tokio::sync::oneshot::channel::<Value>();
        let request = LambdaStrategyParamsRequest(RequestType::ParamSnapshot { result: tx });
        sender.send(request).await;
        let snapshot = rx.await?;
        Ok(snapshot)
    }

    pub async fn request_update_strategy_params(
        sender: &LambdaStrategyParamsRequestSender,
        value: Value,
    ) -> Result<(), SendError<LambdaStrategyParamsRequest>> {
        let request = LambdaStrategyParamsRequest(RequestType::UpdateParam { value });
        sender.send(request).await
    }

    pub async fn request_set_state(
        sender: &LambdaStrategyParamsRequestSender,
        value: StrategyStateEnum,
    ) -> Result<(), SendError<LambdaStrategyParamsRequest>> {
        let request = LambdaStrategyParamsRequest(RequestType::SetState { value });
        sender.send(request).await
    }

    pub async fn request_lambda_state_snapshot(
        sender: &LambdaStrategyParamsRequestSender,
    ) -> anyhow::Result<StrategyStateEnum> {
        let (result, rx) = tokio::sync::oneshot::channel::<StrategyStateEnum>();
        let request = LambdaStrategyParamsRequest(RequestType::StateSnapshot { result });
        sender.send(request).await?;
        let snapshot = rx.await?;
        Ok(snapshot)
    }
}

impl LambdaInstance {
    pub fn new(config: LambdaInstanceConfig) -> LambdaInstance {
        let (tx, rx) = tokio::sync::mpsc::channel::<LambdaStrategyParamsRequest>(100);
        LambdaInstance {
            name: config.name.to_string(),
            lambda_params: config.lambda_params,
            init_params: config.init_params,
            strategy_params: RefCell::new(config.strategy_params),
            strategy_params_request_sender: tx,
            strategy_params_receiver: RwLock::new(rx),
            state_cache: RefCell::new(StrategyStateEnum::None),
        }
    }

    async fn save_instance_config(&self) -> Result<(), ConfyError> {
        let strategy_params = &self.strategy_params.borrow();
        let strategy_params = strategy_params.deref().clone();
        let config = LambdaInstanceConfig {
            name: self.name.clone(),
            lambda_params: self.lambda_params.clone(),
            init_params: self.init_params.clone(),
            strategy_params,
        };
        confy::store_path(format!("./instance/{}.toml", self.name), config)
    }

    pub async fn subscribe_strategy_params(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut strategy_params_receiver = self.strategy_params_receiver.write().await;
        while let Some(msg) = strategy_params_receiver.recv().await {
            match msg.0 {
                RequestType::ParamSnapshot { result } => {
                    let value = self.strategy_params.borrow().deref().clone();
                    result.send(value).unwrap();
                }
                RequestType::UpdateParam { value } => {
                    self.strategy_params.replace(value);
                    self.save_instance_config().await?;
                }
                RequestType::SetState { value } => {
                    self.state_cache.replace(value);
                }
                RequestType::StateSnapshot { result } => {
                    let mut snapshot = self.state_cache.borrow().deref().clone();
                    result.send(snapshot);
                }
            }
        }
        Ok(())
    }

    pub async fn subscribe_rest(&self) -> Result<(), rocket::Error> {
        let param_service =
            LambdaStrategyParamService::new(self.strategy_params_request_sender.clone());
        param_service.subscribe().await
    }

    pub async fn subscribe(&self) -> Result<(), Box<dyn std::error::Error>> {
        tokio::select! {
            Err(err) = self.subscribe_strategy_params() => {},
            result = self.subscribe_rest() => {
                panic!("subscribe_rest error: {:?}", result)
            }
        }
        Ok(())
    }
}
