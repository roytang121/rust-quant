use crate::model::market_data_model::MarketDepth;
use crate::model::constants::Exchanges;
use tokio::sync::RwLock;
use crate::pubsub::{SubscribeMarketDepthRequest, MarketDepthCache};
use std::sync::Arc;
use std::time::Duration;
use crate::ftx::ftx_order_gateway::FtxOrderGateway;
use crate::core::orders::{OrderGateway, OrderUpdateService};
use crate::core::config::ConfigStore;
use crate::pubsub::simple_message_bus::RedisBackedMessageBus;
use crate::model::Instrument;
use crate::lambda::lambda::Lambda;

pub async fn engine(market_depth_requests: Vec<SubscribeMarketDepthRequest>, optimizer: i32) {
    // market depth request
    let mut market_depth_cache = MarketDepthCache::new();
    let md_cache_ref = market_depth_cache.cache.clone();
    let md_cache_poll = tokio::spawn(async move {
        market_depth_cache.subscribe().await;
    });

    // order gateway
    let mut order_gateway = FtxOrderGateway::new();

    // order update cache
    let mut order_update_service = OrderUpdateService::new();
    let ous_cache_ref = order_update_service.cache.clone();

    // message bus
    let mut message_bus = RedisBackedMessageBus::new().await.unwrap();
    let message_bus_sender = message_bus.publish_tx.clone();

    // lambda
    let lambda = async move {
        tokio::time::sleep(Duration::from_secs(3)).await;
        let instrument = Instrument::new(Exchanges::FTX, "ETH-PERP", ous_cache_ref, message_bus_sender);
        let lambda = Lambda::new(md_cache_ref, instrument);
        lambda.subscribe().await
    };

    tokio::select! {
        Err(err) = md_cache_poll => {
            log::error!("md cache panic: {}", err)
        },
        Err(err) = order_gateway.subscribe() => {
            log::error!("order_gateway panic: {}", err);
        },
        Err(err) = order_update_service.subscribe() => {
            log::error!("order_update_service panic: {}", err);
        },
        _ = lambda => {},
        _ = message_bus.publish_poll() => {},
    }
}