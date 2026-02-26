use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Package {
    pub id: i64,
    pub name: String,
    pub uid: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PackageVersion {
    pub id: i64,
    pub version: String,
    pub file_name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketCredentials {
    pub market: String,
    pub base_url: String,
    pub api_key: String,
    pub active: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct License {
    pub id: i64,
    pub license: String,
    pub token: String,
    pub package_id: i64,
    pub package_name: String,
    pub package_uid: String,
    pub source: String,
    pub active: bool,
    pub deleted: bool,
    pub created_at: String,
}
