use crate::cache::OrderUpdateCache;
use crate::model::constants::{Exchanges, PublishChannel};
use crate::model::{OrderFill, OrderRequest, OrderSide, OrderStatus, OrderType, OrderUpdate};

use crate::pubsub::PublishPayload;

use crate::pubsub::simple_message_bus::{
    MessageConsumer, RedisBackedMessageBus, TypedMessageConsumer,
};
pub use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug)]
pub struct Instrument {
    pub exchange: Exchanges,
    pub market: String,
    pub order_cache: Arc<OrderUpdateCache>,
    pub message_bus_sender: tokio::sync::mpsc::Sender<PublishPayload>,
}

pub struct InstrumentSymbol(pub Exchanges, pub String);

impl FromStr for InstrumentSymbol {
    type Err = ();

    fn from_str(token: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = token.split(".").collect();
        let market = split.get(0).expect("Invalid token.").to_owned();
        let exchange = split.get(1).expect("Invalid token.").to_owned();
        let exchange = Exchanges::from_str(exchange).expect("Unknown exchange");
        Ok(InstrumentSymbol(exchange, market.to_string()))
    }
}

impl Instrument {
    pub fn new(
        exchange: Exchanges,
        market: &str,
        order_cache: Arc<OrderUpdateCache>,
        message_bus_sender: tokio::sync::mpsc::Sender<PublishPayload>,
    ) -> Self {
        Instrument {
            exchange,
            market: market.to_string(),
            order_cache,
            message_bus_sender,
        }
    }

    pub fn instrument_symbol(token: &str) -> InstrumentSymbol {
        let split: Vec<&str> = token.split(".").collect();
        let market = split.get(0).expect("Invalid token.").to_owned();
        let exchange = split.get(1).expect("Invalid token.").to_owned();
        let exchange = Exchanges::from_str(exchange).expect("Unknown exchange");
        InstrumentSymbol(exchange, market.to_string())
    }

    pub fn get_open_orders(&self, include_pending_cancels: bool) -> Vec<OrderUpdate> {
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
                    OrderStatus::Closed | OrderStatus::Failed => {}
                }
            }
        }
        open_orders
    }

    pub fn get_open_buy_orders(&self, include_pending_cancels: bool) -> Vec<OrderUpdate> {
        let open_orders = self.get_open_orders(include_pending_cancels);
        open_orders
            .into_iter()
            .filter(|order| order.side == OrderSide::Buy)
            .map(|order| order.clone())
            .collect()
    }

    pub fn get_open_sell_orders(&self, include_pending_cancels: bool) -> Vec<OrderUpdate> {
        let open_orders = self.get_open_orders(include_pending_cancels);
        open_orders
            .into_iter()
            .filter(|order| order.side == OrderSide::Sell)
            .map(|order| order.clone())
            .collect()
    }

    pub async fn send_order(
        &self,
        side: OrderSide,
        price: f64,
        size: f64,
        type_: OrderType,
    ) -> anyhow::Result<Option<String>> {
        let mut order_request = OrderRequest {
            exchange: self.exchange.clone(),
            market: self.market.clone(),
            side,
            price,
            size,
            type_: type_,
            ioc: false,
            post_only: true,
            client_id: None,
        };
        let client_id = order_request.generate_client_id().clone();
        OrderRequest::send_order(
            &self.order_cache.cache,
            &self.message_bus_sender,
            order_request,
        )
        .await?;
        Ok(client_id)
    }

    pub async fn cancel_order(&self, client_id: &str) -> anyhow::Result<()> {
        OrderRequest::cancel_order(
            &self.order_cache.cache,
            &self.message_bus_sender,
            client_id,
            self.market.as_str(),
        )
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

    pub async fn subscribe_order_update<T>(&self, consumer: &T) -> anyhow::Result<()>
    where
        T: TypedMessageConsumer<OrderUpdate> + Sync,
    {
        let order_update_filter =
            OrderUpdateFilter(self.exchange.to_owned(), self.market.to_owned(), consumer);
        RedisBackedMessageBus::subscribe_channels(
            vec![PublishChannel::OrderUpdate.as_ref()],
            &order_update_filter,
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
    async fn consume(&self, msg: &[u8]) -> anyhow::Result<()> {
        let order_fill: OrderFill = serde_json::from_slice(msg)?;
        if order_fill.exchange == self.0 && order_fill.market == self.1 {
            self.2.consume(order_fill).await
        } else {
            Ok(())
        }
    }
}

pub struct OrderUpdateFilter<'r, ResultConsumer>(Exchanges, String, &'r ResultConsumer);

#[async_trait::async_trait]
impl<'r, ResultConsumer> MessageConsumer for OrderUpdateFilter<'r, ResultConsumer>
where
    ResultConsumer: TypedMessageConsumer<OrderUpdate> + Sync,
{
    async fn consume(&self, msg: &[u8]) -> anyhow::Result<()> {
        let order_update: OrderUpdate = serde_json::from_slice(msg)?;
        if order_update.exchange == self.0 && order_update.market == self.1 {
            self.2.consume(order_update).await
        } else {
            Ok(())
        }
    }
}
