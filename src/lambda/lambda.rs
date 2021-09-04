use crate::cache::MarketDepthCache;
use crate::lambda::lambda_instance::{
    LambdaStrategyParamsRequest, LambdaStrategyParamsRequestSender,
};
use crate::lambda::strategy::swap_mm::params::{
    StrategyStateEnum, SwapMMInitParams, SwapMMStrategyParams, SwapMMStrategyStateStruct,
};
use crate::lambda::{LambdaInstance, LambdaInstanceConfig};

use crate::model::{Instrument, InstrumentToken, OrderSide};
use crate::pubsub::PublishPayload;

use rocket::tokio::sync::mpsc::error::SendError;

use crate::cache::OrderUpdateCache;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

type InitParams = SwapMMInitParams;
type StrategyParams = SwapMMStrategyParams;
type StrategyState = SwapMMStrategyStateStruct;

pub struct Lambda<'r> {
    market_depth: &'r MarketDepthCache,
    depth_instrument: Instrument<'r>,
    lambda_instance: LambdaInstance,
    strategy_state: Arc<RwLock<StrategyState>>,
    strategy_params_request_sender: LambdaStrategyParamsRequestSender,
}

impl<'r> Lambda<'r> {
    pub fn new(
        instance_config: LambdaInstanceConfig,
        market_depth: &'r MarketDepthCache,
        order_cache: &'r OrderUpdateCache,
        message_bus_sender: tokio::sync::mpsc::Sender<PublishPayload>,
    ) -> Self {
        let lambda_instance = LambdaInstance::new(instance_config);
        let strategy_params_sender = lambda_instance.strategy_params_request_sender.clone();
        let init_params =
            serde_json::from_value::<InitParams>(lambda_instance.init_params.clone()).unwrap();
        let depth_instrument_token =
            Instrument::instrument_token(init_params.depth_symbol.as_str());
        let depth_instrument = match depth_instrument_token {
            InstrumentToken(exchange, market) => Instrument {
                exchange,
                market,
                order_cache,
                message_bus_sender,
            },
        };
        Lambda {
            market_depth,
            depth_instrument,
            lambda_instance,
            strategy_state: Arc::new(RwLock::new(StrategyState {
                state: Default::default(),
                bid_px: None,
                ask_px: None,
                bid_level: None,
                ask_level: None,
            })),
            strategy_params_request_sender: strategy_params_sender,
        }
    }

    async fn get_strategy_params(&self) -> Result<StrategyParams, Box<dyn std::error::Error>> {
        let params = LambdaStrategyParamsRequest::request_strategy_params_snapshot(
            &self.strategy_params_request_sender,
        )
        .await?;
        let transformed = serde_json::from_value::<StrategyParams>(params).unwrap();
        Ok(transformed)
    }

    async fn read_strategy_state(&self) -> anyhow::Result<RwLockReadGuard<'_, StrategyState>> {
        let state = self.strategy_state.read().await;
        Ok(state)
    }

    async fn write_strategy_state(&self) -> anyhow::Result<RwLockWriteGuard<'_, StrategyState>> {
        let state = self.strategy_state.write().await;
        Ok(state)
    }

    async fn publish_state(&self) -> Result<(), SendError<LambdaStrategyParamsRequest>> {
        let strategy_state = self.strategy_state.read().await;
        let copy_state = strategy_state.deref().clone();
        LambdaStrategyParamsRequest::request_set_state(
            &self.strategy_params_request_sender,
            StrategyStateEnum::SwapMM(copy_state),
        )
        .await
    }

    async fn update(&self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // info!("lambda update in thread_id: {:?}", std::thread::current().id());
            if let Some(md) = self
                .market_depth
                .get_clone(self.depth_instrument.market.as_str())
            {
                let targe_size = 31.0f64;

                let mut sum_bid_size = 0.0f64;
                let mut target_bid_price = 0.0f64;
                let mut target_bid_level = 0;
                for level in md.bids.iter() {
                    sum_bid_size += level.size;
                    if sum_bid_size >= targe_size {
                        target_bid_price = level.price;
                        break;
                    }
                    target_bid_level += 1;
                }

                let mut sum_ask_size = 0.0f64;
                let mut target_ask_price = 0.0f64;
                let mut target_ask_level = 0;
                for level in md.asks.iter() {
                    sum_ask_size += level.size;
                    if sum_ask_size >= targe_size {
                        target_ask_price = level.price;
                        break;
                    }
                    target_ask_level += 1;
                }

                log::info!("bid[{}]: {}", target_bid_level, target_bid_price);
                log::info!("ask[{}]: {}", target_ask_level, target_ask_price);

                let mut strategy_state = self.write_strategy_state().await?;
                strategy_state.bid_px = Some(target_bid_price);
                strategy_state.ask_px = Some(target_ask_price);
                drop(strategy_state);
                self.publish_state().await;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    async fn add_order(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut counter = 20000;
        loop {
            if counter <= 3000 {
                counter += 1;
                let open_orders = self.depth_instrument.get_order_orders(true);
                if open_orders.is_empty() {
                    log::info!("sending order for instrument {:?}", self.depth_instrument);
                    self.depth_instrument
                        .send_order(OrderSide::Buy, 100.0, 0.001)
                        .await?;
                } else {
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    async fn cancel_orders(&self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let open_orders = self.depth_instrument.get_order_orders(false);
            for order in open_orders {
                self.depth_instrument
                    .cancel_order(order.client_id.unwrap().as_str())
                    .await?;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    pub async fn subscribe(&self) -> Result<(), Box<dyn std::error::Error>> {
        tokio::select! {
            result = self.update() => {
                panic!("lambda update panic: {:?}", result)
            }
            result = self.add_order() => {
                panic!("lambda add_order panic: {:?}", result)
            }
            result = self.cancel_orders() => {
                panic!("lambda cancel_orders panic: {:?}", result)
            }
            result = self.lambda_instance.subscribe() => {
                panic!("lambda_instance subscribe_strategy_params panic: {:?}", result)
            }
        }
    }
}
