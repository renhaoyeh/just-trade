use std::collections::HashMap;
use sqlx::SqlitePool;
use tauri::{AppHandle, Emitter, Manager};

use crate::agents;
use crate::db;
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
    /// Seconds to wait between each agent call (for rate-limited free APIs)
    pub cooldown_secs: u64,
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
            cooldown_secs: 0,
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
#[derive(Clone, Serialize, Deserialize)]
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

const MAX_RETRIES: u32 = 3;
const RETRY_BASE_DELAY_SECS: u64 = 15;

async fn call_agent(
    app: &AppHandle,
    config: &LlmConfig,
    agent_id: &str,
    user_message: &str,
    vars: &HashMap<String, String>,
    cooldown_secs: u64,
) -> Result<String, LlmError> {
    let agent = agents::get_agent(agent_id)
        .ok_or_else(|| LlmError::Api(format!("Unknown agent: {agent_id}")))?;

    let system_prompt = agent.render(vars);
    let messages = vec![
        ChatMessage { role: Role::System, content: system_prompt },
        ChatMessage { role: Role::User, content: user_message.to_string() },
    ];

    let client = create_client(config.clone())?;

    let mut last_err = LlmError::Api("Unknown error".into());
    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            let delay = RETRY_BASE_DELAY_SECS * 2u64.pow(attempt - 1);
            eprintln!("Rate limited, retrying in {delay}s (attempt {attempt}/{MAX_RETRIES})");
            tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
        }
        match client.chat(&messages).await {
            Ok(resp) => {
                cooldown(app, cooldown_secs).await;
                return Ok(resp.content);
            }
            Err(LlmError::Api(msg)) if msg.contains("429") || msg.to_lowercase().contains("rate limit") => {
                last_err = LlmError::Api(msg);
            }
            Err(e) => return Err(e),
        }
    }
    Err(last_err)
}

async fn cooldown(app: &AppHandle, secs: u64) {
    if secs > 0 {
        let _ = app.emit("analysis-progress", AnalysisProgress {
            phase: "cooldown".to_string(),
            agent: String::new(),
            status: "running".to_string(),
            message: Some(format!("Waiting {secs}s (rate limit)")),
        });
        tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
    }
}

