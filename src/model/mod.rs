pub mod constants;
mod instrument;
pub mod market_data_model;
mod measurement_cache;
mod order_data_model;

pub use instrument::{Instrument, InstrumentSymbol, OrderFillFilter};
pub use measurement_cache::*;
pub use order_data_model::{
    CancelOrderRequest, OrderFill, OrderRequest, OrderSide, OrderStatus, OrderType, OrderUpdate,
    OrderUpdateCacheInner,
};

pub mod global_measurement {
    use crate::model::{Measurement, TSOptions};

    pub static ORDER_LATENCY: Measurement = Measurement::OrderLatency { options: TSOptions { retention: 0 } };
    pub static TO_ACK: Measurement = Measurement::ToAck { options: TSOptions { retention: 0 }};
}
