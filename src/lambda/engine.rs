use std::sync::Arc;

use crate::cache::MarketDepthCache;
use crate::cache::OrderUpdateCache;

use crate::core::OrderGateway;
use crate::ftx::ftx_order_gateway::FtxOrderGateway;
use crate::lambda::lambda::Lambda;

use crate::ftx::FtxRestClient;
use crate::lambda::lambda_instance::GenericLambdaInstanceConfig;
use crate::model::MeasurementCache;
use crate::pubsub::simple_message_bus::{MessageBusSender, RedisBackedMessageBus};
use crate::pubsub::SubscribeMarketDepthRequest;
use std::time::Duration;

pub async fn thread_order_update_cache(
    order_update_cache: Arc<OrderUpdateCache>,
) -> anyhow::Result<()> {
    tokio::spawn(async move {
        order_update_cache.subscribe().await;
    })
    .await;
    Err(anyhow!("thread_order_update_cache uncaught error",))
}

pub async fn thread_market_depth(
    market_depth_cache: Arc<MarketDepthCache>,
    market_depth_requests: Vec<SubscribeMarketDepthRequest>,
) -> anyhow::Result<()> {
    tokio::spawn(async move {
        market_depth_cache.subscribe(&market_depth_requests).await;
    })
    .await;
    Err(anyhow!("thread_market_depth uncaught error"))
}

pub async fn thread_order_gateway(message_bus_sender: MessageBusSender) -> anyhow::Result<()> {
    // order gateway
    tokio::spawn(async move {
        loop {
            let client = Arc::new(FtxRestClient::new());
            let ftx_order_gateway =
                FtxOrderGateway::new(message_bus_sender.clone(), client.clone());
            tokio::select! {
                Err(err) = ftx_order_gateway.subscribe() => {
                    error!("ftx_order_gateway: {}", err);
                }
            }
            tokio::time::sleep(Duration::from_millis(3000)).await;
        }
    })
    .await;
    Err(anyhow!("thread_order_gateway uncaught error"))
}

pub async fn engine(instance_config: GenericLambdaInstanceConfig) {
    // market depth request
    let market_depth_cache = Arc::new(MarketDepthCache::new());
    let market_depth_tokens = &instance_config.lambda_params.market_depths;
    let market_depth_requests: Vec<SubscribeMarketDepthRequest> = market_depth_tokens
        .iter()
        .map(|token| SubscribeMarketDepthRequest::from_token(token.as_str()))
        .collect();

    // order update cache
    let order_update_cache = Arc::new(OrderUpdateCache::new());

    // message bus
    let mut message_bus = RedisBackedMessageBus::new().await.unwrap();
    let message_bus_sender = message_bus.publish_tx.clone();

    let measurement_cache = Arc::new(MeasurementCache::new().await);

    // lambda
    let lambda = Lambda::new(
        instance_config,
        market_depth_cache.clone(),
        order_update_cache.clone(),
        message_bus_sender.clone(),
        measurement_cache.clone(),
    );

    tokio::select! {
        Err(err) = thread_market_depth(market_depth_cache.clone(), market_depth_requests) => {
            log::error!("market_depth_cache panic: {}", err)
        },
        Err(err) = thread_order_gateway(message_bus_sender.clone()) => {
            log::error!("order_gateway panic: {}", err);
        },
        Err(err) = thread_order_update_cache(order_update_cache.clone()) => {
            log::error!("order_update_service panic: {}", err);
        },
        result = lambda.subscribe() => {
            log::error!("lambda completed: {:?}", result)
        },
        result = message_bus.subscribe() => {
            log::error!("message_bus completed: {:?}", result)
        },
    }
}
