use super::config::LlmProvider;

/// Known models per provider for validation
pub fn known_models(provider: &LlmProvider) -> Vec<&'static str> {
    match provider {
        LlmProvider::OpenAI => vec![
            "gpt-4o",
            "gpt-4o-mini",
            "gpt-4-turbo",
            "gpt-4",
            "gpt-3.5-turbo",
            "o1",
            "o1-mini",
            "o1-preview",
            "o3-mini",
        ],
        LlmProvider::Anthropic => vec![
            "claude-opus-4-6-20250515",
            "claude-sonnet-4-6-20250514",
            "claude-haiku-4-5-20251001",
            "claude-sonnet-4-5-20250514",
            "claude-opus-4-5-20250514",
        ],
        LlmProvider::Google => vec![
            "gemini-2.5-pro",
            "gemini-2.5-flash",
            "gemini-2.0-flash",
            "gemini-1.5-pro",
            "gemini-1.5-flash",
        ],
        LlmProvider::Ollama => vec![], // accepts any model
    }
}

/// Validate model for provider. Ollama accepts anything.
pub fn validate_model(provider: &LlmProvider, model: &str) -> bool {
    if *provider == LlmProvider::Ollama {
        return true;
    }
    let models = known_models(provider);
    if models.is_empty() {
        return true;
    }
    models.iter().any(|m| model.starts_with(m))
}
