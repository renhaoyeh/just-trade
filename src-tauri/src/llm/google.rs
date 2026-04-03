use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use super::client::{LlmClient, LlmError};
use super::config::{ChatMessage, ChatResponse, LlmConfig, Role, TokenUsage};

const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

pub struct GoogleClient {
    client: reqwest::Client,
    config: LlmConfig,
    base_url: String,
    api_key: String,
}

#[derive(Serialize)]
struct GoogleRequest {
    contents: Vec<GoogleContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GoogleContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Serialize)]
struct GoogleContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    parts: Vec<GooglePart>,
}

#[derive(Serialize, Deserialize)]
struct GooglePart {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct GoogleResponse {
    candidates: Option<Vec<GoogleCandidate>>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<GoogleUsage>,
    #[serde(rename = "modelVersion")]
    model_version: Option<String>,
}

#[derive(Deserialize)]
struct GoogleCandidate {
    content: GoogleRespContent,
}

#[derive(Deserialize)]
struct GoogleRespContent {
    parts: Vec<GooglePart>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleUsage {
    prompt_token_count: Option<u64>,
    candidates_token_count: Option<u64>,
    total_token_count: Option<u64>,
}

#[derive(Deserialize)]
struct GoogleErrorResponse {
    error: GoogleErrorDetail,
}

#[derive(Deserialize)]
struct GoogleErrorDetail {
    message: String,
}

impl GoogleClient {
    pub fn new(config: LlmConfig) -> Result<Self, LlmError> {
        let api_key = config
            .api_key
            .clone()
            .or_else(|| std::env::var("GOOGLE_API_KEY").ok())
            .ok_or_else(|| LlmError::MissingApiKey("google".to_string()))?;

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(120))
            .build()?;

        Ok(Self {
            client,
            config,
            base_url,
            api_key,
        })
    }
}

#[async_trait]
impl LlmClient for GoogleClient {
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, LlmError> {
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, self.config.model, self.api_key
        );

        // Extract system instruction
        let system_instruction = messages
            .iter()
            .find(|m| matches!(m.role, Role::System))
            .map(|m| GoogleContent {
                role: None,
                parts: vec![GooglePart {
                    text: Some(m.content.clone()),
                }],
            });

        let contents: Vec<GoogleContent> = messages
            .iter()
            .filter(|m| !matches!(m.role, Role::System))
            .map(|m| GoogleContent {
                role: Some(match m.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "model".to_string(),
                    Role::System => unreachable!(),
                }),
                parts: vec![GooglePart {
                    text: Some(m.content.clone()),
                }],
            })
            .collect();

        let request = GoogleRequest {
            contents,
            system_instruction,
            generation_config: Some(GenerationConfig {
                temperature: self.config.temperature,
                max_output_tokens: self.config.max_tokens,
            }),
        };

        let resp = self.client.post(&url).json(&request).send().await?;
        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            let err_msg = serde_json::from_str::<GoogleErrorResponse>(&body)
                .map(|e| e.error.message)
                .unwrap_or(body);
            return Err(LlmError::Api(format!(
                "Google API error ({status}): {err_msg}"
            )));
        }

        let parsed: GoogleResponse = serde_json::from_str(&body)
            .map_err(|e| LlmError::Api(format!("Failed to parse response: {e}")))?;

        let content = parsed
            .candidates
            .as_ref()
            .and_then(|c| c.first())
            .map(|c| {
                c.content
                    .parts
                    .iter()
                    .filter_map(|p| p.text.as_deref())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();

        let model = parsed
            .model_version
            .unwrap_or_else(|| self.config.model.clone());

        Ok(ChatResponse {
            content,
            model,
            provider: self.config.provider.to_string(),
            usage: parsed.usage_metadata.map(|u| TokenUsage {
                prompt_tokens: u.prompt_token_count,
                completion_tokens: u.candidates_token_count,
                total_tokens: u.total_token_count,
            }),
        })
    }

    fn provider_name(&self) -> &str {
        "google"
    }

    fn model_name(&self) -> &str {
        &self.config.model
    }
}
