use chrono::NaiveDate;
use sqlx::PgPool;

use crate::models::stock_price::{NewStockPrice, StockPrice};

/// Upsert stock prices (insert or update on conflict).
pub async fn upsert_prices(
    pool: &PgPool,
    prices: &[NewStockPrice],
) -> Result<(), sqlx::Error> {
    // Batch in chunks of 500 to stay within PostgreSQL parameter limits
    for chunk in prices.chunks(500) {
        let mut symbols = Vec::with_capacity(chunk.len());
        let mut dates = Vec::with_capacity(chunk.len());
        let mut opens = Vec::with_capacity(chunk.len());
        let mut highs = Vec::with_capacity(chunk.len());
        let mut lows = Vec::with_capacity(chunk.len());
        let mut closes = Vec::with_capacity(chunk.len());
        let mut volumes = Vec::with_capacity(chunk.len());
        let mut adj_closes: Vec<Option<f64>> = Vec::with_capacity(chunk.len());

        for p in chunk {
            symbols.push(p.symbol.as_str());
            dates.push(p.date);
            opens.push(p.open);
            highs.push(p.high);
            lows.push(p.low);
            closes.push(p.close);
            volumes.push(p.volume);
            adj_closes.push(p.adj_close);
        }

        sqlx::query(
            r#"
            INSERT INTO stock_prices (symbol, date, open, high, low, close, volume, adj_close)
            SELECT * FROM UNNEST($1::text[], $2::date[], $3::float8[], $4::float8[], $5::float8[], $6::float8[], $7::int8[], $8::float8[])
            ON CONFLICT (symbol, date) DO UPDATE SET
                open = EXCLUDED.open,
                high = EXCLUDED.high,
                low = EXCLUDED.low,
                close = EXCLUDED.close,
                volume = EXCLUDED.volume,
                adj_close = EXCLUDED.adj_close
            "#,
        )
        .bind(&symbols)
        .bind(&dates)
        .bind(&opens)
        .bind(&highs)
        .bind(&lows)
        .bind(&closes)
        .bind(&volumes)
        .bind(&adj_closes)
        .execute(pool)
        .await?;
    }

    Ok(())
}

/// Get stock prices for a symbol within a date range.
pub async fn get_prices(
    pool: &PgPool,
    symbol: &str,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<Vec<StockPrice>, sqlx::Error> {
    sqlx::query_as::<_, StockPrice>(
        r#"
        SELECT id, symbol, date, open, high, low, close, volume, adj_close, created_at
        FROM stock_prices
        WHERE symbol = $1 AND date >= $2 AND date <= $3
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
    pool: &PgPool,
    symbol: &str,
) -> Result<Option<NaiveDate>, sqlx::Error> {
    sqlx::query_scalar::<_, NaiveDate>(
        "SELECT date FROM stock_prices WHERE symbol = $1 ORDER BY date DESC LIMIT 1",
    )
    .bind(symbol)
    .fetch_optional(pool)
    .await
}

/// Get the earliest date we have data for a symbol.
pub async fn get_earliest_date(
    pool: &PgPool,
    symbol: &str,
) -> Result<Option<NaiveDate>, sqlx::Error> {
    sqlx::query_scalar::<_, NaiveDate>(
        "SELECT date FROM stock_prices WHERE symbol = $1 ORDER BY date ASC LIMIT 1",
    )
    .bind(symbol)
    .fetch_optional(pool)
    .await
}
