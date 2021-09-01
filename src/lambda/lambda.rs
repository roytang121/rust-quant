use crate::lambda::lambda_instance::{LambdaStrategyParamsRequest, LambdaStrategyParamsRequestSender};
use crate::lambda::strategy::swap_mm::params::SwapMMStrategyParams;
use crate::lambda::{LambdaInstance, LambdaInstanceConfig};
use crate::model::market_data_model::MarketDepth;
use crate::model::{Instrument, OrderSide};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub struct Lambda {
    market_depth: Arc<DashMap<String, MarketDepth>>,
    depth_instrument: Instrument,
    lambda_instance: Arc<RwLock<LambdaInstance>>,
    strategy_params_request_sender: LambdaStrategyParamsRequestSender,
}

type StrategyParams = SwapMMStrategyParams;

impl Lambda {
    pub fn new(
        market_depth: Arc<DashMap<String, MarketDepth>>,
        depth_instrument: Instrument,
        lambda_instance: LambdaInstance,
    ) -> Self {
        let strategy_params_sender = lambda_instance.strategy_params_request_sender.clone();
        Lambda {
            market_depth,
            depth_instrument,
            lambda_instance: Arc::new(RwLock::new(lambda_instance)),
            strategy_params_request_sender: strategy_params_sender,
        }
    }

    async fn get_strategy_params(&self) -> Result<StrategyParams, Box<dyn std::error::Error>> {
        let (params_request, mut result) = LambdaInstance::request_strategy_params_snapshot();
        self.strategy_params_request_sender.send(params_request).await;
        let params = result.await?;
        let transformed = serde_json::from_value::<StrategyParams>(params)?;
        Ok(transformed)
    }

    async fn update(&self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            log::info!(
                "lambda update open_orders: {:?}",
                self.depth_instrument.get_order_orders(true)
            );

            if let Some(md) = self.market_depth.get(self.depth_instrument.market.as_str()) {
                let targe_size = 31.0f32;

                let mut sum_bid_size = 0.0f32;
                let mut target_bid_price = 0.0f32;
                let mut target_bid_level = 0;
                for level in md.bids.iter() {
                    sum_bid_size += level.size;
                    if sum_bid_size >= targe_size {
                        target_bid_price = level.price;
                        break;
                    }
                    target_bid_level += 1;
                }

                let mut sum_ask_size = 0.0f32;
                let mut target_ask_price = 0.0f32;
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

                let params = self.get_strategy_params().await?;
                log::info!("s_params: {:?}", params);
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
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
        let mut lambda_instance = self.lambda_instance.write().await;
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
            result = lambda_instance.subscribe() => {
                panic!("lambda_instance subscribe_strategy_params panic: {:?}", result)
            }
        }
        Ok(())
    }
}
