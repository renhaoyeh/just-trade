use chrono::{NaiveDate, TimeZone, Utc};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

use crate::models::stock_info::StockInfo;
use crate::models::stock_news::{SearchResult, StockNews};
use crate::models::stock_price::NewStockPrice;
use crate::models::yahoo_response::{ChartResponse, SearchResponse};

const YAHOO_CHART_URL: &str = "https://query1.finance.yahoo.com/v8/finance/chart";
const YAHOO_QUOTE_SUMMARY_URL: &str = "https://query1.finance.yahoo.com/v10/finance/quoteSummary";
const YAHOO_SEARCH_URL: &str = "https://query1.finance.yahoo.com/v1/finance/search";
const YAHOO_CRUMB_URL: &str = "https://query2.finance.yahoo.com/v1/test/getcrumb";
const YAHOO_COOKIE_URL: &str = "https://fc.yahoo.com/";

const MAX_RETRIES: u32 = 3;
const RETRY_BASE_DELAY_MS: u64 = 500;

#[derive(Debug, thiserror::Error)]
pub enum YahooError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Yahoo API error: {0}")]
    Api(String),
    #[error("No data found for symbol: {0}")]
    NoData(String),
    #[error("Failed to acquire crumb")]
    CrumbFailed,
}

pub struct YahooClient {
    client: reqwest::Client,
    crumb: Arc<RwLock<Option<String>>>,
    lang: String,
    region: String,
}

