use crate::cache::MarketDepthCache;
use crate::lambda::lambda_instance::{
    LambdaStrategyParamsRequest, LambdaStrategyParamsRequestSender,
};
use crate::lambda::strategy::swap_mm::params::{
    SwapMMInitParams, SwapMMStrategyParams, SwapMMStrategyStateStruct,
};
use crate::lambda::{
    GenericLambdaInstanceConfig, LambdaInstance, LambdaInstanceConfig, LambdaState,
};

use crate::model::{
    Instrument, InstrumentSymbol, MeasurementCache, OrderFill, OrderSide, OrderStatus, OrderType,
    OrderUpdate,
};
use crate::pubsub::PublishPayload;

use rocket::tokio::sync::mpsc::error::SendError;

use crate::cache::OrderUpdateCache;

use crate::pubsub::simple_message_bus::TypedMessageConsumer;
use dashmap::mapref::one::RefMut;
use dashmap::{DashMap, DashSet};

use std::collections::hash_map::RandomState;

use std::sync::Arc;
use std::time::Duration;

type InitParams = SwapMMInitParams;
type StrategyParams = SwapMMStrategyParams;
type StrategyState = SwapMMStrategyStateStruct;

pub struct Lambda {
    market_depth: Arc<MarketDepthCache>,
    depth_instrument: Arc<Instrument>,
    hedge_instrument: Arc<Instrument>,
    lambda_instance: LambdaInstance,
    strategy_state: Arc<DashMap<String, StrategyState>>,
    strategy_params_request_sender: LambdaStrategyParamsRequestSender,
    measurement_cache: Arc<MeasurementCache>,
}

impl Lambda {
    pub fn new(
        instance_config: GenericLambdaInstanceConfig,
        market_depth: Arc<MarketDepthCache>,
        order_cache: Arc<OrderUpdateCache>,
        message_bus_sender: tokio::sync::mpsc::Sender<PublishPayload>,
        measurement_cache: Arc<MeasurementCache>,
    ) -> Self {
        let lambda_instance =
            LambdaInstance::new(LambdaInstanceConfig::load(instance_config.name.as_str()));
        let strategy_params_sender = lambda_instance.strategy_params_request_sender.clone();
        let init_params =
            serde_json::from_value::<InitParams>(lambda_instance.init_params.clone()).unwrap();

        // depth_instrument
        let depth_instrument_token =
            Instrument::instrument_symbol(init_params.depth_symbol.as_str());
        let depth_instrument = match depth_instrument_token {
            InstrumentSymbol(exchange, market) => Arc::new(Instrument {
                exchange,
                market,
                order_cache: order_cache.clone(),
                message_bus_sender: message_bus_sender.clone(),
            }),
        };
        // hedge_instrument
        let hedge_instrument_token =
            Instrument::instrument_symbol(init_params.hedge_symbol.as_str());
        let hedge_instrument = match hedge_instrument_token {
            InstrumentSymbol(exchange, market) => Arc::new(Instrument {
                exchange,
                market,
                order_cache: order_cache.clone(),
                message_bus_sender: message_bus_sender.clone(),
            }),
        };

        let strategy_state = DashMap::new();
        strategy_state.insert(String::default(), StrategyState::default());

        Lambda {
            market_depth,
            depth_instrument,
            hedge_instrument,
            lambda_instance,
            strategy_state: Arc::new(strategy_state),
            strategy_params_request_sender: strategy_params_sender,
            measurement_cache,
        }
    }

    fn get_strategy_params(&self) -> StrategyParams {
        match self.lambda_instance.get_strategy_params_clone() {
            None => {
                panic!("StrategyParams could not be None")
            }
            Some(value) => return serde_json::from_value::<StrategyParams>(value).unwrap(),
        }
    }

    fn get_strategy_state(&self) -> StrategyState {
        let state = self.strategy_state.get(String::default().as_str()).unwrap();
        state.value().clone()
    }

