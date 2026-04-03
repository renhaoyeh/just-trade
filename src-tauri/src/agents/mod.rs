pub mod analysts;
pub mod managers;
pub mod researchers;
pub mod risk_mgmt;
pub mod trader;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An agent definition with its system prompt template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDef {
    pub id: &'static str,
    pub name: &'static str,
    pub category: AgentCategory,
    pub prompt_template: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentCategory {
    Analyst,
    Researcher,
    RiskManagement,
    Manager,
    Trader,
}

impl AgentDef {
    /// Render the prompt template by replacing `{key}` placeholders with values.
    pub fn render(&self, vars: &HashMap<String, String>) -> String {
        let mut result = self.prompt_template.to_string();
        for (key, value) in vars {
            result = result.replace(&format!("{{{key}}}"), value);
        }
        result
    }
}

/// All available agent definitions
pub const ALL_AGENTS: &[&AgentDef] = &[
    // Analysts
    &analysts::fundamentals::AGENT,
    &analysts::market::AGENT,
    &analysts::news::AGENT,
    &analysts::social_media::AGENT,
    // Researchers
    &researchers::bull::AGENT,
    &researchers::bear::AGENT,
    // Risk Management
    &risk_mgmt::aggressive::AGENT,
    &risk_mgmt::conservative::AGENT,
    &risk_mgmt::neutral::AGENT,
    // Managers
    &managers::research::AGENT,
    &managers::portfolio::AGENT,
    // Trader
    &trader::AGENT,
];

/// Look up an agent by ID
pub fn get_agent(id: &str) -> Option<&'static AgentDef> {
    ALL_AGENTS.iter().find(|a| a.id == id).copied()
}
