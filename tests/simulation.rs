#[cfg(test)]
mod test_common;

#[cfg(test)]
mod lambda_test {
    use rust_quant::lambda::{Lambda, GenericLambdaInstanceConfig};
    use rust_quant::cache::{MarketDepthCache, OrderUpdateCache};
    use std::sync::Arc;
    use rust_quant::pubsub::simple_message_bus::RedisBackedMessageBus;
    use rust_quant::model::MeasurementCache;

    #[tokio::test]
    pub async fn it_init() {
        std::env::set_var("ENV", "sim");
        let instance_config = GenericLambdaInstanceConfig::load("swap-mm-sim");
        let market_depth = Arc::new(MarketDepthCache::new());
        let order_update_cache = Arc::new(OrderUpdateCache::new());
        let message_bus = Arc::new(RedisBackedMessageBus::new().await.unwrap());
        let measurement_cache = Arc::new(MeasurementCache::new().await);
        let lambda = Lambda::new(
            instance_config,
            market_depth.clone(),
            order_update_cache.clone(),
            message_bus.publish_tx.clone(),
            measurement_cache.clone(),
        );
        lambda.subscribe().await;
    }
}