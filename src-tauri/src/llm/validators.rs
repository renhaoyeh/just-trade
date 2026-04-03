use super::catalog::known_models;
use super::config::LlmProvider;

/// Validate a model for a given provider.
///
/// Mirrors TradingAgents' `validators.py`:
/// - Ollama: always valid (accepts any model string)
/// - Unknown/empty catalog: valid by default
/// - Known providers: check against catalog (prefix match)
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
