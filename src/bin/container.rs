#[macro_use]
extern crate log;

use std::error::Error;

use rust_quant::lambda::{GenericLambdaInstanceConfig, LambdaEngine};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();
    let instance_name = args
        .get(1)
        .expect("Missing parameter: instance")
        .to_string();
    let config = GenericLambdaInstanceConfig::load(instance_name.as_str());
    let engine = LambdaEngine::init(config).await;
    // rust_quant::lambda::engine(config).await;
    engine.subscribe().await;
    Ok(())
}
