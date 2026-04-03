use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Data vendor configuration.
///
/// Mirrors TradingAgents' `default_config.py` data_vendors / tool_vendors pattern.
/// Category-level defaults with optional tool-level overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataflowConfig {
    /// Default vendor per data category
    pub data_vendors: HashMap<String, String>,
    /// Tool-level vendor overrides (takes precedence)
    pub tool_vendors: HashMap<String, String>,
}

impl Default for DataflowConfig {
    fn default() -> Self {
        let mut data_vendors = HashMap::new();
        data_vendors.insert("core_stock_apis".into(), "yahoo".into());
        data_vendors.insert("technical_indicators".into(), "yahoo".into());
        data_vendors.insert("fundamental_data".into(), "yahoo".into());
        data_vendors.insert("news_data".into(), "yahoo".into());
        data_vendors.insert("sector_data".into(), "twse".into());

        Self {
            data_vendors,
            tool_vendors: HashMap::new(),
        }
    }
}

impl DataflowConfig {
    /// Get the configured vendor for a data category, with tool-level override.
    pub fn get_vendor(&self, category: &str, tool: Option<&str>) -> String {
        // Tool-level takes precedence
        if let Some(t) = tool {
            if let Some(v) = self.tool_vendors.get(t) {
                return v.clone();
            }
        }
        // Fall back to category-level
        self.data_vendors
            .get(category)
            .cloned()
            .unwrap_or_else(|| "yahoo".into())
    }
}
