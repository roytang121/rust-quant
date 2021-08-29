#[macro_use]
extern crate log;

use rust_quant::ftx::FtxRestClient;
use rust_quant::model::constants::Exchanges;
use rust_quant::model::{OrderRequest, OrderSide, OrderType};
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let client = FtxRestClient::new();
    let client_ref = Arc::new(RwLock::new(client));
    let mut counter = 0;
    while counter <= 3 {
        let client_handle = client_ref.clone();
        tokio::spawn(async move {
            let client = client_handle.read().await;
            let order_request = OrderRequest {
                exchange: Exchanges::FTX,
                market: String::from("ETH-PERP"),
                side: OrderSide::Buy,
                price: 1000.0,
                size: 0.001,
                type_: OrderType::Limit,
                ioc: false,
                post_only: true,
            };
            let json = client.place_order(order_request).await.unwrap();
            log::info!("{}", json);
            tokio::time::sleep(Duration::from_millis(10)).await;
            if let Some(clientId) = json["result"]["id"].as_i64() {
                let json = client.cancel_order(clientId).await.unwrap();
                log::info!("{}", json);
            }
        });
        counter+=1;
    }
    tokio::time::sleep(Duration::from_secs(10)).await;
    Ok(())
}
