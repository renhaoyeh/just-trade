use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// App settings persisted as JSON in the app data directory
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub llm_provider: Option<String>,
    pub llm_model: Option<String>,
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub google_api_key: Option<String>,
    pub ollama_base_url: Option<String>,
}

fn settings_path(app: &AppHandle) -> PathBuf {
    let data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
    std::fs::create_dir_all(&data_dir).ok();
    data_dir.join("settings.json")
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<AppSettings, String> {
    let path = settings_path(&app);
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    let path = settings_path(&app);
    let content = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())
}
