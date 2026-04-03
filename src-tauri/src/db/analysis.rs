use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AnalysisRecord {
    pub id: i64,
    pub symbol: String,
    pub signal: String,
    pub created_at: String,
    pub report_path: String,
}

pub async fn save_analysis(
    pool: &SqlitePool,
    symbol: &str,
    signal: &str,
    report_path: &str,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO analysis_history (symbol, signal, report_path)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(symbol)
    .bind(signal)
    .bind(report_path)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// Check if an analysis for this symbol already exists today.
pub async fn get_today_analysis(
    pool: &SqlitePool,
    symbol: &str,
) -> Result<Option<AnalysisRecord>, sqlx::Error> {
    sqlx::query_as::<_, AnalysisRecord>(
        r#"
        SELECT id, symbol, signal, created_at, report_path
        FROM analysis_history
        WHERE symbol = ? AND date(created_at) = date('now')
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(symbol)
    .fetch_optional(pool)
    .await
}

/// Save a single agent step result. Upserts on (symbol, trade_date, agent_id).
pub async fn save_step(
    pool: &SqlitePool,
    symbol: &str,
    trade_date: &str,
    agent_id: &str,
    phase: &str,
    content: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO analysis_steps (symbol, trade_date, agent_id, phase, content)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT (symbol, trade_date, agent_id) DO UPDATE SET
            content = excluded.content,
            created_at = datetime('now')
        "#,
    )
    .bind(symbol)
    .bind(trade_date)
    .bind(agent_id)
    .bind(phase)
    .bind(content)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get all completed steps for a symbol on a given date.
pub async fn get_steps(
    pool: &SqlitePool,
    symbol: &str,
    trade_date: &str,
) -> Result<Vec<(String, String, String)>, sqlx::Error> {
    sqlx::query_as::<_, (String, String, String)>(
        r#"
        SELECT agent_id, phase, content
        FROM analysis_steps
        WHERE symbol = ? AND trade_date = ?
        ORDER BY id ASC
        "#,
    )
    .bind(symbol)
    .bind(trade_date)
    .fetch_all(pool)
    .await
}

pub async fn get_analysis_history(
    pool: &SqlitePool,
    limit: i64,
) -> Result<Vec<AnalysisRecord>, sqlx::Error> {
    sqlx::query_as::<_, AnalysisRecord>(
        r#"
        SELECT id, symbol, signal, created_at, report_path
        FROM analysis_history
        ORDER BY created_at DESC
        LIMIT ?
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn get_analysis_by_id(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<AnalysisRecord>, sqlx::Error> {
    sqlx::query_as::<_, AnalysisRecord>(
        r#"
        SELECT id, symbol, signal, created_at, report_path
        FROM analysis_history
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}
