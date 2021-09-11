#[cfg(test)]
use mockall::{automock, mock, predicate::*};
#[cfg(test)]
mod test_common;

#[cfg(test)]
mod order_update_cache_test {
    use super::*;
    use ntest::timeout;
    use rocket::tokio::task::JoinHandle;
    use rust_quant::cache::OrderUpdateCache;
    use rust_quant::model::constants::PublishChannel::OrderUpdate;
    use rust_quant::model::constants::{Exchanges, PublishChannel};
    use rust_quant::model::{OrderSide, OrderStatus, OrderType};
    use rust_quant::pubsub::simple_message_bus::{MessageBusSender, RedisBackedMessageBus};
    use rust_quant::pubsub::PublishPayload;
    use std::str::FromStr;
    use std::sync::Arc;
    use std::time::Duration;
    use test_common::common::*;

    #[tokio::test]
    async fn can_init() {
        before_each();
        let order_update_cache = OrderUpdateCache::new();
        assert_eq!(order_update_cache.cache.len(), 0)
    }

    #[tokio::test]
    async fn insert_cache() {
        before_each();
        let order_update_cache = Arc::new(OrderUpdateCache::new());
        spawn_thread_order_update_cache(order_update_cache.clone());
        sleep(100).await;

        let message_bus = RedisBackedMessageBus::new().await.unwrap();

        let mut order_update = rust_quant::model::OrderUpdate::default();
        order_update.client_id = Some("order-1".to_string());

        message_bus
            .publish(PublishChannel::OrderUpdate.as_ref(), &order_update)
            .await;

        sleep(100).await;
        assert_eq!(order_update_cache.cache.len(), 1)
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn massive_insert_cache() {
        before_each();
        let order_update_cache = Arc::new(OrderUpdateCache::new());
        spawn_thread_order_update_cache(order_update_cache.clone());
        sleep(100).await;

        let message_bus = Arc::new(RedisBackedMessageBus::new().await.unwrap());
        spawn_thread_message_bus(message_bus.clone());
        sleep(100).await;

        let mut count = 10000;
        for i in 0..count {
            let mut order_update = rust_quant::model::OrderUpdate::default();
            order_update.client_id = Some(format!("order-{}", i).to_string());
            let payload = PublishPayload {
                channel: PublishChannel::OrderUpdate.to_string(),
                payload: RedisBackedMessageBus::pack_json(&order_update).unwrap(),
            };
            message_bus.publish_spawn(payload.channel.to_string(), order_update);
        }

        sleep(2000).await;
        assert_eq!(order_update_cache.cache.len(), 10000)
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn massive_cancel() {
        before_each();
        let order_update_cache = Arc::new(OrderUpdateCache::new());
        spawn_thread_order_update_cache(order_update_cache.clone());
        sleep(100).await;

        let message_bus = Arc::new(RedisBackedMessageBus::new().await.unwrap());
        spawn_thread_message_bus(message_bus.clone());
        sleep(100).await;

        let message_bus_2 = message_bus.clone();
        let mut count = 10000;
        for i in 0..count {
            let message_bus_ref = message_bus.clone();
            tokio::spawn(async move {
                let mut order_update = rust_quant::model::OrderUpdate::default();
                order_update.client_id = Some(format!("order-{}", i).to_string());
                let payload = PublishPayload {
                    channel: PublishChannel::OrderUpdate.to_string(),
                    payload: RedisBackedMessageBus::pack_json(&order_update).unwrap(),
                };
                message_bus_ref.publish_spawn(payload.channel.to_string(), order_update);
            });
            let message_bus_ref = message_bus.clone();
            tokio::spawn(async move {
                let mut order_update = rust_quant::model::OrderUpdate::default();
                order_update.client_id = Some(format!("order-{}", i).to_string());
                order_update.status = OrderStatus::Closed;
                let payload = PublishPayload {
                    channel: PublishChannel::OrderUpdate.to_string(),
                    payload: RedisBackedMessageBus::pack_json(&order_update).unwrap(),
                };
                message_bus_ref.publish_spawn(payload.channel.to_string(), order_update);
            });
        }

        sleep(4000).await;
        assert_eq!(order_update_cache.cache.len(), 0)
    }
}
