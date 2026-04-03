use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use super::base_client::{LlmClient, LlmError};
use super::config::{ChatMessage, ChatResponse, LlmConfig, Role, TokenUsage};

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
const API_VERSION: &str = "2023-06-01";

pub struct AnthropicClient {
    client: reqwest::Client,
    config: LlmConfig,
    base_url: String,
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    /// Extended thinking configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<ThinkingConfig>,
}

#[derive(Serialize)]
struct ThinkingConfig {
    #[serde(rename = "type")]
    thinking_type: String,
    /// Budget tokens for thinking (required when type = "enabled")
    #[serde(skip_serializing_if = "Option::is_none")]
    budget_tokens: Option<u32>,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    model: String,
    usage: Option<AnthropicUsage>,
}

#[derive(Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
}

#[derive(Deserialize)]
struct AnthropicErrorResponse {
    error: AnthropicErrorDetail,
}

#[derive(Deserialize)]
struct AnthropicErrorDetail {
    message: String,
}

impl AnthropicClient {
    pub fn new(config: LlmConfig) -> Result<Self, LlmError> {
        let api_key = config
            .api_key
            .clone()
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
            .ok_or_else(|| LlmError::MissingApiKey("anthropic".to_string()))?;

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        let mut headers = HeaderMap::new();
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&api_key)
                .map_err(|e| LlmError::Api(format!("Invalid API key: {e}")))?,
        );
        headers.insert(
            "anthropic-version",
            HeaderValue::from_static(API_VERSION),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(120))
            .build()?;

        Ok(Self {
            client,
            config,
            base_url,
        })
    }
}

/// Normalize Anthropic's content blocks to plain text.
/// Extracts only "text" type blocks, discards "thinking" etc.
fn normalize_content(blocks: &[AnthropicContent]) -> String {
    blocks
        .iter()
        .filter(|b| b.content_type == "text")
        .filter_map(|b| b.text.as_deref())
        .collect::<Vec<_>>()
        .join("\n")
}

#[async_trait]
impl LlmClient for AnthropicClient {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, LlmError> {
        let url = format!("{}/v1/messages", self.base_url);

        // Extract system message if present
        let system = messages
            .iter()
            .find(|m| matches!(m.role, Role::System))
            .map(|m| m.content.clone());

        let api_messages: Vec<AnthropicMessage> = messages
            .iter()
            .filter(|m| !matches!(m.role, Role::System))
            .map(|m| AnthropicMessage {
                role: match m.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::System => unreachable!(),
                },
                content: m.content.clone(),
            })
            .collect();

        // Map effort level to thinking config
        let thinking = self.config.effort.as_deref().map(|effort| {
            let budget = match effort {
                "low" => 2048,
                "medium" => 8192,
                "high" => 32768,
                _ => 8192,
            };
            ThinkingConfig {
                thinking_type: "enabled".to_string(),
                budget_tokens: Some(budget),
            }
        });

        let request = AnthropicRequest {
            model: self.config.model.clone(),
            messages: api_messages,
            max_tokens: self.config.max_tokens.unwrap_or(4096),
            system,
            // Temperature must be unset when thinking is enabled
            temperature: if thinking.is_some() { None } else { self.config.temperature },
            thinking,
        };

        let resp = self.client.post(&url).json(&request).send().await?;
        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            let err_msg = serde_json::from_str::<AnthropicErrorResponse>(&body)
                .map(|e| e.error.message)
                .unwrap_or(body);
            return Err(LlmError::Api(format!(
                "Anthropic API error ({status}): {err_msg}"
            )));
        }

        let parsed: AnthropicResponse = serde_json::from_str(&body)
            .map_err(|e| LlmError::Api(format!("Failed to parse response: {e}")))?;

        let content = normalize_content(&parsed.content);

        Ok(ChatResponse {
            content,
            model: parsed.model,
            provider: self.config.provider.to_string(),
            usage: parsed.usage.map(|u| TokenUsage {
                prompt_tokens: u.input_tokens,
                completion_tokens: u.output_tokens,
                total_tokens: match (u.input_tokens, u.output_tokens) {
                    (Some(i), Some(o)) => Some(i + o),
                    _ => None,
                },
            }),
        })
    }

    fn provider_name(&self) -> &str {
        "anthropic"
    }

    fn model_name(&self) -> &str {
        &self.config.model
    }

    fn validate_model(&self) -> bool {
        super::validators::validate_model(&self.config.provider, &self.config.model)
    }
}
