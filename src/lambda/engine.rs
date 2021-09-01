use crate::cache::MarketDepthCache;
use crate::core::config::ConfigStore;
use crate::core::orders::{OrderGateway, OrderUpdateService};
use crate::ftx::ftx_order_gateway::FtxOrderGateway;
use crate::lambda::lambda::Lambda;
use crate::lambda::LambdaInstance;
use crate::model::constants::Exchanges;
use crate::model::market_data_model::MarketDepth;
use crate::model::Instrument;
use crate::pubsub::simple_message_bus::RedisBackedMessageBus;
use crate::pubsub::SubscribeMarketDepthRequest;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub async fn engine(lambda_instance: LambdaInstance) {
    log::info!(
        "Starting engine. Lambda Instance Config {:?}",
        &lambda_instance.lambda_params,
        // lambda_instance.strategy_params.read().await.get()
    );
    // market depth request
    let mut market_depth_cache = MarketDepthCache::new();
    let md_cache_ref = market_depth_cache.cache.clone();
    let market_depth_tokens = &lambda_instance.lambda_params.market_depths;
    let market_depth_requests: Vec<SubscribeMarketDepthRequest> = market_depth_tokens
        .iter()
        .map(|token| SubscribeMarketDepthRequest::from_token(token.as_str()))
        .collect();
    let md_cache_poll = tokio::spawn(async move {
        market_depth_cache
            .subscribe(market_depth_requests.as_slice())
            .await;
    });

    // order gateway
    let mut order_gateway = FtxOrderGateway::new();

    // order update cache
    let mut order_update_service = OrderUpdateService::new();
    let ous_cache_ref = order_update_service.cache.clone();
    let order_update_handle = tokio::spawn(async move {
        order_update_service.subscribe().await;
    });

    // message bus
    let mut message_bus = RedisBackedMessageBus::new().await.unwrap();
    let message_bus_sender = message_bus.publish_tx.clone();

    // lambda
    let lambda = async move {
        tokio::time::sleep(Duration::from_secs(3)).await;
        let instrument = Instrument::new(
            Exchanges::FTX,
            "ETH-PERP",
            ous_cache_ref,
            message_bus_sender,
        );
        let lambda = Lambda::new(md_cache_ref, instrument, lambda_instance);
        lambda.subscribe().await
    };

    tokio::select! {
        Err(err) = md_cache_poll => {
            log::error!("md cache panic: {}", err)
        },
        Err(err) = order_gateway.subscribe() => {
            log::error!("order_gateway panic: {}", err);
        },
        Err(err) = order_update_handle => {
            log::error!("order_update_service panic: {}", err);
        },
        result = lambda => {
            log::error!("lambda completed: {:?}", result)
        },
        _ = message_bus.publish_poll() => {},
    }
}
