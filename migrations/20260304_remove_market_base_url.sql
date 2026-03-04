-- SQLite doesn't support DROP COLUMN before 3.35, so recreate the table
CREATE TABLE market_configs_new (
    market     TEXT PRIMARY KEY,
    api_key    TEXT NOT NULL,
    active     BOOLEAN NOT NULL DEFAULT 1,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO market_configs_new (market, api_key, active, updated_at)
SELECT market, api_key, active, updated_at FROM market_configs;

DROP TABLE market_configs;

ALTER TABLE market_configs_new RENAME TO market_configs;
