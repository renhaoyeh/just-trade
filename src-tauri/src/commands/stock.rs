use chrono::NaiveDate;
use sqlx::SqlitePool;
use tauri::State;

use crate::db::stock_prices;
use crate::models::stock_info::StockInfo;
use crate::models::stock_news::{SearchResult, StockNews};
use crate::models::stock_price::StockPrice;
use crate::services::yahoo::YahooClient;

/// Fetch stock history with cache-through pattern:
/// 1. Check DB for existing data
/// 2. Fetch missing data from Yahoo Finance
/// 3. Store in DB
/// 4. Return full range from DB
#[tauri::command]
pub async fn fetch_stock_history(
    pool: State<'_, SqlitePool>,
    yahoo: State<'_, YahooClient>,
    symbol: String,
    start_date: String,
    end_date: String,
) -> Result<Vec<StockPrice>, String> {
    let start = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid start_date: {e}"))?;
    let end = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid end_date: {e}"))?;

    let pool = pool.inner();

    // Check what we already have in DB
    let latest = stock_prices::get_latest_date(pool, &symbol)
        .await
        .map_err(|e| format!("DB error: {e}"))?;
    let earliest = stock_prices::get_earliest_date(pool, &symbol)
        .await
        .map_err(|e| format!("DB error: {e}"))?;

    // Determine what we need to fetch from Yahoo
    let mut fetch_ranges: Vec<(NaiveDate, NaiveDate)> = Vec::new();

    match (earliest, latest) {
        (Some(db_earliest), Some(db_latest)) => {
            // Need data before our earliest?
            if start < db_earliest {
                fetch_ranges.push((start, db_earliest.pred_opt().unwrap_or(db_earliest)));
            }
            // Need data after our latest?
            if end > db_latest {
                fetch_ranges.push((db_latest.succ_opt().unwrap_or(db_latest), end));
            }
        }
        _ => {
            // No data at all, fetch everything
            fetch_ranges.push((start, end));
        }
    }

    // Fetch missing ranges from Yahoo
    for (fetch_start, fetch_end) in fetch_ranges {
        match yahoo.fetch_chart(&symbol, fetch_start, fetch_end, "1d").await {
            Ok(prices) => {
                if !prices.is_empty() {
                    stock_prices::upsert_prices(pool, &prices)
                        .await
                        .map_err(|e| format!("DB upsert error: {e}"))?;
                }
            }
            Err(e) => {
                eprintln!("Yahoo fetch error for {symbol}: {e}");
                // Don't fail entirely - return what we have in DB
            }
        }
    }

    // Return full range from DB
    stock_prices::get_prices(pool, &symbol, start, end)
        .await
        .map_err(|e| format!("DB error: {e}"))
}

#[tauri::command]
pub async fn get_stock_info(
    yahoo: State<'_, YahooClient>,
    symbol: String,
) -> Result<StockInfo, String> {
    yahoo
        .fetch_quote_summary(&symbol)
        .await
        .map_err(|e| format!("Failed to fetch stock info: {e}"))
}

#[tauri::command]
pub async fn fetch_stock_news(
    yahoo: State<'_, YahooClient>,
    symbol: String,
    count: Option<u32>,
) -> Result<Vec<StockNews>, String> {
    yahoo
        .fetch_news(&symbol, count.unwrap_or(10))
        .await
        .map_err(|e| format!("Failed to fetch news: {e}"))
}

#[tauri::command]
pub async fn search_stocks(
    yahoo: State<'_, YahooClient>,
    query: String,
) -> Result<Vec<SearchResult>, String> {
    yahoo
        .search(&query)
        .await
        .map_err(|e| format!("Failed to search stocks: {e}"))
}
