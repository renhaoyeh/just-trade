CREATE TABLE IF NOT EXISTS analysis_steps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    trade_date TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    phase TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(symbol, trade_date, agent_id)
);

CREATE INDEX IF NOT EXISTS idx_analysis_steps_lookup
    ON analysis_steps (symbol, trade_date);
