#[macro_use]
extern crate log;

use std::error::Error;
use std::future::Future;
use std::time::Duration;

use rust_quant::ftx::market_depth::market_depth;
use rust_quant::ftx::ticker::ticker;
use rust_quant::lambda::{LambdaInstance, LambdaInstanceConfig};
use rust_quant::model::constants::Exchanges;
use rust_quant::pubsub::SubscribeMarketDepthRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();
    let instance_name = args
        .get(1)
        .expect("Missing parameter: instance")
        .to_string();
    let config = LambdaInstanceConfig::load(instance_name.as_str());
    rust_quant::lambda::engine(config).await;
    Ok(())
}
