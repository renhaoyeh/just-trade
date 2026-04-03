use std::collections::HashMap;

use crate::agents;
use crate::llm::factory::create_client;
use crate::llm::config::{ChatMessage, ChatResponse, LlmConfig, LlmProvider, Role};

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
        "groq" => LlmProvider::Groq,
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

/// Test LLM connection by hitting /models endpoint.
/// Returns the list of available model IDs.
#[tauri::command]
pub async fn llm_test(config: LlmConfig) -> Result<Vec<String>, String> {
    let api_key = config.api_key.clone().unwrap_or_default();
    let client = reqwest::Client::new();

    match config.provider {
        LlmProvider::OpenAI | LlmProvider::Groq => {
            let base = config.base_url.clone().unwrap_or_else(|| {
                if config.provider == LlmProvider::Groq {
                    "https://api.groq.com/openai/v1".to_string()
                } else {
                    "https://api.openai.com/v1".to_string()
                }
            });
            let resp = client
                .get(format!("{base}/models"))
                .bearer_auth(&api_key)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            if !resp.status().is_success() {
                return Err(format!("HTTP {}", resp.status()));
            }
            let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let models = body["data"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| m["id"].as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            Ok(models)
        }
        LlmProvider::Anthropic => {
            let resp = client
                .get("https://api.anthropic.com/v1/models")
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .send()
                .await
                .map_err(|e| e.to_string())?;
            if !resp.status().is_success() {
                return Err(format!("HTTP {}", resp.status()));
            }
            let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let models = body["data"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| m["id"].as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            Ok(models)
        }
        LlmProvider::Google => {
            let base = config
                .base_url
                .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string());
            let resp = client
                .get(format!("{base}/models?key={api_key}"))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            if !resp.status().is_success() {
                return Err(format!("HTTP {}", resp.status()));
            }
            let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let models = body["models"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| {
                            m["name"]
                                .as_str()
                                .map(|n| n.strip_prefix("models/").unwrap_or(n).to_string())
                        })
                        .collect()
                })
                .unwrap_or_default();
            Ok(models)
        }
        LlmProvider::Ollama => {
            let base = config
                .base_url
                .unwrap_or_else(|| "http://localhost:11434".to_string());
            let resp = client
                .get(format!("{base}/api/tags"))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            if !resp.status().is_success() {
                return Err(format!("HTTP {}", resp.status()));
            }
            let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let models = body["models"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| m["name"].as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            Ok(models)
        }
    }
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
