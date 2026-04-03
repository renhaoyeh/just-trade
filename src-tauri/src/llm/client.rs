use async_trait::async_trait;

use super::config::{ChatMessage, ChatResponse, LlmConfig};

/// Error type for LLM operations
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error: {0}")]
    Api(String),
    #[error("Missing API key for provider: {0}")]
    MissingApiKey(String),
    #[error("Unsupported provider: {0}")]
    UnsupportedProvider(String),
}

/// Trait that all LLM provider clients must implement
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Send a chat completion request
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, LlmError>;

    /// Get the provider name
    fn provider_name(&self) -> &str;

    /// Get the model name
    fn model_name(&self) -> &str;
}

/// Factory function to create an LLM client from config
pub fn create_client(config: LlmConfig) -> Result<Box<dyn LlmClient>, LlmError> {
    use super::config::LlmProvider;

    match config.provider {
        LlmProvider::OpenAI => Ok(Box::new(super::openai::OpenAIClient::new(config)?)),
        LlmProvider::Anthropic => Ok(Box::new(super::anthropic::AnthropicClient::new(config)?)),
        LlmProvider::Google => Ok(Box::new(super::google::GoogleClient::new(config)?)),
        LlmProvider::Ollama => {
            let mut config = config;
            config.base_url = config
                .base_url
                .or(Some("http://localhost:11434".to_string()));
            Ok(Box::new(super::openai::OpenAIClient::new(config)?))
        }
    }
}