/// Run or load a single step. Returns cached content if available, otherwise calls LLM.
async fn run_or_load_step(
    app: &AppHandle,
    pool: &SqlitePool,
    config: &LlmConfig,
    symbol: &str,
    today: &str,
    agent_id: &str,
    agent_name: &str,
    phase: &str,
    prompt: &str,
    vars: &HashMap<String, String>,
    cooldown_secs: u64,
    cached_steps: &HashMap<String, String>,
) -> Result<String, String> {
    // Check cache first
    if let Some(content) = cached_steps.get(agent_id) {
        emit_progress(app, phase, agent_name, "done", Some(content));
        return Ok(content.clone());
    }

    // Run LLM
    emit_progress(app, phase, agent_name, "running", None);
    match call_agent(app, config, agent_id, prompt, vars, cooldown_secs).await {
        Ok(resp) => {
            // Save step to DB immediately
            let _ = db::analysis::save_step(pool, symbol, today, agent_id, phase, &resp).await;
            emit_progress(app, phase, agent_name, "done", Some(&resp));
            Ok(resp)
        }
        Err(e) => {
            emit_progress(app, phase, agent_name, "error", Some(&e.to_string()));
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn run_analysis(
    app: AppHandle,
    symbol: String,
    pipeline_config: PipelineConfig,
) -> Result<AnalysisResult, String> {
    let pool = app.state::<SqlitePool>();
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    // Load any steps already completed today for this symbol
    let existing_steps = db::analysis::get_steps(pool.inner(), &symbol, &today)
        .await
        .unwrap_or_default();
    let cached_steps: HashMap<String, String> = existing_steps
        .into_iter()
        .map(|(agent_id, _phase, content)| (agent_id, content))
        .collect();

    if !cached_steps.is_empty() {
        eprintln!("Resuming analysis for {symbol}: {} steps cached", cached_steps.len());
    }

    let quick = &pipeline_config.quick_llm;
    let deep = &pipeline_config.deep_llm;
    let pool_ref = pool.inner();

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
        let report = run_or_load_step(
            &app, pool_ref, quick, &symbol, &today,
            agent_id, name, "analysts", &prompt, &HashMap::new(),
            pipeline_config.cooldown_secs, &cached_steps,
        ).await?;
        match *field {
            "market" => result.market_report = report,
            "news" => result.news_report = report,
            "fundamentals" => result.fundamentals_report = report,
            "social" => result.social_report = report,
            _ => {}
        }
    }

    // ── Phase 2: Bull vs Bear Debate ──
    let mut debate_history = String::new();
    let mut current_response = String::new();

    for round in 0..pipeline_config.max_debate_rounds {
        let mut vars = HashMap::new();
        vars.insert("market_research_report".into(), result.market_report.clone());
        vars.insert("sentiment_report".into(), result.social_report.clone());
        vars.insert("news_report".into(), result.news_report.clone());
        vars.insert("fundamentals_report".into(), result.fundamentals_report.clone());
        vars.insert("history".into(), debate_history.clone());
        vars.insert("current_response".into(), current_response.clone());
        vars.insert("past_memory_str".into(), String::new());

        let bull_id = format!("bull_researcher_r{}", round + 1);
        let resp = run_or_load_step(
            &app, pool_ref, quick, &symbol, &today,
            &bull_id, "Bull Researcher", "debate", &prompt, &vars,
            pipeline_config.cooldown_secs, &cached_steps,
        ).await?;
        debate_history.push_str(&format!("\n\n**Bull (Round {}):**\n{resp}", round + 1));
        current_response = resp.clone();
        result.bull_arguments = resp;

        vars.insert("history".into(), debate_history.clone());
        vars.insert("current_response".into(), current_response.clone());

        let bear_id = format!("bear_researcher_r{}", round + 1);
        let resp = run_or_load_step(
            &app, pool_ref, quick, &symbol, &today,
            &bear_id, "Bear Researcher", "debate", &prompt, &vars,
            pipeline_config.cooldown_secs, &cached_steps,
        ).await?;
        debate_history.push_str(&format!("\n\n**Bear (Round {}):**\n{resp}", round + 1));
        current_response = resp.clone();
        result.bear_arguments = resp;
    }

    // ── Phase 3: Research Manager ──
    {
        let mut vars = HashMap::new();
        vars.insert("history".into(), debate_history.clone());
        vars.insert("past_memory_str".into(), String::new());
        vars.insert("instrument_context".into(), format!("Stock: {symbol}"));

        result.investment_decision = run_or_load_step(
            &app, pool_ref, deep, &symbol, &today,
            "research_manager", "Research Manager", "decision", &prompt, &vars,
            pipeline_config.cooldown_secs, &cached_steps,
        ).await?;
    }

    // ── Phase 4: Trader ──
    {
        let mut vars = HashMap::new();
        vars.insert("past_memory_str".into(), String::new());
        let trader_prompt = format!(
            "Based on the investment plan below, make your trading decision for {symbol}.\n\nInvestment Plan:\n{}",
            result.investment_decision
        );
        result.trader_plan = run_or_load_step(
            &app, pool_ref, deep, &symbol, &today,
            "trader", "Trader", "decision", &trader_prompt, &vars,
            pipeline_config.cooldown_secs, &cached_steps,
        ).await?;
    }

    // ── Phase 5: Risk Management Debate ──
    let mut risk_history = String::new();
    let mut agg_resp = String::new();
    let mut con_resp = String::new();
    let mut neu_resp = String::new();

    for round in 0..pipeline_config.max_risk_rounds {
        let mut vars = HashMap::new();
        vars.insert("trader_decision".into(), result.trader_plan.clone());
        vars.insert("market_research_report".into(), result.market_report.clone());
        vars.insert("sentiment_report".into(), result.social_report.clone());
        vars.insert("news_report".into(), result.news_report.clone());
        vars.insert("fundamentals_report".into(), result.fundamentals_report.clone());
        vars.insert("history".into(), risk_history.clone());
        vars.insert("current_conservative_response".into(), con_resp.clone());
        vars.insert("current_neutral_response".into(), neu_resp.clone());

        let agg_id = format!("aggressive_risk_r{}", round + 1);
        let resp = run_or_load_step(
            &app, pool_ref, quick, &symbol, &today,
            &agg_id, "Aggressive Analyst", "risk", &prompt, &vars,
            pipeline_config.cooldown_secs, &cached_steps,
        ).await?;
        risk_history.push_str(&format!("\n\n**Aggressive (Round {}):**\n{resp}", round + 1));
        agg_resp = resp.clone();
        result.risk_aggressive = resp;

        vars.insert("history".into(), risk_history.clone());
        vars.insert("current_aggressive_response".into(), agg_resp.clone());

        let con_id = format!("conservative_risk_r{}", round + 1);
        let resp = run_or_load_step(
            &app, pool_ref, quick, &symbol, &today,
            &con_id, "Conservative Analyst", "risk", &prompt, &vars,
            pipeline_config.cooldown_secs, &cached_steps,
        ).await?;
        risk_history.push_str(&format!("\n\n**Conservative (Round {}):**\n{resp}", round + 1));
        con_resp = resp.clone();
        result.risk_conservative = resp;

        vars.insert("history".into(), risk_history.clone());
        vars.insert("current_conservative_response".into(), con_resp.clone());

        let neu_id = format!("neutral_risk_r{}", round + 1);
        let resp = run_or_load_step(
            &app, pool_ref, quick, &symbol, &today,
            &neu_id, "Neutral Analyst", "risk", &prompt, &vars,
            pipeline_config.cooldown_secs, &cached_steps,
        ).await?;
        risk_history.push_str(&format!("\n\n**Neutral (Round {}):**\n{resp}", round + 1));
        neu_resp = resp.clone();
        result.risk_neutral = resp;
    }

    // ── Phase 6: Portfolio Manager ──
    {
        let mut vars = HashMap::new();
        vars.insert("instrument_context".into(), format!("Stock: {symbol}"));
        vars.insert("trader_plan".into(), result.trader_plan.clone());
        vars.insert("past_memory_str".into(), String::new());
        vars.insert("history".into(), risk_history.clone());

        result.final_decision = run_or_load_step(
            &app, pool_ref, deep, &symbol, &today,
            "portfolio_manager", "Portfolio Manager", "final", &prompt, &vars,
            pipeline_config.cooldown_secs, &cached_steps,
        ).await?;
        result.signal = extract_signal(&result.final_decision);
    }

    // ── Save to DB + JSON ──
    let pool = app.state::<SqlitePool>();
    let data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
    let reports_dir = data_dir.join("reports");
    std::fs::create_dir_all(&reports_dir).ok();

    let filename = format!(
        "{}_{}.json",
        symbol.replace('.', "_"),
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    );
    let report_path = reports_dir.join(&filename);

    if let Ok(json) = serde_json::to_string_pretty(&result) {
        std::fs::write(&report_path, &json).ok();
    }

    let _ = db::analysis::save_analysis(
        pool.inner(),
        &symbol,
        &result.signal,
        &report_path.to_string_lossy(),
    )
    .await;

    emit_progress(&app, "complete", "", "done", Some(&result.signal));
    Ok(result)
}

#[tauri::command]
pub async fn get_analysis_history(
    pool: tauri::State<'_, SqlitePool>,
    limit: Option<i64>,
) -> Result<Vec<db::analysis::AnalysisRecord>, String> {
    db::analysis::get_analysis_history(pool.inner(), limit.unwrap_or(20))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_analysis_detail(
    pool: tauri::State<'_, SqlitePool>,
    id: i64,
) -> Result<AnalysisResult, String> {
    let record = db::analysis::get_analysis_by_id(pool.inner(), id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Analysis not found")?;

    let content = std::fs::read_to_string(&record.report_path)
        .map_err(|e| format!("Failed to read report: {e}"))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse report: {e}"))
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
