use std::collections::HashMap;

use crate::llm::agents;
use crate::llm::client::create_client;
use crate::llm::config::{ChatMessage, ChatResponse, LlmConfig, Role};

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

/// List all available agent definitions
#[tauri::command]
pub fn llm_agents() -> Vec<serde_json::Value> {
    agents::ALL_AGENTS
        .iter()
        .map(|a| {
            serde_json::json!({
                "id": a.id,
                "name": a.name,
                "category": a.category,
            })
        })
        .collect()
}

/// Chat using a predefined agent persona
#[tauri::command]
pub async fn llm_agent_chat(
    config: LlmConfig,
    agent_id: String,
    user_message: String,
    vars: Option<HashMap<String, String>>,
) -> Result<ChatResponse, String> {
    let agent = agents::get_agent(&agent_id)
        .ok_or_else(|| format!("Unknown agent: {agent_id}"))?;

    let system_prompt = match vars {
        Some(v) => agent.render(&v),
        None => agent.prompt_template.to_string(),
    };

    let messages = vec![
        ChatMessage {
            role: Role::System,
            content: system_prompt,
        },
        ChatMessage {
            role: Role::User,
            content: user_message,
        },
    ];

    let client = create_client(config).map_err(|e| e.to_string())?;
    client.chat(&messages).await.map_err(|e| e.to_string())
}