    fn write_strategy_state(&self) -> Option<RefMut<'_, String, StrategyState, RandomState>> {
        self.strategy_state.get_mut(String::default().as_str())
    }
    //
    // async fn write_strategy_state(&self) -> RwLockWriteGuard<'_, StrategyState> {
    //     self.strategy_state.write().await
    // }

    async fn publish_state(&self) -> Result<(), SendError<LambdaStrategyParamsRequest>> {
        let strategy_state = self.get_strategy_state();
        let value = serde_json::to_value(strategy_state).unwrap();
        LambdaStrategyParamsRequest::request_set_state(&self.strategy_params_request_sender, value)
            .await
    }

    async fn period_publish_state(&self) -> anyhow::Result<()> {
        loop {
            match self.publish_state().await {
                Ok(_) => {}
                Err(err) => error!("Error publishing lambda state: {}", err),
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    async fn period_update(&self) -> anyhow::Result<()> {
        loop {
            // info!("lambda update in thread_id: {:?}", std::thread::current().id());
            if let Some(md) = self
                .market_depth
                .get_clone(self.depth_instrument.market.as_str())
            {
                let target_size = 31.0f64;

                let mut sum_bid_size = 0.0f64;
                let mut target_bid_price = 0.0f64;
                let mut target_bid_level = 0i64;
                for level in md.bids.iter() {
                    sum_bid_size += level.size;
                    if sum_bid_size >= target_size {
                        target_bid_price = level.price;
                        break;
                    }
                    target_bid_level += 1;
                }

                let mut sum_ask_size = 0.0f64;
                let mut target_ask_price = 0.0f64;
                let mut target_ask_level = 0i64;
                for level in md.asks.iter() {
                    sum_ask_size += level.size;
                    if sum_ask_size >= target_size {
                        target_ask_price = level.price;
                        break;
                    }
                    target_ask_level += 1;
                }

                // debug!("bid[{}]: {}", target_bid_level, target_bid_price);
                // debug!("ask[{}]: {}", target_ask_level, target_ask_price);

                // measurement
                let bid_basis_bp =
                    ((target_bid_price - md.bids[0].price) / md.bids[0].price) * 10000.0;
                let ask_basis_bp =
                    ((target_ask_price - md.asks[0].price) / md.bids[0].price) * 10000.0;

                let open_bid_orders = self.depth_instrument.get_open_buy_orders(true);
                let open_bid_cnt = open_bid_orders.len();
                let open_bid = open_bid_orders.get(0);

                if let Some(mut state) = self.write_strategy_state() {
                    state.target_bid_px = Some(target_bid_price);
                    state.target_ask_px = Some(target_ask_price);
                    state.target_bid_level = Some(target_bid_level);
                    state.target_ask_level = Some(target_ask_level);
                    state.bid_basis_bp = Some(bid_basis_bp);
                    state.ask_basis_bp = Some(ask_basis_bp);
                    state.open_bid_cnt = Some(open_bid_cnt);
                    if let Some(bid_0) = md.bids.get(0) {
                        state.depth_bid_px = Some(bid_0.price)
                    }
                    if let Some(ask_0) = md.asks.get(0) {
                        state.depth_ask_px = Some(ask_0.price)
                    }
                    match open_bid {
                        None => {}
                        Some(open_bid) => {
                            state.open_bid_px = Some(open_bid.price);
                        }
                    }
                }
            } else {
                if let Some(mut state) = self.strategy_state.get_mut(String::default().as_str()) {
                    state.depth_bid_px = None;
                    state.depth_ask_px = None;
                    state.bid_basis_bp = None;
                    state.ask_basis_bp = None;
                    state.target_bid_px = None;
                    state.target_bid_level = None;
                    state.target_ask_px = None;
                    state.target_ask_level = None
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    async fn period_cancel_orders(&self) -> anyhow::Result<()> {
        loop {
            self.cancel_orders().await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    async fn period_run_trading(&self) -> anyhow::Result<()> {
        loop {
            if self.should_run_trading() {
                self.add_order().await?;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    fn should_run_trading(&self) -> bool {
        let params = self.get_strategy_params();
        return match params.state {
            LambdaState::Init
            | LambdaState::Paused
            | LambdaState::Stopped
            | LambdaState::AutoPaused => false,
            LambdaState::Live => true,
        };
    }

    async fn cancel_orders(&self) {
        let open_buy_orders = self.depth_instrument.get_open_buy_orders(false);
        if open_buy_orders.len() > 1 {
            error!("depth_instrument open_buy_orders > 1");
        }

        let state = self.get_strategy_state();
        if let (Some(open_bid_px), Some(target_bid_px)) = (state.open_bid_px, state.target_bid_px) {
            if !open_buy_orders.is_empty() && open_bid_px != target_bid_px {
                for order in open_buy_orders {
                    self.depth_instrument
                        .cancel_order(order.client_id.unwrap_or_default().as_str());
                    if let Some(mut write_state) =
                        self.strategy_state.get_mut(String::default().as_str())
                    {
                        write_state.enable_buy = true;
                    }
                }
            }
        } else { // missing required states
            if open_buy_orders.is_empty() { // if no open orders
                if let Some(mut write_state) =
                self.strategy_state.get_mut(String::default().as_str())
                {
                    write_state.enable_buy = true;
                }
            }
        };
        drop(state)
    }

    async fn add_order(&self) -> anyhow::Result<()> {
        let open_orders = self.depth_instrument.get_open_orders(true);
        let params = self.get_strategy_params();
        if open_orders.is_empty() {
            let state = self.get_strategy_state();
            if let (Some(enable_buy), Some(_depth_bid_px), Some(target_bid_px), Some(target_bid_level), Some(bid_basis_bp)) = (
                state.enable_buy,
                state.depth_bid_px,
                state.target_bid_px,
                state.target_bid_level,
                state.bid_basis_bp,
            ) {
                if enable_buy && target_bid_level >= params.min_level && bid_basis_bp <= -params.min_basis {
                    self.depth_instrument
                        .send_order(
                            OrderSide::Buy,
                            target_bid_px,
                            params.base_size,
                            OrderType::Limit,
                        )
                        .await;
                }
            }
        };
        Ok(())
    }

    pub async fn subscribe(&self) -> Result<(), Box<dyn std::error::Error>> {
        let hedger =
            SimpleHedger::new(self.depth_instrument.clone(), self.hedge_instrument.clone());
        tokio::select! {
            result = self.period_update() => {
                panic!("lambda update panic: {:?}", result)
            }
            result = self.period_run_trading() => {
                panic!("lambda period_run_trading panic: {:?}", result)
            }
            result = self.period_publish_state() => {
                panic!("period_publish_state panic: {:?}", result)
            }
            result = self.period_cancel_orders() => {
                panic!("lambda cancel_orders panic: {:?}", result)
            }
            result = self.lambda_instance.subscribe() => {
                panic!("lambda_instance subscribe_strategy_params panic: {:?}", result)
            }
            result = hedger.subscribe() => {
                panic!("depth_instrument.subscribe_order_fill panic: {:?}", result)
            }
        }
    }
}
pub struct SimpleHedger {
    pub hedge_orders: Arc<DashSet<String>>,
    pub depth_instrument: Arc<Instrument>,
    pub hedge_instrument: Arc<Instrument>,
}
impl SimpleHedger {
    pub fn new(depth_instrument: Arc<Instrument>, hedge_instrument: Arc<Instrument>) -> Self {
        SimpleHedger {
            hedge_orders: Arc::new(DashSet::new()),
            depth_instrument,
            hedge_instrument,
        }
    }
    pub async fn subscribe(&self) -> anyhow::Result<()> {
        log::info!("simple_hedger subscribing...");
        tokio::select! {
            Err(err) = self.depth_instrument.subscribe_order_fill(self) => {
                error!("Hedge subscribe_order_fill: {}", err);
            },
            Err(err) = self.hedge_instrument.subscribe_order_update(self) => {
                error!("Hedge subscribe_order_update: {}", err);
            }
        }
        Err(anyhow!("Hedge subscribe uncaught"))
    }
}
#[async_trait::async_trait]
impl TypedMessageConsumer<OrderFill> for SimpleHedger {
    async fn consume(&self, order_fill: OrderFill) -> anyhow::Result<()> {
        let hedge_order = self.hedge_instrument.send_order(
            OrderSide::flip_side(&order_fill.side),
            0.0,
            order_fill.size.clone(),
            OrderType::Market,
        );
        if let Some(client_id) = hedge_order.await? {
            self.hedge_orders.insert(client_id);
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl TypedMessageConsumer<OrderUpdate> for SimpleHedger {
    async fn consume(&self, order_update: OrderUpdate) -> anyhow::Result<()> {
        if let Some(client_id) = order_update.client_id {
            match order_update.status {
                OrderStatus::New => {}
                OrderStatus::Open => {}
                OrderStatus::PendingNew => {}
                OrderStatus::PendingCancel => {}
                OrderStatus::Closed => {
                    if order_update.filledSize >= order_update.size {
                        self.hedge_orders.remove(client_id.as_str());
                    }
                }
                OrderStatus::Failed => {
                    self.hedge_orders.remove(client_id.as_str());
                    // resend order
                    self.hedge_instrument.send_order(
                        order_update.side.clone(),
                        0.0,
                        order_update.size.clone(),
                        OrderType::Market,
                    );
                }
            }
        };
        Ok(())
    }
}
