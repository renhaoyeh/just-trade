use serde::Deserialize;

// ── v8/finance/chart response ──

#[derive(Debug, Deserialize)]
pub struct ChartResponse {
    pub chart: ChartResultWrapper,
}

#[derive(Debug, Deserialize)]
pub struct ChartResultWrapper {
    pub result: Option<Vec<ChartData>>,
    pub error: Option<YahooError>,
}

#[derive(Debug, Deserialize)]
pub struct YahooError {
    pub code: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChartData {
    pub timestamp: Option<Vec<i64>>,
    pub meta: ChartMeta,
    pub indicators: Indicators,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartMeta {
    pub currency: Option<String>,
    pub symbol: Option<String>,
    pub exchange_name: Option<String>,
    pub regular_market_price: Option<f64>,
    pub previous_close: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct Indicators {
    pub quote: Option<Vec<QuoteData>>,
    pub adjclose: Option<Vec<AdjCloseData>>,
}

#[derive(Debug, Deserialize)]
pub struct QuoteData {
    pub open: Option<Vec<Option<f64>>>,
    pub high: Option<Vec<Option<f64>>>,
    pub low: Option<Vec<Option<f64>>>,
    pub close: Option<Vec<Option<f64>>>,
    pub volume: Option<Vec<Option<i64>>>,
}

#[derive(Debug, Deserialize)]
pub struct AdjCloseData {
    pub adjclose: Option<Vec<Option<f64>>>,
}

// ── v10/finance/quoteSummary response ──

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteSummaryResponse {
    pub quote_summary: Option<QuoteSummaryResult>,
}

#[derive(Debug, Deserialize)]
pub struct QuoteSummaryResult {
    pub result: Option<Vec<QuoteSummaryData>>,
    pub error: Option<YahooError>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteSummaryData {
    pub summary_detail: Option<SummaryDetail>,
    pub price: Option<PriceData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryDetail {
    pub market_cap: Option<YahooValue>,
    pub trailing_pe: Option<YahooValue>,
    pub forward_pe: Option<YahooValue>,
    pub dividend_yield: Option<YahooValue>,
    pub fifty_two_week_high: Option<YahooValue>,
    pub fifty_two_week_low: Option<YahooValue>,
    pub fifty_day_average: Option<YahooValue>,
    pub two_hundred_day_average: Option<YahooValue>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceData {
    pub short_name: Option<String>,
    pub long_name: Option<String>,
    pub currency: Option<String>,
    pub exchange_name: Option<String>,
}

/// Yahoo wraps many numeric values as { "raw": 123.45, "fmt": "123.45" }
#[derive(Debug, Deserialize)]
pub struct YahooValue {
    pub raw: Option<f64>,
    pub fmt: Option<String>,
}

// ── v1/finance/search response ──

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub quotes: Option<Vec<SearchQuote>>,
    pub news: Option<Vec<SearchNewsItem>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuote {
    pub symbol: Option<String>,
    pub short_name: Option<String>,
    pub exchange: Option<String>,
    pub quote_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchNewsItem {
    pub title: Option<String>,
    pub publisher: Option<String>,
    pub link: Option<String>,
    pub provider_publish_time: Option<i64>,
}
