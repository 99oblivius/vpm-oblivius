-- Packages table
CREATE TABLE packages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    display_name TEXT NOT NULL DEFAULT '',
    uid TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Package versions
CREATE TABLE package_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    package_id INTEGER NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
    version TEXT NOT NULL,
    file_name TEXT NOT NULL,
    manifest_json TEXT NOT NULL DEFAULT '{}',
    zip_sha256 TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(package_id, version)
);

-- Market product mappings
CREATE TABLE package_markets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    package_id INTEGER NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
    market TEXT NOT NULL,
    product_id TEXT NOT NULL,
    UNIQUE(market, product_id)
);

-- Licenses
CREATE TABLE licenses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    license TEXT NOT NULL UNIQUE,
    token TEXT NOT NULL UNIQUE,
    package_id INTEGER NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
    source TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT 1,
    deleted BOOLEAN NOT NULL DEFAULT 0,
    use_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Market Settings and Credentials
CREATE TABLE market_configs (
    market TEXT PRIMARY KEY,
    base_url TEXT NOT NULL,
    api_key TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT 1,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes
CREATE INDEX idx_licenses_token ON licenses(token);
CREATE INDEX idx_licenses_license ON licenses(license);
CREATE INDEX idx_package_markets_lookup ON package_markets(market, product_id);
