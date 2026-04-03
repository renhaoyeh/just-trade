use super::config::LlmProvider;

/// A model option: (description, model_id)
pub type ModelOption = (&'static str, &'static str);

/// Get model options for a provider, split by mode (quick vs deep thinking).
///
/// Mirrors TradingAgents' `MODEL_OPTIONS` structure.
pub fn model_options(provider: &LlmProvider, mode: &str) -> Vec<ModelOption> {
    match (provider, mode) {
        // OpenAI
        (LlmProvider::OpenAI, "quick") => vec![
            ("GPT-4o Mini — fast, affordable", "gpt-4o-mini"),
            ("GPT-4o — balanced", "gpt-4o"),
            ("GPT-4 Turbo — high quality", "gpt-4-turbo"),
        ],
        (LlmProvider::OpenAI, "deep") => vec![
            ("GPT-4o — balanced", "gpt-4o"),
            ("o1 — reasoning", "o1"),
            ("o1 Mini — fast reasoning", "o1-mini"),
            ("o3 Mini — latest reasoning", "o3-mini"),
        ],
        // Anthropic
        (LlmProvider::Anthropic, "quick") => vec![
            ("Sonnet 4.6 — fast, capable", "claude-sonnet-4-6-20250514"),
            ("Haiku 4.5 — fastest", "claude-haiku-4-5-20251001"),
            ("Sonnet 4.5 — balanced", "claude-sonnet-4-5-20250514"),
        ],
        (LlmProvider::Anthropic, "deep") => vec![
            ("Opus 4.6 — most capable", "claude-opus-4-6-20250515"),
            ("Opus 4.5 — deep reasoning", "claude-opus-4-5-20250514"),
            ("Sonnet 4.6 — capable", "claude-sonnet-4-6-20250514"),
        ],
        // Google
        (LlmProvider::Google, "quick") => vec![
            ("Gemini 2.5 Flash — fast", "gemini-2.5-flash"),
            ("Gemini 2.0 Flash — balanced", "gemini-2.0-flash"),
            ("Gemini 1.5 Flash — lightweight", "gemini-1.5-flash"),
        ],
        (LlmProvider::Google, "deep") => vec![
            ("Gemini 2.5 Pro — most capable", "gemini-2.5-pro"),
            ("Gemini 2.5 Flash — balanced", "gemini-2.5-flash"),
            ("Gemini 1.5 Pro — reliable", "gemini-1.5-pro"),
        ],
        // Ollama — user provides any model string
        (LlmProvider::Ollama, _) => vec![
            ("Qwen3 — general purpose", "qwen3"),
            ("Llama 3 — Meta open model", "llama3"),
            ("Mistral — fast local", "mistral"),
        ],
        _ => vec![],
    }
}

/// Flat list of all known model IDs for a provider (used by validators).
pub fn known_models(provider: &LlmProvider) -> Vec<&'static str> {
    let mut models: Vec<&str> = Vec::new();
    for mode in ["quick", "deep"] {
        for (_, id) in model_options(provider, mode) {
            if !models.contains(&id) {
                models.push(id);
            }
        }
    }
    models
}
