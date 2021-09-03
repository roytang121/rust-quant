use crate::cache::MarketDepthCache;
use crate::core::config::ConfigStore;
use crate::core::orders::{OrderGateway, OrderUpdateService};
use crate::ftx::ftx_order_gateway::FtxOrderGateway;
use crate::lambda::lambda::Lambda;
use crate::lambda::{LambdaInstance, LambdaInstanceConfig, LambdaState};
use crate::model::constants::Exchanges;
use crate::model::market_data_model::MarketDepth;
use crate::model::Instrument;
use crate::pubsub::simple_message_bus::RedisBackedMessageBus;
use crate::pubsub::SubscribeMarketDepthRequest;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub async fn thread_market_depth(
    market_depth_cache: Arc<MarketDepthCache>,
    market_depth_requests: Vec<SubscribeMarketDepthRequest>,
) -> anyhow::Result<()> {
    tokio::spawn(async move {
        market_depth_cache.subscribe(&market_depth_requests).await;
    })
    .await;
    Ok(())
}

pub async fn engine(instance_config: LambdaInstanceConfig) {
    // market depth request
    let market_depth_cache = Arc::new(MarketDepthCache::new());
    let market_depth_tokens = &instance_config.lambda_params.market_depths;
    let market_depth_requests: Vec<SubscribeMarketDepthRequest> = market_depth_tokens
        .iter()
        .map(|token| SubscribeMarketDepthRequest::from_token(token.as_str()))
        .collect();

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
    let lambda = Lambda::new(
        instance_config,
        &market_depth_cache,
        ous_cache_ref,
        message_bus_sender,
    );

    tokio::select! {
        Err(err) = thread_market_depth(market_depth_cache.clone(), market_depth_requests) => {
            log::error!("market_depth_cache panic: {}", err)
        },
        Err(err) = order_gateway.subscribe() => {
            log::error!("order_gateway panic: {}", err);
        },
        Err(err) = order_update_handle => {
            log::error!("order_update_service panic: {}", err);
        },
        result = lambda.subscribe() => {
            log::error!("lambda completed: {:?}", result)
        },
        _ = message_bus.publish_poll() => {},
    }
}
