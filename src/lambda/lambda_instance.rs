use crate::lambda::param_service::LambdaStrategyParamService;

use confy::ConfyError;
use rocket::tokio::sync::mpsc::error::SendError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use dashmap::DashMap;

use std::fmt::Debug;
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
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct GenericLambdaInstanceConfig {
    pub name: String,
    pub lambda_params: LambdaParams,
}

impl LambdaInstanceConfig {
    pub fn load(instance_name: &str) -> Self {
        let mut config: LambdaInstanceConfig =
            confy::load_path(format!("./instance/{}.toml", instance_name)).unwrap();
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

impl GenericLambdaInstanceConfig {
    pub fn load(instance_name: &str) -> Self {
        let mut config: GenericLambdaInstanceConfig =
            confy::load_path(format!("./instance/{}.toml", instance_name)).unwrap();
        if config.name != instance_name {
            panic!("config name != instance_name")
        }
        config.name = instance_name.to_string();
        config
    }
}

pub type LambdaStrategyParamsRequestSender = tokio::sync::mpsc::Sender<LambdaStrategyParamsRequest>;

#[derive(
    Serialize,
    Deserialize,
    Debug,
    strum_macros::EnumString,
    strum_macros::Display,
    Clone,
    PartialOrd,
    PartialEq,
)]
pub enum LambdaInstanceValueCacheKey {
    StrategyState,
    StrategyParam,
}

pub struct LambdaInstance {
    pub name: String,
    pub lambda_params: LambdaParams,
    pub init_params: Value,
    pub strategy_params_request_sender: LambdaStrategyParamsRequestSender,
    strategy_params_receiver: RwLock<tokio::sync::mpsc::Receiver<LambdaStrategyParamsRequest>>,
    pub value_cache: DashMap<String, Option<Value>>,
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
        value: Value,
    },
    StateSnapshot {
        result: tokio::sync::oneshot::Sender<Option<Value>>,
    },
}

#[derive(Debug)]
pub struct LambdaStrategyParamsRequest(RequestType);

impl LambdaStrategyParamsRequest {
    pub async fn request_strategy_params_snapshot(
        sender: &LambdaStrategyParamsRequestSender,
    ) -> anyhow::Result<Value> {
        let (result, rx) = tokio::sync::oneshot::channel::<Value>();
        let request = LambdaStrategyParamsRequest(RequestType::ParamSnapshot { result });
        sender.send(request).await?;
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
        value: Value,
    ) -> Result<(), SendError<LambdaStrategyParamsRequest>> {
        let request = LambdaStrategyParamsRequest(RequestType::SetState { value });
        sender.send(request).await
    }

    pub async fn request_lambda_state_snapshot(
        sender: &LambdaStrategyParamsRequestSender,
    ) -> anyhow::Result<Option<Value>> {
        let (result, rx) = tokio::sync::oneshot::channel::<Option<Value>>();
        let request = LambdaStrategyParamsRequest(RequestType::StateSnapshot { result });
        sender.send(request).await?;
        let snapshot = rx.await?;
        Ok(snapshot)
    }
}

impl LambdaInstance {
    pub fn new(config: LambdaInstanceConfig) -> LambdaInstance {
        let (tx, rx) = tokio::sync::mpsc::channel::<LambdaStrategyParamsRequest>(100);
        let value_cache = DashMap::new();
        value_cache.insert(LambdaInstanceValueCacheKey::StrategyState.to_string(), None);
        value_cache.insert(
            LambdaInstanceValueCacheKey::StrategyParam.to_string(),
            Some(config.strategy_params),
        );
        LambdaInstance {
            name: config.name.to_string(),
            lambda_params: config.lambda_params,
            init_params: config.init_params,
            strategy_params_request_sender: tx,
            strategy_params_receiver: RwLock::new(rx),
            value_cache,
        }
    }

    async fn save_instance_config(&self) -> Result<(), ConfyError> {
        let strategy_params = self
            .value_cache
            .get(
                LambdaInstanceValueCacheKey::StrategyParam
                    .to_string()
                    .as_str(),
            )
            .unwrap()
            .value()
            .clone()
            .unwrap();
        let config = LambdaInstanceConfig {
            name: self.name.clone(),
            lambda_params: self.lambda_params.clone(),
            init_params: self.init_params.clone(),
            strategy_params,
        };
        confy::store_path(format!("./instance/{}.toml", self.name), config)
    }

    pub fn get_strategy_params_clone(&self) -> Option<Value> {
        return match self.value_cache.get(
            LambdaInstanceValueCacheKey::StrategyParam
                .to_string()
                .as_str(),
        ) {
            None => None,
            Some(value) => value.value().clone(),
        };
    }

    pub async fn subscribe_strategy_params_requests(
        &self,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut strategy_params_receiver = self.strategy_params_receiver.write().await;
        while let Some(msg) = strategy_params_receiver.recv().await {
            match msg.0 {
                RequestType::ParamSnapshot { result } => {
                    if let Some(value) = self.value_cache.get(
                        LambdaInstanceValueCacheKey::StrategyParam
                            .to_string()
                            .as_str(),
                    ) {
                        match value.value().clone() {
                            None => {
                                panic!("Request Strategy params but it's nil");
                            }
                            Some(value) => {
                                result.send(value).unwrap();
                            }
                        }
                    } else {
                        panic!("Uncaught: getting ParamSnapshot should not be nil")
                    }
                }
                RequestType::UpdateParam { value } => {
                    info!("UpdateParam: {}", value);
                    self.value_cache.insert(
                        LambdaInstanceValueCacheKey::StrategyParam.to_string(),
                        Some(value),
                    );
                    self.save_instance_config().await?;
                }
                RequestType::SetState { value } => {
                    self.value_cache.insert(
                        LambdaInstanceValueCacheKey::StrategyState.to_string(),
                        Some(value),
                    );
                }
                RequestType::StateSnapshot { result } => {
                    if let Some(value) = self.value_cache.get(
                        LambdaInstanceValueCacheKey::StrategyState
                            .to_string()
                            .as_str(),
                    ) {
                        match value.value().clone() {
                            None => {
                                result.send(None);
                            }
                            Some(value) => {
                                result.send(Some(value));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn subscribe_rest(&self) -> anyhow::Result<()> {
        let sender = self.strategy_params_request_sender.clone();
        tokio::spawn(async move {
            info!("spawning subscribe_rest");
            let param_service = LambdaStrategyParamService::new(sender);
            param_service.subscribe().await;
        })
        .await;
        Err(anyhow!("subscribe_rest uncaught"))
    }

    pub async fn subscribe(&self) -> Result<(), Box<dyn std::error::Error>> {
        tokio::select! {
            Err(_err) = self.subscribe_strategy_params_requests() => {},
            result = self.subscribe_rest() => {
                error!("subscribe_rest error: {:?}", result)
            }
        }
        Ok(())
    }
}
