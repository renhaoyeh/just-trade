use std::collections::HashMap;
use tauri::{AppHandle, Emitter};

use crate::agents;
use crate::llm::base_client::LlmError;
use crate::llm::config::{ChatMessage, LlmConfig, Role};
use crate::llm::factory::create_client;
use serde::{Deserialize, Serialize};

/// Pipeline configuration — all parameters the user can tweak
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// LLM config for quick tasks (analysts, debaters)
    pub quick_llm: LlmConfig,
    /// LLM config for deep thinking tasks (managers, trader)
    pub deep_llm: LlmConfig,
    /// Number of bull/bear debate rounds (each round = 1 bull + 1 bear)
    pub max_debate_rounds: u32,
    /// Number of risk debate rounds (each round = aggressive + conservative + neutral)
    pub max_risk_rounds: u32,
    /// Which analysts to run
    pub enable_market_analyst: bool,
    pub enable_news_analyst: bool,
    pub enable_fundamentals_analyst: bool,
    pub enable_social_analyst: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            quick_llm: LlmConfig::default(),
            deep_llm: LlmConfig::default(),
            max_debate_rounds: 1,
            max_risk_rounds: 1,
            enable_market_analyst: true,
            enable_news_analyst: true,
            enable_fundamentals_analyst: true,
            enable_social_analyst: false,
        }
    }
}

/// Progress event emitted to frontend
#[derive(Clone, Serialize)]
pub struct AnalysisProgress {
    pub phase: String,
    pub agent: String,
    pub status: String, // "running" | "done" | "error"
    pub message: Option<String>,
}

/// Final analysis result
#[derive(Clone, Serialize)]
pub struct AnalysisResult {
    pub market_report: String,
    pub news_report: String,
    pub fundamentals_report: String,
    pub social_report: String,
    pub bull_arguments: String,
    pub bear_arguments: String,
    pub investment_decision: String,
    pub trader_plan: String,
    pub risk_aggressive: String,
    pub risk_conservative: String,
    pub risk_neutral: String,
    pub final_decision: String,
    pub signal: String,
}

fn emit_progress(app: &AppHandle, phase: &str, agent: &str, status: &str, message: Option<&str>) {
    let _ = app.emit("analysis-progress", AnalysisProgress {
        phase: phase.to_string(),
        agent: agent.to_string(),
        status: status.to_string(),
        message: message.map(String::from),
    });
}

async fn call_agent(
    config: &LlmConfig,
    agent_id: &str,
    user_message: &str,
    vars: &HashMap<String, String>,
) -> Result<String, LlmError> {
    let agent = agents::get_agent(agent_id)
        .ok_or_else(|| LlmError::Api(format!("Unknown agent: {agent_id}")))?;

    let system_prompt = agent.render(vars);
    let messages = vec![
        ChatMessage { role: Role::System, content: system_prompt },
        ChatMessage { role: Role::User, content: user_message.to_string() },
    ];

    let client = create_client(config.clone())?;
    let resp = client.chat(&messages).await?;
    Ok(resp.content)
}

