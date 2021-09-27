#[cfg(test)]
mod test_common;
#[cfg(test)]
use mockall::{automock, mock, predicate::*};

#[cfg(test)]
mod lambda_test {
    use super::*;
    use rust_quant::cache::{MarketDepthCache, OrderUpdateCache};
    use rust_quant::lambda::{GenericLambdaInstanceConfig, Lambda, LambdaState};
    use rust_quant::model::{InstrumentSymbol, MeasurementCache, Instrument};
    use rust_quant::pubsub::simple_message_bus::RedisBackedMessageBus;
    use rust_quant::pubsub::SubscribeMarketDepthRequest;
    use std::str::FromStr;
    use std::sync::Arc;
    use test_common::common::*;
    use rust_quant::model::market_data_model::{MarketDepth, PriceLevel};
    use rust_quant::model::constants::Exchanges;

    #[tokio::test]
    pub async fn it_init() {
        std::env::set_var("RUST_LOG", "INFO");
        std::env::set_var("ENV", "sim");
        let instance_config = GenericLambdaInstanceConfig::load("swap-mm-sim");
        let subscribe_md_requests = instance_config
            .lambda_params
            .market_depths
            .clone()
            .into_iter()
            .map(|token| InstrumentSymbol::from_str(token.as_str()).unwrap())
            .map(|symbol| SubscribeMarketDepthRequest::new(symbol.0, symbol.1.as_str()))
            .collect::<Vec<SubscribeMarketDepthRequest>>();
        let market_depth_cache = Arc::new(MarketDepthCache::new());
        let order_update_cache = Arc::new(OrderUpdateCache::new());
        let message_bus = Arc::new(RedisBackedMessageBus::new().await.unwrap());
        let measurement_cache = Arc::new(MeasurementCache::new().await);
        let lambda = Arc::new(Lambda::new(
            instance_config,
            market_depth_cache.clone(),
            order_update_cache.clone(),
            message_bus.publish_tx.clone(),
            measurement_cache.clone(),
        ));

        spawn_thread_market_depth_cache(market_depth_cache.clone(), subscribe_md_requests);
        let handle = spawn_thread_lambda(lambda.clone());

        sleep(1000).await;
        
        let md_0 = MarketDepth {
            timestamp: chrono::Utc::now().timestamp_millis(),
            exchange: Exchanges::SIM,
            market: "ETH-PERP".to_string(),
            bids: vec![PriceLevel { price: 99.0, size: 1.0 }],
            asks: vec![PriceLevel { price: 101.0, size: 1.0 }]
        };
        market_depth_cache.cache.insert("ETH-PERP".to_string(), md_0);

        sleep(20).await;

        assert!(matches!(lambda.get_strategy_params().state, LambdaState::Init));
        assert_eq!(lambda.get_strategy_state().depth_bid_px, Some(99.0));

        drop(handle);
    }
}
