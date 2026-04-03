use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockNews {
    pub title: String,
    pub summary: Option<String>,
    pub publisher: String,
    pub link: Option<String>,
    pub pub_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub symbol: String,
    pub short_name: Option<String>,
    pub exchange: Option<String>,
    pub quote_type: Option<String>,
}
