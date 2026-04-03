CREATE TABLE IF NOT EXISTS analysis_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    symbol TEXT NOT NULL,
    signal TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    report_path TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_analysis_history_symbol
    ON analysis_history (symbol);

CREATE INDEX IF NOT EXISTS idx_analysis_history_created
    ON analysis_history (created_at DESC);
