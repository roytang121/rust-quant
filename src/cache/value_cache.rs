use std::sync::Arc;
use dashmap::DashMap;
use serde_json::Value;

type Cache = Arc<DashMap<String, Value>>;

pub struct ValueCache {
    cache: Cache,
}

#[derive(Debug, strum_macros::Display)]
pub enum ValueCacheKey {
    StrategyParams,
    StrategyStates,
}

impl ValueCache {
    pub fn new() -> Self {
        ValueCache {
            cache: Arc::new(DashMap::new())
        }
    }

    pub fn get_clone(&self, key: ValueCacheKey) -> Option<Value> {
        let key = key.to_string();
        match self.cache.get(&key) {
            None => None,
            Some(value) => Some(value.value().clone())
        }
    }

    pub fn insert(&self, key: ValueCacheKey, value: Value) -> Option<Value> {
        self.cache.insert(key.to_string(), value)
    }

    pub async fn subscribe(&self) {
        // empty
    }
}
