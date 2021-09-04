use crate::cache::OrderUpdateCache;
use crate::model::constants::{Exchanges, PublishChannel};
use crate::model::{OrderFill, OrderRequest, OrderSide, OrderStatus, OrderType, OrderUpdate};

use crate::pubsub::PublishPayload;

use crate::pubsub::simple_message_bus::{
    MessageConsumer, RedisBackedMessageBus, TypedMessageConsumer,
};
pub use std::str::FromStr;

#[derive(Debug)]
pub struct Instrument<'r> {
    pub exchange: Exchanges,
    pub market: String,
    pub(crate) order_cache: &'r OrderUpdateCache,
    pub(crate) message_bus_sender: tokio::sync::mpsc::Sender<PublishPayload>,
}

pub struct InstrumentToken(pub Exchanges, pub String);

impl<'r> Instrument<'r> {
    pub fn new(
        exchange: Exchanges,
        market: &str,
        order_cache: &'r OrderUpdateCache,
        message_bus_sender: tokio::sync::mpsc::Sender<PublishPayload>,
    ) -> Self {
        Instrument {
            exchange,
            market: market.to_string(),
            order_cache,
            message_bus_sender,
        }
    }

    pub fn instrument_token(token: &str) -> InstrumentToken {
        let split: Vec<&str> = token.split(".").collect();
        let market = split.get(0).expect("Invalid token.").to_owned();
        let exchange = split.get(1).expect("Invalid token.").to_owned();
        let exchange = Exchanges::from_str(exchange).expect("Unknown exchange");
        InstrumentToken(exchange, market.to_string())
    }

    pub fn get_order_orders(&self, include_pending_cancels: bool) -> Vec<OrderUpdate> {
        let mut open_orders: Vec<OrderUpdate> = vec![];
        for ou in self.order_cache.cache.iter() {
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
        OrderRequest::send_order(
            &self.order_cache.cache,
            &self.message_bus_sender,
            order_request,
        )
        .await
    }

    pub async fn cancel_order(&self, client_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        OrderRequest::cancel_order(&self.order_cache.cache, &self.message_bus_sender, client_id)
            .await
    }

    pub async fn subscribe_order_fill<T>(&self, consumer: &T) -> anyhow::Result<()>
    where
        T: TypedMessageConsumer<OrderFill> + Sync,
    {
        let order_fill_filter =
            OrderFillFilter(self.exchange.to_owned(), self.market.to_owned(), consumer);
        RedisBackedMessageBus::subscribe_channels(
            vec![PublishChannel::OrderFill.as_ref()],
            &order_fill_filter,
        )
        .await
    }
}

pub struct OrderFillFilter<'r, ResultConsumer>(Exchanges, String, &'r ResultConsumer);

#[async_trait::async_trait]
impl<'r, ResultConsumer> MessageConsumer for OrderFillFilter<'r, ResultConsumer>
where
    ResultConsumer: TypedMessageConsumer<OrderFill> + Sync,
{
    async fn consume(&self, msg: &mut str) -> anyhow::Result<()> {
        let order_fill: OrderFill = serde_json::from_str(msg)?;
        if order_fill.exchange == self.0 && order_fill.market == self.1 {
            self.2.consume(order_fill).await
        } else {
            Ok(())
        }
    }
}
