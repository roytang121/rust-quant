use serde::{Deserialize, Serialize};

use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize)]
// #[serde(deny_unknown_fields)]
pub struct OrderbookUpdate {
    pub time: f64,
    pub checksum: u64,
    pub bids: Vec<Vec<f32>>,
    pub asks: Vec<Vec<f32>>,
    pub action: String,
}
#[derive(Serialize, Deserialize)]
// #[serde(deny_unknown_fields)]
pub struct OrderbookMessage {
    pub channel: String,
    pub market: String,
    pub data: OrderbookUpdate,
    pub type_: String,
}

// PriceLevel
#[derive(Serialize, Deserialize)]
pub struct PriceLevel {
    pub price_f: f32,
    pub size: f32,
}
impl ToString for PriceLevel {
    fn to_string(&self) -> String {
        self.price_f.to_string()
    }
}
impl Hash for PriceLevel {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.price_f.to_string().hash(state)
    }
}
impl PartialEq for PriceLevel {
    fn eq(&self, other: &Self) -> bool {
        self.price_f == other.price_f
    }
    fn ne(&self, other: &Self) -> bool {
        self.price_f != other.price_f
    }
}
impl Eq for PriceLevel {}
impl Clone for PriceLevel {
    fn clone(&self) -> Self {
        PriceLevel {
            price_f: self.price_f,
            size: self.size,
        }
    }
}
// end PriceLevel

#[derive(Serialize, Deserialize)]
pub struct OrderBookSnapshot {
    pub timestamp: u128,
    pub latency: f64,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}

fn main() {}
