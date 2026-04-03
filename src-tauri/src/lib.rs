use tauri::Manager;

pub mod agents;
mod commands;
pub mod dataflows;
mod db;
pub mod llm;
mod models;
mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Initialize API clients
            let yahoo_client = services::yahoo::YahooClient::new("zh-TW", "TW");
            app.manage(yahoo_client);
            let twse_client = services::twse::TwseClient::new();
            app.manage(twse_client);
            let fugle_ws = services::fugle::FugleWsClient::new();
            app.manage(fugle_ws);

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
            commands::sector::get_sectors,
            commands::sector::get_sector_stocks,
            commands::llm::llm_chat,
            commands::llm::llm_models,
            commands::llm::llm_test,
            commands::llm::llm_agents,
            commands::llm::llm_agent_chat,
            commands::analysis::run_analysis,
            commands::analysis::get_analysis_history,
            commands::analysis::get_analysis_detail,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::fugle::fugle_ws_connect,
            commands::fugle::fugle_ws_disconnect,
            commands::fugle::fugle_ws_subscribe,
            commands::fugle::fugle_ws_unsubscribe,
            commands::fugle::fugle_ws_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
