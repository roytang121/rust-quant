#[cfg(test)]
pub mod common {
    use rust_quant::cache::{MarketDepthCache, OrderUpdateCache};
    use rust_quant::lambda::strategy::swap_mm::lambda::Lambda;
    use rust_quant::pubsub::simple_message_bus::RedisBackedMessageBus;
    use rust_quant::pubsub::SubscribeMarketDepthRequest;
    use std::error::Error;
    use std::sync::{Arc, Once};
    use std::time::Duration;
    use tokio::task::JoinHandle;

    static INIT: Once = Once::new();

    pub fn before_each() {
        INIT.call_once(|| {
            std::env::set_var("ENV", "development");
            std::env::set_var("RUST_LOG", "INFO");
            env_logger::init();
        });
    }

    pub fn spawn_thread_order_update_cache(order_update_cache: Arc<OrderUpdateCache>) {
        tokio::spawn(async move { order_update_cache.subscribe().await });
    }

    pub fn spawn_thread_message_bus(message_bus: Arc<RedisBackedMessageBus>) {
        tokio::spawn(async move { message_bus.subscribe().await });
    }

    pub fn spawn_thread_market_depth_cache(
        market_depth_cache: Arc<MarketDepthCache>,
        requests: Vec<SubscribeMarketDepthRequest>,
    ) {
        tokio::spawn(async move { market_depth_cache.subscribe(requests.as_slice()).await });
    }

    pub fn spawn_thread_lambda(lambda: Arc<Lambda>) -> JoinHandle<()> {
        tokio::spawn(async move {
            lambda.subscribe().await;
        })
    }

    pub async fn sleep(ms: u64) {
        tokio::time::sleep(Duration::from_millis(ms)).await;
    }
}
