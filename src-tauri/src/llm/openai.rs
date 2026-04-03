use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use super::client::{LlmClient, LlmError};
use super::config::{ChatMessage, ChatResponse, LlmConfig, Role, TokenUsage};

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";

pub struct OpenAIClient {
    client: reqwest::Client,
    config: LlmConfig,
    base_url: String,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<String>,
}

#[derive(Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    model: String,
    usage: Option<OpenAIUsage>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIRespMessage,
}

#[derive(Deserialize)]
struct OpenAIRespMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct OpenAIUsage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    total_tokens: Option<u64>,
}

#[derive(Deserialize)]
struct OpenAIErrorResponse {
    error: OpenAIErrorDetail,
}

#[derive(Deserialize)]
struct OpenAIErrorDetail {
    message: String,
}

impl OpenAIClient {
    pub fn new(config: LlmConfig) -> Result<Self, LlmError> {
        let api_key = config
            .api_key
            .clone()
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .ok_or_else(|| LlmError::MissingApiKey("openai".to_string()))?;

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {api_key}"))
                .map_err(|e| LlmError::Api(format!("Invalid API key: {e}")))?,
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

fn role_to_string(role: &Role) -> String {
    match role {
        Role::System => "system".to_string(),
        Role::User => "user".to_string(),
        Role::Assistant => "assistant".to_string(),
    }
}

#[async_trait]
impl LlmClient for OpenAIClient {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, LlmError> {
        let url = format!("{}/chat/completions", self.base_url);

        let request = OpenAIRequest {
            model: self.config.model.clone(),
            messages: messages
                .iter()
                .map(|m| OpenAIMessage {
                    role: role_to_string(&m.role),
                    content: m.content.clone(),
                })
                .collect(),
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            reasoning_effort: self.config.reasoning_effort.clone(),
        };

        let resp = self.client.post(&url).json(&request).send().await?;
        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            let err_msg = serde_json::from_str::<OpenAIErrorResponse>(&body)
                .map(|e| e.error.message)
                .unwrap_or(body);
            return Err(LlmError::Api(format!("OpenAI API error ({status}): {err_msg}")));
        }

        let parsed: OpenAIResponse = serde_json::from_str(&body)
            .map_err(|e| LlmError::Api(format!("Failed to parse response: {e}")))?;

        let content = parsed
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(ChatResponse {
            content,
            model: parsed.model,
            provider: self.config.provider.to_string(),
            usage: parsed.usage.map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
        })
    }

    fn provider_name(&self) -> &str {
        "openai"
    }

    fn model_name(&self) -> &str {
        &self.config.model
    }
}
