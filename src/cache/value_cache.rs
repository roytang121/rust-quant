use crate::lambda::GenericLambdaInstanceConfig;
use crate::pubsub::simple_message_bus::RedisBackedMessageBus;
use dashmap::DashMap;
use serde_json::Value;
use std::sync::Arc;
use redis::aio::MultiplexedConnection;
use crate::core::config::ConfigStore;
use redis::AsyncCommands;

type Cache = Arc<DashMap<String, Value>>;

#[derive(Debug, strum_macros::Display)]
pub enum ValueCacheKey {
    StrategyParams,
    StrategyStates,
}

pub struct ValueCache {
    cache: Cache,
    instance_config: GenericLambdaInstanceConfig,
    redis_conn: MultiplexedConnection,
}

impl ValueCache {
    pub async fn new(
        instance_config: GenericLambdaInstanceConfig,
    ) -> Self {
        let conf = ConfigStore::load();
        let redis_client = redis::Client::open(conf.redis_url).unwrap();
        let redis_conn = redis_client.get_multiplexed_async_connection().await.unwrap();
        ValueCache {
            cache: Arc::new(DashMap::new()),
            instance_config,
            redis_conn,
        }
    }

    pub fn get_clone(&self, key: ValueCacheKey) -> Option<Value> {
        let key = key.to_string();
        match self.cache.get(&key) {
            None => None,
            Some(value) => Some(value.value().clone()),
        }
    }

    fn get_instance_value_cache_key(&self, value_cache_key: ValueCacheKey) -> String {
        format!("ValueCache:{}:{}", value_cache_key.to_string(), self.instance_config.name.as_str())
    }

    pub fn insert(&self, key: ValueCacheKey, value: Value) -> Option<Value> {
        let old_value = self.cache.insert(key.to_string(), value.clone());
        let set_key = self.get_instance_value_cache_key(key);
        let mut conn = self.redis_conn.clone();
        tokio::spawn(async move {
            let json = value.to_string();
            conn.set::<&str, &str, redis::Value>(set_key.as_str(), json.as_str()).await.unwrap();
        });
        old_value
    }

    pub async fn subscribe(&self) {
        // empty
    }
}
