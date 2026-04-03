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

/// Categorize a model ID by capability
fn categorize_model(id: &str) -> &'static str {
    let id = id.to_lowercase();
    // Reasoning
    if id.starts_with("o1") || id.starts_with("o3") || id.starts_with("o4")
        || id.contains("reasoning") || id.contains("deepseek-r1")
    {
        return "reasoning";
    }
    // Embedding
    if id.contains("embed") {
        return "embedding";
    }
    // Image generation
    if id.contains("dall-e") || id.contains("imagen") || id.contains("image-generation") {
        return "image";
    }
    // Audio / Speech
    if id.contains("whisper") || id.contains("tts") || id.contains("audio") {
        return "audio";
    }
    // Moderation
    if id.contains("moderation") {
        return "moderation";
    }
    // Chat (default)
    "chat"
}

fn group_models(models: Vec<String>) -> HashMap<String, Vec<String>> {
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    for m in models {
        let cat = categorize_model(&m).to_string();
        groups.entry(cat).or_default().push(m);
    }
    // Sort each group
    for v in groups.values_mut() {
        v.sort();
    }
    groups
}

/// Test LLM connection by hitting /models endpoint.
/// Returns models grouped by capability: { "chat": [...], "reasoning": [...], ... }
#[tauri::command]
pub async fn llm_test(config: LlmConfig) -> Result<HashMap<String, Vec<String>>, String> {
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
            Ok(group_models(models))
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
            Ok(group_models(models))
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
            Ok(group_models(models))
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
            Ok(group_models(models))
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
