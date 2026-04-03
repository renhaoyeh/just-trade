use sqlx::PgPool;
use tauri::State;

#[tauri::command]
pub async fn check_db(pool: State<'_, PgPool>) -> Result<String, String> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool.inner())
        .await
        .map(|_| "Database connection OK".to_string())
        .map_err(|e| format!("Database error: {e}"))
}
