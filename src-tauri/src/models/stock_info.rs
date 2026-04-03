use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockInfo {
    pub symbol: String,
    pub short_name: Option<String>,
    pub long_name: Option<String>,
    pub market_cap: Option<i64>,
    pub pe_ratio: Option<f64>,
    pub forward_pe: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub fifty_two_week_high: Option<f64>,
    pub fifty_two_week_low: Option<f64>,
    pub fifty_day_average: Option<f64>,
    pub two_hundred_day_average: Option<f64>,
    pub currency: Option<String>,
    pub exchange: Option<String>,
}
