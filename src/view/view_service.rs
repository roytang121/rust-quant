use crate::cache::{ValueCache, ValueCacheKey};
use crate::lambda::GenericLambdaInstanceConfig;
use crate::model::constants::PublishChannel;
use crate::pubsub::simple_message_bus::{MessageConsumer, RedisBackedMessageBus};
use crate::view::utils::{value_to_entries, KeyValueEntry};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;

pub struct ViewService {
    instance_config: GenericLambdaInstanceConfig,
    message_bus: Arc<RedisBackedMessageBus>,
    value_cache: Arc<ValueCache>,
}
impl ViewService {
    pub fn new(
        instance_config: GenericLambdaInstanceConfig,
        message_bus: Arc<RedisBackedMessageBus>,
        value_cache: Arc<ValueCache>,
    ) -> Self {
        ViewService {
            instance_config,
            message_bus,
            value_cache,
        }
    }

    pub async fn publish_strategy_states(&self) -> anyhow::Result<()> {
        match self.value_cache.get_clone(ValueCacheKey::StrategyStates) {
            None => {}
            Some(value) => {
                let channel = format!(
                    "{}:{}",
                    PublishChannel::StrategyStates.to_string(),
                    self.instance_config.name
                );
                let entries = value_to_entries(&value, "states");
                self.message_bus.publish(&channel, &entries).await?;
            }
        }
        Ok(())
    }

    pub async fn publish_strategy_params(&self) -> anyhow::Result<()> {
        match self.value_cache.get_clone(ValueCacheKey::StrategyParams) {
            None => {}
            Some(value) => {
                let channel = format!(
                    "{}:{}",
                    PublishChannel::StrategyParams.to_string(),
                    self.instance_config.name,
                );
                let entries = value_to_entries(&value, "params");
                self.message_bus.publish(&channel, &entries).await?;
            }
        }
        Ok(())
    }

    pub async fn update_params(&self) -> anyhow::Result<()> {
        let consumer = ParamUpdateConsumer(self.value_cache.clone());
        RedisBackedMessageBus::subscribe_channels(
            vec![format!(
                "{}:{}",
                PublishChannel::UpdateParam.to_string(),
                self.instance_config.name
            )
            .as_str()],
            &consumer,
        )
        .await;
        Ok(())
    }

    pub async fn publish_state_entries(&self) -> anyhow::Result<()> {
        loop {
            self.publish_strategy_states().await?;
            self.publish_strategy_params().await?;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        Ok(())
    }

    pub async fn subscribe(&self) -> anyhow::Result<()> {
        tokio::select! {
            result = self.publish_state_entries() => {
                error!("publish_state_entries completed: {:?}", result);
            }
            result = self.update_params() => {
                error!("update_params completed: {:?}", result);
            }
        }
        Ok(())
    }
}

struct ParamUpdateConsumer(Arc<ValueCache>);
#[async_trait::async_trait]
impl MessageConsumer for ParamUpdateConsumer {
    async fn consume(&self, msg: &[u8]) -> anyhow::Result<()> {
        let entry = serde_json::from_slice::<KeyValueEntry>(msg)
            .expect("ParamUpdateConsumer Failed to consumer message");
        info!("Receive UpdateParma {:?}", entry);
        if let Some(mut params) = self.0.get_clone(ValueCacheKey::StrategyParams) {
            match params.get(&entry.key) {
                None => {
                    warn!(
                        "Error updating StrategyParams, key not found : {}",
                        entry.key
                    );
                }
                Some(_) => params[entry.key.clone()] = entry.value.clone(),
            }
            self.0.insert(ValueCacheKey::StrategyParams, params);
        } else {
            error!("Cannot get StrategyParams from ValueCache")
        }
        Ok(())
    }
}
