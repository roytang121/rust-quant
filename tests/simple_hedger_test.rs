#[cfg(test)]
use rust_quant::lambda::SimpleHedger;
use std::sync::Arc;

#[cfg(test)]
mod test_common;

#[cfg(test)]
fn spawn_thread_hedger_subscribe(hedger: Arc<SimpleHedger>) {
    tokio::spawn(async move {
        log::info!("spawn_thread_hedger_subscribe");
        hedger.subscribe().await
    });
}

#[cfg(test)]
mod simple_hedger_test {
    use super::*;
    use rust_quant::cache::OrderUpdateCache;
    use rust_quant::lambda::SimpleHedger;
    use rust_quant::model::constants::Exchanges;
    use rust_quant::model::{Instrument, OrderFill, OrderSide};
    use rust_quant::pubsub::simple_message_bus::RedisBackedMessageBus;
    use std::sync::Arc;
    use test_common::common::*;

    #[tokio::test]
    async fn can_init() {
        before_each();
        let message_bus = Arc::new(RedisBackedMessageBus::new().await.unwrap());
        spawn_thread_message_bus(message_bus.clone());
        sleep(100).await;

        let order_update_cache = Arc::new(OrderUpdateCache::new());
        spawn_thread_order_update_cache(order_update_cache.clone());
        sleep(100).await;

        let depth_instrument = Arc::new(Instrument {
            exchange: Exchanges::Unknown,
            market: "ETH-PERP".to_string(),
            order_cache: order_update_cache.clone(),
            message_bus_sender: message_bus.publish_tx.clone(),
        });

        let hedge_instrument = Arc::new(Instrument {
            exchange: Exchanges::Unknown,
            market: "ETH/USD".to_string(),
            order_cache: order_update_cache.clone(),
            message_bus_sender: message_bus.publish_tx.clone(),
        });

        let hedger = Arc::new(SimpleHedger::new(
            depth_instrument.clone(),
            hedge_instrument.clone(),
        ));
        spawn_thread_hedger_subscribe(hedger.clone());
        sleep(100).await;

        let count = 10;
        for _i in 0..count {
            let mut order_fill = OrderFill::default();
            order_fill.exchange = Exchanges::Unknown;
            order_fill.market = "ETH-PERP".to_string();
            order_fill.side = OrderSide::Buy;
            message_bus.publish_spawn(
                rust_quant::model::constants::PublishChannel::OrderFill.to_string(),
                order_fill,
            );
        }

        sleep(100).await;

        assert_eq!(hedger.hedge_orders.len(), 10);
    }
}
