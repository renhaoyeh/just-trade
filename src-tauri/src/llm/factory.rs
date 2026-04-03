use super::base_client::{LlmClient, LlmError};
use super::config::{LlmConfig, LlmProvider};

/// Create an LLM client from config.
///
/// Mirrors TradingAgents' `create_llm_client()` factory:
/// - OpenAI → OpenAIClient
/// - Anthropic → AnthropicClient
/// - Google → GoogleClient
/// - Ollama → OpenAIClient with localhost base_url (OpenAI-compatible API)
pub fn create_client(config: LlmConfig) -> Result<Box<dyn LlmClient>, LlmError> {
    match config.provider {
        LlmProvider::OpenAI => Ok(Box::new(super::openai::OpenAIClient::new(config)?)),
        LlmProvider::Anthropic => Ok(Box::new(super::anthropic::AnthropicClient::new(config)?)),
        LlmProvider::Google => Ok(Box::new(super::google::GoogleClient::new(config)?)),
        LlmProvider::Groq => {
            let mut config = config;
            config.base_url = config
                .base_url
                .or(Some("https://api.groq.com/openai/v1".to_string()));
            if config.api_key.is_none() {
                config.api_key = std::env::var("GROQ_API_KEY").ok();
            }
            Ok(Box::new(super::openai::OpenAIClient::new(config)?))
        }
        LlmProvider::Ollama => {
            let mut config = config;
            config.base_url = config
                .base_url
                .or(Some("http://localhost:11434/v1".to_string()));
            // Ollama doesn't need an API key
            if config.api_key.is_none() {
                config.api_key = Some("ollama".to_string());
            }
            Ok(Box::new(super::openai::OpenAIClient::new(config)?))
        }
    }
}
