#[macro_use]
extern crate log;

use std::error::Error;
use std::future::Future;
use std::time::Duration;

use rust_quant::ftx::market_depth::market_depth;
use rust_quant::ftx::ticker::ticker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    rust_quant::lambda::engine(vec![], 1).await;
    Ok(())
}
