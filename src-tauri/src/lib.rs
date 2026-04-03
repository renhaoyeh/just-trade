use tauri::Manager;

mod commands;
mod db;
mod models;
mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Initialize Yahoo Finance client
            let yahoo_client = services::yahoo::YahooClient::new();
            app.manage(yahoo_client);

            // Initialize SQLite database in app data directory
            let app_data = app.path().app_data_dir().expect("Failed to get app data dir");
            let db_path = app_data.join("just_trade.db");
            let pool = tauri::async_runtime::block_on(db::create_pool(&db_path))
                .expect("Failed to initialize database");
            app.manage(pool);
            println!("Database initialized at {}", db_path.display());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet::greet,
            commands::health::check_db,
            commands::stock::fetch_stock_history,
            commands::stock::get_stock_info,
            commands::stock::fetch_stock_news,
            commands::stock::search_stocks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
