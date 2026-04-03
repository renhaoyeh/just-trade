/// Unified data interface — routes data requests to the configured vendor.
///
/// Mirrors TradingAgents' `interface.py`:
/// - Tools organized by category
/// - Vendor routing with fallback support
/// - Category → vendor mapping from config

/// Tool categories and their methods.
pub const TOOLS_CATEGORIES: &[(&str, &str, &[&str])] = &[
    (
        "core_stock_apis",
        "OHLCV stock price data",
        &["fetch_stock_history"],
    ),
    (
        "technical_indicators",
        "Technical analysis indicators",
        &["get_indicators"],
    ),
    (
        "fundamental_data",
        "Company fundamentals",
        &["get_stock_info"],
    ),
    (
        "news_data",
        "News and sentiment",
        &["fetch_stock_news"],
    ),
    (
        "sector_data",
        "Industry sector classification",
        &["get_sectors", "get_sector_stocks"],
    ),
];

/// Supported data vendors.
pub const VENDOR_LIST: &[&str] = &["yahoo", "twse"];

/// Look up which category a tool method belongs to.
pub fn get_category_for_method(method: &str) -> Option<&'static str> {
    for (category, _, tools) in TOOLS_CATEGORIES {
        if tools.contains(&method) {
            return Some(category);
        }
    }
    None
}
