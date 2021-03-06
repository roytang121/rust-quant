use crate::model::constants::Exchanges;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
pub use std::str::FromStr;

// PriceLevel
#[derive(Serialize, Deserialize, Debug)]
pub struct PriceLevel {
    pub price: f64,
    pub size: f64,
}
impl ToString for PriceLevel {
    fn to_string(&self) -> String {
        self.price.to_string()
    }
}
impl Hash for PriceLevel {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.price.to_string().hash(state)
    }
}
impl PartialEq for PriceLevel {
    fn eq(&self, other: &Self) -> bool {
        self.price == other.price
    }
    fn ne(&self, other: &Self) -> bool {
        self.price != other.price
    }
}
impl Eq for PriceLevel {}
impl Clone for PriceLevel {
    fn clone(&self) -> Self {
        PriceLevel {
            price: self.price,
            size: self.size,
        }
    }
}
// end PriceLevel

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarketDepth {
    pub timestamp: i64,
    pub exchange: Exchanges,
    pub market: String,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}
