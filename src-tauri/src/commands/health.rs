use sqlx::SqlitePool;
use tauri::State;

#[tauri::command]
pub async fn check_db(pool: State<'_, SqlitePool>) -> Result<String, String> {
    sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(pool.inner())
        .await
        .map(|_| "Database connection OK".to_string())
        .map_err(|e| format!("Database error: {e}"))
}
