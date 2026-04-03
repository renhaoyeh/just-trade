use serde::{Deserialize, Serialize};

/// A TWSE industry sector with stock count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sector {
    pub name: String,
    pub stock_count: i64,
}

/// A stock within a sector, with live quote data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorStock {
    pub symbol: String,
    pub name: String,
    pub sector: String,
    pub closing_price: Option<f64>,
    pub change: Option<f64>,
    pub trade_volume: Option<i64>,
}
