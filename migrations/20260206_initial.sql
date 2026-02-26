-- Packages table
CREATE TABLE packages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,                    -- "My Cool Avatar"
    uid TEXT NOT NULL UNIQUE,             -- "a1b2c3d4e5"
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Package versions
CREATE TABLE package_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    package_id INTEGER NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
    version TEXT NOT NULL,                 -- "1.0.0"
    file_name TEXT NOT NULL,               -- "my-cool-avatar-1.0.0.zip"
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(package_id, version)
);

-- Market product mappings
CREATE TABLE package_markets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    package_id INTEGER NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
    market TEXT NOT NULL,                  -- "payhip"
    product_id TEXT NOT NULL,              -- "mVT0"
    UNIQUE(market, product_id)
);

-- Licenses
CREATE TABLE licenses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    license TEXT NOT NULL UNIQUE,          -- The key
    token TEXT NOT NULL UNIQUE,            -- VPM access token
    package_id INTEGER NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
    source TEXT NOT NULL,                  -- "gift", "payhip"
    active BOOLEAN NOT NULL DEFAULT 1,
    deleted BOOLEAN NOT NULL DEFAULT 0,
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