#[tauri::command]
pub async fn run_analysis(
    app: AppHandle,
    symbol: String,
    pipeline_config: PipelineConfig,
) -> Result<AnalysisResult, String> {
    let quick = &pipeline_config.quick_llm;
    let deep = &pipeline_config.deep_llm;

    let mut result = AnalysisResult {
        market_report: String::new(),
        news_report: String::new(),
        fundamentals_report: String::new(),
        social_report: String::new(),
        bull_arguments: String::new(),
        bear_arguments: String::new(),
        investment_decision: String::new(),
        trader_plan: String::new(),
        risk_aggressive: String::new(),
        risk_conservative: String::new(),
        risk_neutral: String::new(),
        final_decision: String::new(),
        signal: String::new(),
    };

    let prompt = format!("Analyze {symbol} as of today. Provide a detailed report.");

    // ── Phase 1: Analysts ──
    let analysts: Vec<(&str, &str, bool, &str)> = vec![
        ("market_analyst", "Market Analyst", pipeline_config.enable_market_analyst, "market"),
        ("news_analyst", "News Analyst", pipeline_config.enable_news_analyst, "news"),
        ("fundamentals_analyst", "Fundamentals Analyst", pipeline_config.enable_fundamentals_analyst, "fundamentals"),
        ("social_media_analyst", "Social Media Analyst", pipeline_config.enable_social_analyst, "social"),
    ];

    for (agent_id, name, enabled, field) in &analysts {
        if !enabled {
            continue;
        }
        emit_progress(&app, "analysts", name, "running", None);
        match call_agent(quick, agent_id, &prompt, &HashMap::new()).await {
            Ok(report) => {
                match *field {
                    "market" => result.market_report = report,
                    "news" => result.news_report = report,
                    "fundamentals" => result.fundamentals_report = report,
                    "social" => result.social_report = report,
                    _ => {}
                }
                emit_progress(&app, "analysts", name, "done", None);
            }
            Err(e) => {
                emit_progress(&app, "analysts", name, "error", Some(&e.to_string()));
                return Err(e.to_string());
            }
        }
    }

    // ── Phase 2: Bull vs Bear Debate ──
    let mut debate_history = String::new();
    let mut current_response = String::new();

    for round in 0..pipeline_config.max_debate_rounds {
        // Bull
        emit_progress(&app, "debate", "Bull Researcher", "running",
            Some(&format!("Round {}/{}", round + 1, pipeline_config.max_debate_rounds)));
        let mut vars = HashMap::new();
        vars.insert("market_research_report".into(), result.market_report.clone());
        vars.insert("sentiment_report".into(), result.social_report.clone());
        vars.insert("news_report".into(), result.news_report.clone());
        vars.insert("fundamentals_report".into(), result.fundamentals_report.clone());
        vars.insert("history".into(), debate_history.clone());
        vars.insert("current_response".into(), current_response.clone());
        vars.insert("past_memory_str".into(), String::new());

        match call_agent(quick, "bull_researcher", &prompt, &vars).await {
            Ok(resp) => {
                debate_history.push_str(&format!("\n\n**Bull (Round {}):**\n{resp}", round + 1));
                current_response = resp.clone();
                result.bull_arguments = resp;
                emit_progress(&app, "debate", "Bull Researcher", "done", None);
            }
            Err(e) => {
                emit_progress(&app, "debate", "Bull Researcher", "error", Some(&e.to_string()));
                return Err(e.to_string());
            }
        }

        // Bear
        emit_progress(&app, "debate", "Bear Researcher", "running",
            Some(&format!("Round {}/{}", round + 1, pipeline_config.max_debate_rounds)));
        vars.insert("history".into(), debate_history.clone());
        vars.insert("current_response".into(), current_response.clone());

        match call_agent(quick, "bear_researcher", &prompt, &vars).await {
            Ok(resp) => {
                debate_history.push_str(&format!("\n\n**Bear (Round {}):**\n{resp}", round + 1));
                current_response = resp.clone();
                result.bear_arguments = resp;
                emit_progress(&app, "debate", "Bear Researcher", "done", None);
            }
            Err(e) => {
                emit_progress(&app, "debate", "Bear Researcher", "error", Some(&e.to_string()));
                return Err(e.to_string());
            }
        }
    }

    // ── Phase 3: Research Manager judges debate ──
    emit_progress(&app, "decision", "Research Manager", "running", None);
    {
        let mut vars = HashMap::new();
        vars.insert("history".into(), debate_history.clone());
        vars.insert("past_memory_str".into(), String::new());
        vars.insert("instrument_context".into(), format!("Stock: {symbol}"));

        match call_agent(deep, "research_manager", &prompt, &vars).await {
            Ok(resp) => {
                result.investment_decision = resp;
                emit_progress(&app, "decision", "Research Manager", "done", None);
            }
            Err(e) => {
                emit_progress(&app, "decision", "Research Manager", "error", Some(&e.to_string()));
                return Err(e.to_string());
            }
        }
    }

    // ── Phase 4: Trader ──
    emit_progress(&app, "decision", "Trader", "running", None);
    {
        let mut vars = HashMap::new();
        vars.insert("past_memory_str".into(), String::new());

        let trader_prompt = format!(
            "Based on the investment plan below, make your trading decision for {symbol}.\n\nInvestment Plan:\n{}",
            result.investment_decision
        );
        match call_agent(deep, "trader", &trader_prompt, &vars).await {
            Ok(resp) => {
                result.trader_plan = resp;
                emit_progress(&app, "decision", "Trader", "done", None);
            }
            Err(e) => {
                emit_progress(&app, "decision", "Trader", "error", Some(&e.to_string()));
                return Err(e.to_string());
            }
        }
    }

    // ── Phase 5: Risk Management Debate ──
    let mut risk_history = String::new();
    let mut agg_resp = String::new();
    let mut con_resp = String::new();
    let mut neu_resp = String::new();

    for round in 0..pipeline_config.max_risk_rounds {
        let round_label = format!("Round {}/{}", round + 1, pipeline_config.max_risk_rounds);

        // Aggressive
        emit_progress(&app, "risk", "Aggressive Analyst", "running", Some(&round_label));
        let mut vars = HashMap::new();
        vars.insert("trader_decision".into(), result.trader_plan.clone());
        vars.insert("market_research_report".into(), result.market_report.clone());
        vars.insert("sentiment_report".into(), result.social_report.clone());
        vars.insert("news_report".into(), result.news_report.clone());
        vars.insert("fundamentals_report".into(), result.fundamentals_report.clone());
        vars.insert("history".into(), risk_history.clone());
        vars.insert("current_conservative_response".into(), con_resp.clone());
        vars.insert("current_neutral_response".into(), neu_resp.clone());

        match call_agent(quick, "aggressive_risk", &prompt, &vars).await {
            Ok(resp) => {
                risk_history.push_str(&format!("\n\n**Aggressive (Round {}):**\n{resp}", round + 1));
                agg_resp = resp.clone();
                result.risk_aggressive = resp;
                emit_progress(&app, "risk", "Aggressive Analyst", "done", None);
            }
            Err(e) => {
                emit_progress(&app, "risk", "Aggressive Analyst", "error", Some(&e.to_string()));
                return Err(e.to_string());
            }
        }

        // Conservative
        emit_progress(&app, "risk", "Conservative Analyst", "running", Some(&round_label));
        vars.insert("history".into(), risk_history.clone());
        vars.insert("current_aggressive_response".into(), agg_resp.clone());
        vars.insert("current_neutral_response".into(), neu_resp.clone());

        match call_agent(quick, "conservative_risk", &prompt, &vars).await {
            Ok(resp) => {
                risk_history.push_str(&format!("\n\n**Conservative (Round {}):**\n{resp}", round + 1));
                con_resp = resp.clone();
                result.risk_conservative = resp;
                emit_progress(&app, "risk", "Conservative Analyst", "done", None);
            }
            Err(e) => {
                emit_progress(&app, "risk", "Conservative Analyst", "error", Some(&e.to_string()));
                return Err(e.to_string());
            }
        }

        // Neutral
        emit_progress(&app, "risk", "Neutral Analyst", "running", Some(&round_label));
        vars.insert("history".into(), risk_history.clone());
        vars.insert("current_aggressive_response".into(), agg_resp.clone());
        vars.insert("current_conservative_response".into(), con_resp.clone());

        match call_agent(quick, "neutral_risk", &prompt, &vars).await {
            Ok(resp) => {
                risk_history.push_str(&format!("\n\n**Neutral (Round {}):**\n{resp}", round + 1));
                neu_resp = resp.clone();
                result.risk_neutral = resp;
                emit_progress(&app, "risk", "Neutral Analyst", "done", None);
            }
            Err(e) => {
                emit_progress(&app, "risk", "Neutral Analyst", "error", Some(&e.to_string()));
                return Err(e.to_string());
            }
        }
    }

    // ── Phase 6: Portfolio Manager ──
    emit_progress(&app, "final", "Portfolio Manager", "running", None);
    {
        let mut vars = HashMap::new();
        vars.insert("instrument_context".into(), format!("Stock: {symbol}"));
        vars.insert("trader_plan".into(), result.trader_plan.clone());
        vars.insert("past_memory_str".into(), String::new());
        vars.insert("history".into(), risk_history.clone());

        match call_agent(deep, "portfolio_manager", &prompt, &vars).await {
            Ok(resp) => {
                // Extract signal from response
                let signal = extract_signal(&resp);
                result.final_decision = resp;
                result.signal = signal;
                emit_progress(&app, "final", "Portfolio Manager", "done", None);
            }
            Err(e) => {
                emit_progress(&app, "final", "Portfolio Manager", "error", Some(&e.to_string()));
                return Err(e.to_string());
            }
        }
    }

    emit_progress(&app, "complete", "", "done", Some(&result.signal));
    Ok(result)
}

/// Extract BUY/HOLD/SELL signal from portfolio manager's response
fn extract_signal(text: &str) -> String {
    let upper = text.to_uppercase();
    for signal in ["BUY", "OVERWEIGHT", "SELL", "UNDERWEIGHT", "HOLD"] {
        if upper.contains(signal) {
            return signal.to_string();
        }
    }
    "HOLD".to_string()
}
