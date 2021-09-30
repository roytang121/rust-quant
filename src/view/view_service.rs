use crate::cache::{ValueCache, ValueCacheKey};
use std::sync::Arc;
use std::time::Duration;
use crate::lambda::GenericLambdaInstanceConfig;
use crate::model::constants::PublishChannel;
use crate::pubsub::simple_message_bus::RedisBackedMessageBus;
use crate::view::utils::value_to_entries;

pub struct ViewService {
    instance_config: GenericLambdaInstanceConfig,
    message_bus: Arc<RedisBackedMessageBus>,
    value_cache: Arc<ValueCache>,
}
impl ViewService {
    pub fn new(instance_config: GenericLambdaInstanceConfig, message_bus: Arc<RedisBackedMessageBus>, value_cache: Arc<ValueCache>) -> Self {
        ViewService { instance_config, message_bus, value_cache }
    }

    pub async fn publish_strategy_states(&self) -> anyhow::Result<()> {
        match self.value_cache.get_clone(ValueCacheKey::StrategyStates) {
            None => {}
            Some(value) => {
                let channel = format!("{}:{}", PublishChannel::StrategyStates.to_string() , self.instance_config.name);
                let entries = value_to_entries(&value, "states");
                self.message_bus.publish(&channel, &entries).await?;
            }
        }
        Ok(())
    }

    pub async fn subscribe(&self) -> anyhow::Result<()> {
        loop {
            self.publish_strategy_states().await?;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
}
