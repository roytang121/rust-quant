




#[cfg(test)]
pub mod common {
    use rust_quant::cache::OrderUpdateCache;
    use rust_quant::pubsub::simple_message_bus::RedisBackedMessageBus;
    use std::sync::{Arc, Once};
    use std::time::Duration;

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

    pub async fn sleep(ms: u64) {
        tokio::time::sleep(Duration::from_millis(ms)).await;
    }
}
