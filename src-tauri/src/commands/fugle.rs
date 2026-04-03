use crate::services::fugle::{FugleWsClient, WsCommand};
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn fugle_ws_connect(
    app: AppHandle,
    fugle: State<'_, FugleWsClient>,
    api_key: String,
) -> Result<(), String> {
    fugle.connect(api_key, app).await
}

#[tauri::command]
pub async fn fugle_ws_disconnect(
    fugle: State<'_, FugleWsClient>,
) -> Result<(), String> {
    fugle.disconnect().await
}

#[tauri::command]
pub async fn fugle_ws_subscribe(
    fugle: State<'_, FugleWsClient>,
    channel: String,
    symbol: String,
) -> Result<(), String> {
    fugle
        .send_command(WsCommand::Subscribe { channel, symbol })
        .await
}

#[tauri::command]
pub async fn fugle_ws_unsubscribe(
    fugle: State<'_, FugleWsClient>,
    channel: String,
    symbol: String,
) -> Result<(), String> {
    fugle
        .send_command(WsCommand::Unsubscribe { channel, symbol })
        .await
}

#[tauri::command]
pub async fn fugle_ws_status(
    fugle: State<'_, FugleWsClient>,
) -> Result<bool, String> {
    Ok(fugle.is_connected().await)
}
