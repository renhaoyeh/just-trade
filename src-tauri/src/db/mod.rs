pub mod analysis;
pub mod sectors;
pub mod stock_prices;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::Path;
use std::str::FromStr;

/// Create a SQLite connection pool. The DB file is stored at the given path.
/// In dev mode, if migrations fail, the DB is deleted and recreated automatically.
pub async fn create_pool(db_path: &Path) -> Result<SqlitePool, sqlx::Error> {
    match try_create_pool(db_path).await {
        Ok(pool) => Ok(pool),
        Err(e) if cfg!(debug_assertions) => {
            eprintln!("Migration failed: {e}. Resetting database...");
            // Delete DB files and retry
            for ext in ["", "-shm", "-wal"] {
                let p = db_path.with_extension(
                    format!("db{ext}"),
                );
                std::fs::remove_file(&p).ok();
            }
            // Also try removing the exact path in case extension logic differs
            std::fs::remove_file(db_path).ok();
            try_create_pool(db_path).await
        }
        Err(e) => Err(e),
    }
}

async fn try_create_pool(db_path: &Path) -> Result<SqlitePool, sqlx::Error> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path.display()))?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
