CREATE TABLE IF NOT EXISTS twse_companies (
    code TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    industry_category TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_twse_companies_industry
    ON twse_companies (industry_category);
