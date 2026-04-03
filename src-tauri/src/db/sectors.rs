use sqlx::SqlitePool;

use crate::models::sector::Sector;
use crate::services::twse::TwseCompanyRaw;

/// Upsert TWSE company/sector mappings.
pub async fn upsert_companies(
    pool: &SqlitePool,
    companies: &[TwseCompanyRaw],
) -> Result<(), sqlx::Error> {
    for c in companies {
        sqlx::query(
            r#"
            INSERT INTO twse_companies (code, name, industry_category, updated_at)
            VALUES (?, ?, ?, datetime('now'))
            ON CONFLICT (code) DO UPDATE SET
                name = excluded.name,
                industry_category = excluded.industry_category,
                updated_at = datetime('now')
            "#,
        )
        .bind(c.code.trim())
        .bind(c.name.trim())
        .bind(c.industry_category.trim())
        .execute(pool)
        .await?;
    }
    Ok(())
}

/// Get all sectors with stock counts.
pub async fn get_all_sectors(pool: &SqlitePool) -> Result<Vec<Sector>, sqlx::Error> {
    sqlx::query_as::<_, (String, i64)>(
        r#"
        SELECT industry_category, COUNT(*) as cnt
        FROM twse_companies
        GROUP BY industry_category
        ORDER BY cnt DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(name, stock_count)| Sector { name, stock_count })
            .collect()
    })
}

/// Get all stock codes and names for a given sector.
pub async fn get_stocks_by_sector(
    pool: &SqlitePool,
    sector: &str,
) -> Result<Vec<(String, String)>, sqlx::Error> {
    sqlx::query_as::<_, (String, String)>(
        r#"
        SELECT code, name
        FROM twse_companies
        WHERE industry_category = ?
        ORDER BY code
        "#,
    )
    .bind(sector)
    .fetch_all(pool)
    .await
}

/// Check if sector data exists and when it was last updated.
pub async fn get_last_update(pool: &SqlitePool) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "SELECT updated_at FROM twse_companies ORDER BY updated_at DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
}
