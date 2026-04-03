use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;

const TWSE_COMPANY_LIST_URL: &str =
    "https://openapi.twse.com.tw/v1/opendata/t187ap03_L";
const TWSE_STOCK_DAY_ALL_URL: &str =
    "https://openapi.twse.com.tw/v1/exchangeReport/STOCK_DAY_ALL";

/// In-memory cache TTL for day-all data (5 minutes)
const DAY_ALL_CACHE_SECS: u64 = 300;

#[derive(Debug, thiserror::Error)]
pub enum TwseError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("TWSE API error: {0}")]
    Api(String),
}

/// Raw response from t187ap03_L (listed company basic info)
#[derive(Debug, Deserialize)]
pub struct TwseCompanyRaw {
    #[serde(rename = "公司代號")]
    pub code: String,
    #[serde(rename = "公司簡稱")]
    pub name: String,
    #[serde(rename = "產業類別")]
    pub industry_category: String,
}

/// Raw response from STOCK_DAY_ALL
#[derive(Debug, Deserialize)]
pub struct TwseDayQuoteRaw {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "TradeVolume")]
    pub trade_volume: String,
    #[serde(rename = "OpeningPrice")]
    pub opening_price: String,
    #[serde(rename = "HighestPrice")]
    pub highest_price: String,
    #[serde(rename = "LowestPrice")]
    pub lowest_price: String,
    #[serde(rename = "ClosingPrice")]
    pub closing_price: String,
    #[serde(rename = "Change")]
    pub change: String,
}

/// Parsed day quote
#[derive(Debug, Clone)]
pub struct TwseDayQuote {
    pub code: String,
    pub name: String,
    pub trade_volume: Option<i64>,
    pub closing_price: Option<f64>,
    pub change: Option<f64>,
}

pub struct TwseClient {
    client: reqwest::Client,
    day_all_cache: Arc<RwLock<Option<(std::time::Instant, Vec<TwseDayQuote>)>>>,
}

impl TwseClient {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            ),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            day_all_cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Fetch all TWSE listed companies with their industry classification.
    pub async fn fetch_listed_companies(&self) -> Result<Vec<TwseCompanyRaw>, TwseError> {
        let resp = self
            .client
            .get(TWSE_COMPANY_LIST_URL)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(TwseError::Api(format!("HTTP {}", resp.status())));
        }

        let companies: Vec<TwseCompanyRaw> = resp.json().await?;
        Ok(companies)
    }

    /// Fetch today's trading data for all TWSE stocks, with in-memory caching.
    pub async fn fetch_stock_day_all(&self) -> Result<Vec<TwseDayQuote>, TwseError> {
        // Check cache
        {
            let cache = self.day_all_cache.read().await;
            if let Some((ts, data)) = cache.as_ref() {
                if ts.elapsed().as_secs() < DAY_ALL_CACHE_SECS {
                    return Ok(data.clone());
                }
            }
        }

        let resp = self
            .client
            .get(TWSE_STOCK_DAY_ALL_URL)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(TwseError::Api(format!("HTTP {}", resp.status())));
        }

        let raw: Vec<TwseDayQuoteRaw> = resp.json().await?;
        let quotes: Vec<TwseDayQuote> = raw
            .into_iter()
            .map(|r| TwseDayQuote {
                code: r.code.trim().to_string(),
                name: r.name.trim().to_string(),
                trade_volume: r.trade_volume.replace(",", "").parse().ok(),
                closing_price: r.closing_price.replace(",", "").parse().ok(),
                change: r.change.replace(",", "").parse().ok(),
            })
            .collect();

        // Update cache
        {
            let mut cache = self.day_all_cache.write().await;
            *cache = Some((std::time::Instant::now(), quotes.clone()));
        }

        Ok(quotes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_day_quote_raw() {
        let raw = TwseDayQuoteRaw {
            code: " 2330 ".to_string(),
            name: " 台積電 ".to_string(),
            trade_volume: "35,678,123".to_string(),
            opening_price: "600.00".to_string(),
            highest_price: "610.00".to_string(),
            lowest_price: "595.00".to_string(),
            closing_price: "605.00".to_string(),
            change: "-5.00".to_string(),
        };

        let quote = TwseDayQuote {
            code: raw.code.trim().to_string(),
            name: raw.name.trim().to_string(),
            trade_volume: raw.trade_volume.replace(",", "").parse().ok(),
            closing_price: raw.closing_price.replace(",", "").parse().ok(),
            change: raw.change.replace(",", "").parse().ok(),
        };

        assert_eq!(quote.code, "2330");
        assert_eq!(quote.name, "台積電");
        assert_eq!(quote.trade_volume, Some(35678123));
        assert_eq!(quote.closing_price, Some(605.0));
        assert_eq!(quote.change, Some(-5.0));
    }

    #[test]
    fn parse_day_quote_missing_values() {
        let raw = TwseDayQuoteRaw {
            code: "9999".to_string(),
            name: "測試".to_string(),
            trade_volume: "--".to_string(),
            opening_price: "--".to_string(),
            highest_price: "--".to_string(),
            lowest_price: "--".to_string(),
            closing_price: "--".to_string(),
            change: "X0.00".to_string(),
        };

        let quote = TwseDayQuote {
            code: raw.code.trim().to_string(),
            name: raw.name.trim().to_string(),
            trade_volume: raw.trade_volume.replace(",", "").parse().ok(),
            closing_price: raw.closing_price.replace(",", "").parse().ok(),
            change: raw.change.replace(",", "").parse().ok(),
        };

        assert_eq!(quote.trade_volume, None);
        assert_eq!(quote.closing_price, None);
        assert_eq!(quote.change, None);
    }
}
