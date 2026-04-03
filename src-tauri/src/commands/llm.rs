use crate::llm::client::create_client;
use crate::llm::config::{ChatMessage, ChatResponse, LlmConfig};

/// Send a chat request to an LLM provider
#[tauri::command]
pub async fn llm_chat(
    config: LlmConfig,
    messages: Vec<ChatMessage>,
) -> Result<ChatResponse, String> {
    let client = create_client(config).map_err(|e| e.to_string())?;
    client.chat(&messages).await.map_err(|e| e.to_string())
}

/// Get available models for a provider
#[tauri::command]
pub fn llm_models(provider: String) -> Result<Vec<String>, String> {
    use crate::llm::catalog::known_models;
    use crate::llm::config::LlmProvider;

    let provider = match provider.to_lowercase().as_str() {
        "openai" => LlmProvider::OpenAI,
        "anthropic" => LlmProvider::Anthropic,
        "google" => LlmProvider::Google,
        "ollama" => LlmProvider::Ollama,
        other => return Err(format!("Unknown provider: {other}")),
    };

    Ok(known_models(&provider)
        .into_iter()
        .map(String::from)
        .collect())
}
