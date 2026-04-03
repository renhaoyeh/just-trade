use chrono::NaiveDate;
use sqlx::SqlitePool;
use tauri::State;

use crate::backtest::engine::{self, BacktestConfig, BacktestResult};
use crate::backtest::strategy::{generate_signals, min_data_points, StrategyConfig};
use crate::db::stock_prices;
use crate::services::yahoo::YahooClient;

#[derive(serde::Deserialize)]
pub struct RunBacktestParams {
    pub symbol: String,
    pub start_date: String,
    pub end_date: String,
    pub initial_capital: f64,
    pub strategy: StrategyConfig,
}

/// Run a backtest with the chosen strategy on a given symbol and date range.
/// Uses the cache-through pattern to fetch price data.
#[tauri::command]
pub async fn run_backtest(
    pool: State<'_, SqlitePool>,
    yahoo: State<'_, YahooClient>,
    params: RunBacktestParams,
) -> Result<BacktestResult, String> {
    let start = NaiveDate::parse_from_str(&params.start_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid start_date: {e}"))?;
    let end = NaiveDate::parse_from_str(&params.end_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid end_date: {e}"))?;

    let pool = pool.inner();

    // Cache-through: fetch missing data from Yahoo
    let latest = stock_prices::get_latest_date(pool, &params.symbol)
        .await
        .map_err(|e| format!("DB error: {e}"))?;
    let earliest = stock_prices::get_earliest_date(pool, &params.symbol)
        .await
        .map_err(|e| format!("DB error: {e}"))?;

    let mut fetch_ranges: Vec<(NaiveDate, NaiveDate)> = Vec::new();
    match (earliest, latest) {
        (Some(db_earliest), Some(db_latest)) => {
            if start < db_earliest {
                fetch_ranges.push((start, db_earliest.pred_opt().unwrap_or(db_earliest)));
            }
            if end > db_latest {
                fetch_ranges.push((db_latest.succ_opt().unwrap_or(db_latest), end));
            }
        }
        _ => {
            fetch_ranges.push((start, end));
        }
    }

    for (fetch_start, fetch_end) in fetch_ranges {
        match yahoo
            .fetch_chart(&params.symbol, fetch_start, fetch_end, "1d")
            .await
        {
            Ok(prices) => {
                if !prices.is_empty() {
                    stock_prices::upsert_prices(pool, &prices)
                        .await
                        .map_err(|e| format!("DB upsert error: {e}"))?;
                }
            }
            Err(e) => {
                eprintln!("Yahoo fetch error for {}: {e}", params.symbol);
            }
        }
    }

    // Get full price data from DB
    let prices = stock_prices::get_prices(pool, &params.symbol, start, end)
        .await
        .map_err(|e| format!("DB error: {e}"))?;

    let required = min_data_points(&params.strategy);
    if prices.len() < required {
        return Err(format!(
            "Not enough data: got {} days, need at least {} for this strategy",
            prices.len(),
            required
        ));
    }

    // Compute strategy signals
    let signals = generate_signals(&prices, &params.strategy);

    // Run backtest engine
    let backtest_config = BacktestConfig {
        initial_capital: params.initial_capital,
        ..Default::default()
    };
    let result = engine::run_backtest(&params.symbol, &prices, &signals, &backtest_config);

    Ok(result)
}
