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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
