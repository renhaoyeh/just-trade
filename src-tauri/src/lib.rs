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

            // Initialize database pool
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match db::create_pool().await {
                    Ok(pool) => {
                        handle.manage(pool);
                        println!("Database connected successfully");
                    }
                    Err(e) => {
                        eprintln!("Failed to connect to database: {e}");
                    }
                }
            });
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
