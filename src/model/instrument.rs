use crate::core::orders::OrderUpdateService;
use crate::model::constants::Exchanges;
use crate::model::{
    OrderRequest, OrderSide, OrderStatus, OrderType, OrderUpdate, OrderUpdateCache,
};
use crate::pubsub::simple_message_bus::RedisBackedMessageBus;
use crate::pubsub::PublishPayload;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Instrument {
    pub exchange: Exchanges,
    pub market: String,
    order_cache: OrderUpdateCache,
    message_bus_sender: tokio::sync::mpsc::Sender<PublishPayload>,
}

impl Instrument {
    pub fn new(
        exchange: Exchanges,
        market: &str,
        order_cache: OrderUpdateCache,
        message_bus_sender: tokio::sync::mpsc::Sender<PublishPayload>,
    ) -> Self {
        Instrument {
            exchange,
            market: market.to_string(),
            order_cache,
            message_bus_sender,
        }
    }

    pub fn get_order_orders(&self, include_pending_cancels: bool) -> Vec<OrderUpdate> {
        let mut open_orders: Vec<OrderUpdate> = vec![];
        for ou in self.order_cache.iter() {
            if ou.exchange == self.exchange && ou.market == self.market {
                match ou.status {
                    OrderStatus::New | OrderStatus::Open | OrderStatus::PendingNew => {
                        open_orders.push(Clone::clone(&ou.value()));
                    }
                    OrderStatus::PendingCancel => {
                        if include_pending_cancels {
                            open_orders.push(Clone::clone(&ou.value()));
                        }
                    }
                    OrderStatus::Closed => {}
                }
            }
        }
        open_orders
    }

    pub async fn send_order(
        &self,
        side: OrderSide,
        price: f64,
        size: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut order_request = OrderRequest {
            exchange: self.exchange.clone(),
            market: self.market.clone(),
            side,
            price,
            size,
            type_: OrderType::Limit,
            ioc: false,
            post_only: true,
            client_id: None,
        };
        order_request.generate_client_id();
        OrderRequest::send_order(&self.order_cache, &self.message_bus_sender, order_request).await
    }

    pub async fn cancel_order(&self, client_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        OrderRequest::cancel_order(&self.order_cache, &self.message_bus_sender, client_id).await
    }
}
