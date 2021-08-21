#[macro_use]
extern crate log;

use std::error::Error;
use std::time::Duration;

use hello_rust::ftx::market_depth::market_depth;
use hello_rust::ftx::ticker::ticker;
use std::future::Future;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();
    let exchange = args
        .get(1)
        .expect("missing argument: exchange")
        .to_owned()
        .to_uppercase();

    let market_data_type = args
        .get(2)
        .expect("missing argument: market_data_type")
        .to_owned()
        .to_uppercase();

    let market = args
        .get(3)
        .expect("missing argument: market")
        .to_owned()
        .to_uppercase();

    loop {
        match exchange.as_str() {
            "FTX" => match market_data_type.as_str() {
                "TICKER" => {
                    if let Err(err) = ticker(market.as_str()).await {
                        handle_error(err)
                    }
                }
                "MARKETDEPTH" => {
                    if let Err(err) = market_depth(market.as_str()).await {
                        handle_error(err)
                    }
                }
                _ => {
                    panic!("Unsupported market_data_type: {}", market_data_type);
                }
            },
            _ => {
                panic!("Unsupported exchange: {}", exchange)
            }
        }
        info!("Sleeping 5 sec to restart...");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

fn handle_error(err: Box<dyn std::error::Error>) {
    error!("{}", err)
}
