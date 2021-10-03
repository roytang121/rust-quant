use std::sync::Arc;

use crate::cache::OrderUpdateCache;
use crate::cache::{MarketDepthCache, ValueCache, ValueCacheKey};

use crate::core::OrderGateway;
use crate::ftx::ftx_order_gateway::FtxOrderGateway;

use crate::ftx::FtxRestClient;
use crate::lambda::lambda_instance::GenericLambdaInstanceConfig;
use crate::lambda::{LambdaInstanceConfig};
use crate::model::MeasurementCache;
use crate::pubsub::simple_message_bus::{MessageBusSender, RedisBackedMessageBus};
use crate::pubsub::SubscribeMarketDepthRequest;
use crate::view::view_service::ViewService;
use std::time::Duration;
use crate::lambda::strategy::LambdaRegistry;
use crate::lambda::strategy::swap_mm::lambda::Lambda;

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

pub async fn thread_order_gateway(
    message_bus_sender: MessageBusSender,
    measurement_cache: Arc<MeasurementCache>,
) -> anyhow::Result<()> {
    // order gateway
    tokio::spawn(async move {
        loop {
            let client = Arc::new(FtxRestClient::new());
            let ftx_order_gateway = FtxOrderGateway::new(
                message_bus_sender.clone(),
                client.clone(),
                measurement_cache.clone(),
            );
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

pub struct LambdaEngine {
    instance_config: GenericLambdaInstanceConfig,
    message_bus: Arc<RedisBackedMessageBus>,
    message_bus_sender: MessageBusSender,
    market_depth_cache: Arc<MarketDepthCache>,
    order_update_cache: Arc<OrderUpdateCache>,
    measurement_cache: Arc<MeasurementCache>,
    value_cache: Arc<ValueCache>,
}

impl LambdaEngine {
    pub async fn init(instance_config: GenericLambdaInstanceConfig) -> Self {
        // market depth request
        let market_depth_cache = Arc::new(MarketDepthCache::new());

        // order update cache
        let order_update_cache = Arc::new(OrderUpdateCache::new());

        // message bus
        let message_bus = Arc::new(RedisBackedMessageBus::new().await.unwrap());
        let message_bus_sender = message_bus.publish_tx.clone();

        // measurement cache
        let measurement_cache = Arc::new(MeasurementCache::new().await);

        // value cache
        let value_cache = Arc::new(ValueCache::new(instance_config.clone()).await);

        // get lambda params
        let lambda_instance_config = LambdaInstanceConfig::load(instance_config.name.as_str());
        value_cache.insert(
            ValueCacheKey::StrategyParams,
            lambda_instance_config.strategy_params.clone(),
        );

        return LambdaEngine {
            instance_config,
            message_bus,
            message_bus_sender,
            market_depth_cache,
            order_update_cache,
            measurement_cache,
            value_cache,
        };
    }

    pub async fn subscribe_lambda(&self) -> anyhow::Result<()> {
        match self.instance_config.registry {
            LambdaRegistry::SwapMM => {
                // lambda
                let lambda = Lambda::new(
                    self.instance_config.clone(),
                    self.market_depth_cache.clone(),
                    self.order_update_cache.clone(),
                    self.message_bus_sender.clone(),
                    self.measurement_cache.clone(),
                    self.value_cache.clone(),
                );

                lambda.subscribe().await?;
            }
            LambdaRegistry::LatencyMM => {}
        }
        Ok(())
    }

    pub async fn subscribe(&self) -> anyhow::Result<()> {
        let market_depth_tokens = &self.instance_config.lambda_params.market_depths;
        let market_depth_requests: Vec<SubscribeMarketDepthRequest> = market_depth_tokens
            .iter()
            .map(|token| SubscribeMarketDepthRequest::from_token(token.as_str()))
            .collect();

        let view_service = ViewService::new(
            self.instance_config.clone(),
            self.message_bus.clone(),
            self.value_cache.clone(),
        );

        tokio::select! {
            Err(err) = thread_market_depth(self.market_depth_cache.clone(), market_depth_requests) => {
                log::error!("market_depth_cache panic: {}", err)
            },
            Err(err) = thread_order_gateway(self.message_bus_sender.clone(), self.measurement_cache.clone()) => {
                log::error!("order_gateway panic: {}", err);
            },
            Err(err) = thread_order_update_cache(self.order_update_cache.clone()) => {
                log::error!("order_update_service panic: {}", err);
            },
            result = self.message_bus.subscribe() => {
                log::error!("message_bus completed: {:?}", result)
            },
            result = self.subscribe_lambda() => {
                log::error!("lambda completed: {:?}", result)
            },
            result = view_service.subscribe() => {
                log::error!("view_serivce completed: {:?}", result)
            }
        }
        Ok(())
    }
}
