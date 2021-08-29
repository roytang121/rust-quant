use crate::model::market_data_model::MarketDepth;
use crate::model::{Instrument, OrderSide};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;

pub struct Lambda {
    market_depth: Arc<DashMap<String, MarketDepth>>,
    depth_instrument: Instrument,
}

impl Lambda {
    pub fn new(
        market_depth: Arc<DashMap<String, MarketDepth>>,
        depth_instrument: Instrument,
    ) -> Self {
        Lambda {
            market_depth,
            depth_instrument,
        }
    }

    async fn update(&self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            log::info!(
                "lambda update open_orders: {:?}",
                self.depth_instrument.get_order_orders(true)
            );
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    async fn add_order(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut counter = 0;
        loop {
            if counter <= 1000 {
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
            Err(err) = self.update() => {
                log::error!("lambda update panic: {}", err)
            }
            Err(err) = self.add_order() => {
                log::error!("lambda add_order panic: {}", err)
            }
            Err(err) = self.cancel_orders() => {
                log::error!("lambda cancel_orders panic: {}", err)
            }
        }
        Ok(())
    }
}