impl YahooClient {
    pub fn new(lang: &str, region: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            ),
        );

        let client = reqwest::Client::builder()
            .cookie_store(true)
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            crumb: Arc::new(RwLock::new(None)),
            lang: lang.to_string(),
            region: region.to_string(),
        }
    }

    /// Ensure we have a valid crumb token, fetching one if needed.
    async fn ensure_crumb(&self) -> Result<String, YahooError> {
        // Check cached crumb first
        {
            let cached = self.crumb.read().await;
            if let Some(ref c) = *cached {
                return Ok(c.clone());
            }
        }

        // Fetch new crumb
        let crumb = self.fetch_crumb().await?;
        let mut w = self.crumb.write().await;
        *w = Some(crumb.clone());
        Ok(crumb)
    }

    /// Invalidate the cached crumb so next call will re-fetch.
    async fn invalidate_crumb(&self) {
        let mut w = self.crumb.write().await;
        *w = None;
    }

    async fn fetch_crumb(&self) -> Result<String, YahooError> {
        // Step 1: Hit fc.yahoo.com to get cookies
        let _ = self.client.get(YAHOO_COOKIE_URL).send().await?;

        // Step 2: Get crumb using the cookies
        let resp = self.client.get(YAHOO_CRUMB_URL).send().await?;

        if !resp.status().is_success() {
            return Err(YahooError::CrumbFailed);
        }

        let crumb = resp.text().await?;
        if crumb.is_empty() || crumb.contains("<!DOCTYPE") {
            return Err(YahooError::CrumbFailed);
        }

        Ok(crumb)
    }

    /// Retry wrapper with crumb refresh on 401/403.
    async fn request_with_retry(
        &self,
        url: &str,
    ) -> Result<reqwest::Response, YahooError> {
        let mut last_err = YahooError::CrumbFailed;

        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                sleep(Duration::from_millis(
                    RETRY_BASE_DELAY_MS * 2u64.pow(attempt - 1),
                ))
                .await;
            }

            match self.client.get(url).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        return Ok(resp);
                    }
                    if status.as_u16() == 401 || status.as_u16() == 403 {
                        self.invalidate_crumb().await;
                        last_err = YahooError::Api(format!("HTTP {status}"));
                        continue;
                    }
                    if status.as_u16() == 429 {
                        last_err = YahooError::Api("Rate limited (429)".into());
                        continue;
                    }
                    return Err(YahooError::Api(format!("HTTP {status}")));
                }
                Err(e) => {
                    last_err = YahooError::Http(e);
                }
            }
        }

        Err(last_err)
    }

    /// Fetch OHLCV historical data for a symbol.
    pub async fn fetch_chart(
        &self,
        symbol: &str,
        start: NaiveDate,
        end: NaiveDate,
        interval: &str,
    ) -> Result<Vec<NewStockPrice>, YahooError> {
        let crumb = self.ensure_crumb().await?;

        let start_ts = start
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        let end_ts = end
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc()
            .timestamp();

        let url = format!(
            "{}/{}?period1={}&period2={}&interval={}&crumb={}&lang={}&region={}",
            YAHOO_CHART_URL, symbol, start_ts, end_ts, interval, crumb, self.lang, self.region
        );

        let resp = self.request_with_retry(&url).await?;
        let chart_resp: ChartResponse = resp.json().await?;

        // Check for API error
        if let Some(ref wrapper) = chart_resp.chart.result {
            if wrapper.is_empty() {
                return Err(YahooError::NoData(symbol.to_string()));
            }
        } else {
            if let Some(ref err) = chart_resp.chart.error {
                return Err(YahooError::Api(
                    err.description
                        .clone()
                        .unwrap_or_else(|| "Unknown error".into()),
                ));
            }
            return Err(YahooError::NoData(symbol.to_string()));
        }

        let data = &chart_resp.chart.result.as_ref().unwrap()[0];
        let timestamps = data.timestamp.as_ref().ok_or_else(|| {
            YahooError::NoData(format!("{}: no timestamps", symbol))
        })?;

        let quote = data
            .indicators
            .quote
            .as_ref()
            .and_then(|q| q.first())
            .ok_or_else(|| YahooError::NoData(format!("{}: no quote data", symbol)))?;

        let adj_closes = data
            .indicators
            .adjclose
            .as_ref()
            .and_then(|a| a.first())
            .and_then(|a| a.adjclose.as_ref());

        let opens = quote.open.as_ref();
        let highs = quote.high.as_ref();
        let lows = quote.low.as_ref();
        let closes = quote.close.as_ref();
        let volumes = quote.volume.as_ref();

        let mut prices = Vec::with_capacity(timestamps.len());

        for i in 0..timestamps.len() {
            let open = opens.and_then(|v| v.get(i).copied().flatten());
            let high = highs.and_then(|v| v.get(i).copied().flatten());
            let low = lows.and_then(|v| v.get(i).copied().flatten());
            let close = closes.and_then(|v| v.get(i).copied().flatten());
            let volume = volumes.and_then(|v| v.get(i).copied().flatten());

            // Skip rows with missing OHLCV data
            let (Some(open), Some(high), Some(low), Some(close), Some(volume)) =
                (open, high, low, close, volume)
            else {
                continue;
            };

            let adj_close = adj_closes.and_then(|v| v.get(i).copied().flatten());

            let date = Utc
                .timestamp_opt(timestamps[i], 0)
                .single()
                .map(|dt| dt.date_naive())
                .ok_or_else(|| YahooError::Api("Invalid timestamp".into()))?;

            prices.push(NewStockPrice {
                symbol: symbol.to_string(),
                date,
                open: (open * 100.0).round() / 100.0,
                high: (high * 100.0).round() / 100.0,
                low: (low * 100.0).round() / 100.0,
                close: (close * 100.0).round() / 100.0,
                volume,
                adj_close: adj_close.map(|v| (v * 100.0).round() / 100.0),
            });
        }

        Ok(prices)
    }

    /// Fetch stock quote summary (company info, PE, market cap, etc.)
    pub async fn fetch_quote_summary(
        &self,
        symbol: &str,
    ) -> Result<StockInfo, YahooError> {
        let crumb = self.ensure_crumb().await?;

        let url = format!(
            "{}/{}?modules=summaryDetail,price&crumb={}&lang={}&region={}",
            YAHOO_QUOTE_SUMMARY_URL, symbol, crumb, self.lang, self.region
        );

        let resp = self.request_with_retry(&url).await?;
        let body: serde_json::Value = resp.json().await?;

        // Parse the nested response manually since Yahoo's structure is complex
        let result = body
            .get("quoteSummary")
            .and_then(|qs| qs.get("result"))
            .and_then(|r| r.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| YahooError::NoData(symbol.to_string()))?;

        let summary = result.get("summaryDetail");
        let price = result.get("price");

        let extract_raw = |obj: Option<&serde_json::Value>, field: &str| -> Option<f64> {
            obj.and_then(|o| o.get(field))
                .and_then(|v| v.get("raw"))
                .and_then(|v| v.as_f64())
        };

        let extract_raw_i64 = |obj: Option<&serde_json::Value>, field: &str| -> Option<i64> {
            obj.and_then(|o| o.get(field))
                .and_then(|v| v.get("raw"))
                .and_then(|v| v.as_i64())
        };

        Ok(StockInfo {
            symbol: symbol.to_string(),
            short_name: price
                .and_then(|p| p.get("shortName"))
                .and_then(|v| v.as_str())
                .map(String::from),
            long_name: price
                .and_then(|p| p.get("longName"))
                .and_then(|v| v.as_str())
                .map(String::from),
            market_cap: extract_raw_i64(summary, "marketCap"),
            pe_ratio: extract_raw(summary, "trailingPE"),
            forward_pe: extract_raw(summary, "forwardPE"),
            dividend_yield: extract_raw(summary, "dividendYield"),
            fifty_two_week_high: extract_raw(summary, "fiftyTwoWeekHigh"),
            fifty_two_week_low: extract_raw(summary, "fiftyTwoWeekLow"),
            fifty_day_average: extract_raw(summary, "fiftyDayAverage"),
            two_hundred_day_average: extract_raw(summary, "twoHundredDayAverage"),
            currency: price
                .and_then(|p| p.get("currency"))
                .and_then(|v| v.as_str())
                .map(String::from),
            exchange: price
                .and_then(|p| p.get("exchangeName"))
                .and_then(|v| v.as_str())
                .map(String::from),
        })
    }

    /// Fetch news for a stock ticker.
    pub async fn fetch_news(
        &self,
        symbol: &str,
        count: u32,
    ) -> Result<Vec<StockNews>, YahooError> {
        let url = format!(
            "{}?q={}&newsCount={}&enableFuzzyQuery=true&quotesCount=0",
            YAHOO_SEARCH_URL, symbol, count
        );

        let resp = self.request_with_retry(&url).await?;
        let search_resp: SearchResponse = resp.json().await?;

        let news_items = search_resp.news.unwrap_or_default();
        let results: Vec<StockNews> = news_items
            .into_iter()
            .map(|item| {
                let pub_date = item.provider_publish_time.map(|ts| {
                    Utc.timestamp_opt(ts, 0)
                        .single()
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_default()
                });

                StockNews {
                    title: item.title.unwrap_or_else(|| "No title".into()),
                    summary: None, // Search API doesn't return summaries
                    publisher: item.publisher.unwrap_or_else(|| "Unknown".into()),
                    link: item.link,
                    pub_date,
                }
            })
            .collect();

        Ok(results)
    }

    /// Search for stock symbols.
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, YahooError> {
        let url = format!(
            "{}?q={}&quotesCount=10&newsCount=0&enableFuzzyQuery=true",
            YAHOO_SEARCH_URL, query
        );

        let resp = self.request_with_retry(&url).await?;
        let search_resp: SearchResponse = resp.json().await?;

        let quotes = search_resp.quotes.unwrap_or_default();
        let results: Vec<SearchResult> = quotes
            .into_iter()
            .filter_map(|q| {
                Some(SearchResult {
                    symbol: q.symbol?,
                    short_name: q.short_name,
                    exchange: q.exchange,
                    quote_type: q.quote_type,
                })
            })
            .collect();

        Ok(results)
    }
}
