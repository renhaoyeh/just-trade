use sqlx::SqlitePool;
use tauri::State;

use crate::db::sectors;
use crate::models::sector::{Sector, SectorStock};
use crate::services::twse::TwseClient;

/// Ensure sector data is fresh (less than 24 hours old), fetching if needed.
async fn ensure_sector_data(
    pool: &SqlitePool,
    twse: &TwseClient,
) -> Result<(), String> {
    let needs_refresh = match sectors::get_last_update(pool)
        .await
        .map_err(|e| format!("DB error: {e}"))?
    {
        Some(updated_at) => {
            let updated = chrono::NaiveDateTime::parse_from_str(&updated_at, "%Y-%m-%d %H:%M:%S")
                .unwrap_or_default();
            let age = chrono::Utc::now().naive_utc() - updated;
            age.num_hours() >= 24
        }
        None => true,
    };

    if needs_refresh {
        let companies = twse
            .fetch_listed_companies()
            .await
            .map_err(|e| format!("TWSE API error: {e}"))?;
        sectors::upsert_companies(pool, &companies)
            .await
            .map_err(|e| format!("DB upsert error: {e}"))?;
    }

    Ok(())
}

/// Get all industry sectors with stock counts.
#[tauri::command]
pub async fn get_sectors(
    pool: State<'_, SqlitePool>,
    twse: State<'_, TwseClient>,
) -> Result<Vec<Sector>, String> {
    let pool = pool.inner();
    ensure_sector_data(pool, twse.inner()).await?;

    sectors::get_all_sectors(pool)
        .await
        .map_err(|e| format!("DB error: {e}"))
}

/// Get all stocks in a sector with live quote data.
#[tauri::command]
pub async fn get_sector_stocks(
    pool: State<'_, SqlitePool>,
    twse: State<'_, TwseClient>,
    sector: String,
) -> Result<Vec<SectorStock>, String> {
    let pool = pool.inner();
    ensure_sector_data(pool, twse.inner()).await?;

    let stocks = sectors::get_stocks_by_sector(pool, &sector)
        .await
        .map_err(|e| format!("DB error: {e}"))?;

    // Fetch live day-all data and join with sector stocks
    let day_all = twse
        .fetch_stock_day_all()
        .await
        .map_err(|e| format!("TWSE API error: {e}"))?;

    let result: Vec<SectorStock> = stocks
        .into_iter()
        .map(|(code, name)| {
            let quote = day_all.iter().find(|q| q.code == code);
            SectorStock {
                symbol: code,
                name,
                sector: sector.clone(),
                closing_price: quote.and_then(|q| q.closing_price),
                change: quote.and_then(|q| q.change),
                trade_volume: quote.and_then(|q| q.trade_volume),
            }
        })
        .collect();

    Ok(result)
}
