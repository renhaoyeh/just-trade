use async_trait::async_trait;

use super::config::{ChatMessage, ChatResponse, LlmConfig};
use super::validators;

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

/// Abstract base trait for LLM clients.
///
/// Mirrors TradingAgents' `BaseLLMClient`:
/// - `chat()` sends messages and returns normalized text content
/// - `validate_model()` checks if model is in the known catalog
/// - `provider_name()` / `model_name()` for identification
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Send a chat completion request.
    /// Implementations must normalize response content to plain text,
    /// discarding reasoning/thinking blocks (same as `normalize_content()`
    /// in the Python version).
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, LlmError>;

    /// Get the provider name
    fn provider_name(&self) -> &str;

    /// Get the model name
    fn model_name(&self) -> &str;

    /// Validate that the model is known for this provider
    fn validate_model(&self) -> bool;
}

/// Warn if model is not in the known list (non-fatal).
/// Call this at the start of each client's `chat()`.
pub fn warn_if_unknown_model(config: &LlmConfig) {
    if !validators::validate_model(&config.provider, &config.model) {
        eprintln!(
            "Warning: Model '{}' is not in the known model list for provider '{}'. Continuing anyway.",
            config.model, config.provider
        );
    }
}
