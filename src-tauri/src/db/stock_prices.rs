use chrono::NaiveDate;
use sqlx::SqlitePool;

use crate::models::stock_price::{NewStockPrice, StockPrice};

/// Upsert stock prices (insert or update on conflict).
pub async fn upsert_prices(
    pool: &SqlitePool,
    prices: &[NewStockPrice],
) -> Result<(), sqlx::Error> {
    for p in prices {
        sqlx::query(
            r#"
            INSERT INTO stock_prices (symbol, date, open, high, low, close, volume, adj_close)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT (symbol, date) DO UPDATE SET
                open = excluded.open,
                high = excluded.high,
                low = excluded.low,
                close = excluded.close,
                volume = excluded.volume,
                adj_close = excluded.adj_close
            "#,
        )
        .bind(&p.symbol)
        .bind(p.date)
        .bind(p.open)
        .bind(p.high)
        .bind(p.low)
        .bind(p.close)
        .bind(p.volume)
        .bind(p.adj_close)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Get stock prices for a symbol within a date range.
pub async fn get_prices(
    pool: &SqlitePool,
    symbol: &str,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<Vec<StockPrice>, sqlx::Error> {
    sqlx::query_as::<_, StockPrice>(
        r#"
        SELECT id, symbol, date, open, high, low, close, volume, adj_close, created_at
        FROM stock_prices
        WHERE symbol = ? AND date >= ? AND date <= ?
        ORDER BY date ASC
        "#,
    )
    .bind(symbol)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await
}

/// Get the latest date we have data for a symbol.
pub async fn get_latest_date(
    pool: &SqlitePool,
    symbol: &str,
) -> Result<Option<NaiveDate>, sqlx::Error> {
    sqlx::query_scalar::<_, NaiveDate>(
        "SELECT date FROM stock_prices WHERE symbol = ? ORDER BY date DESC LIMIT 1",
    )
    .bind(symbol)
    .fetch_optional(pool)
    .await
}

/// Get the earliest date we have data for a symbol.
pub async fn get_earliest_date(
    pool: &SqlitePool,
    symbol: &str,
) -> Result<Option<NaiveDate>, sqlx::Error> {
    sqlx::query_scalar::<_, NaiveDate>(
        "SELECT date FROM stock_prices WHERE symbol = ? ORDER BY date ASC LIMIT 1",
    )
    .bind(symbol)
    .fetch_optional(pool)
    .await
}
