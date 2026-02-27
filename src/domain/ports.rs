use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::app_error::AppResult;
use super::entities::{License, MarketCredentials, Package, PackageVersion};

#[async_trait]
pub trait GiftRepository: Send + Sync {
    async fn code_create(&self, code: &str, token: &str, uid: &str, created_at: &DateTime<Utc>) -> AppResult<()>;
}

#[async_trait]
pub trait LicenseRepository: Send + Sync {
    async fn get(&self, license: &str) -> AppResult<Option<String>>;
    async fn list(&self, cursor: &i64, page_size: &i64) -> AppResult<Vec<License>>;
    async fn register(&self, license: &str, token: &str, uid: &str, source: &str, created_at: &DateTime<Utc>) -> AppResult<()>;
    async fn update(&self, license: &str, active: bool) -> AppResult<()>;
    async fn delete(&self, license: &str) -> AppResult<()>;
    async fn increment_use_count(&self, token: &str) -> AppResult<()>;
    async fn get_package_uid_by_market_product(&self, market: &str, product_id: &str) -> AppResult<Option<String>>;
}

#[async_trait]
pub trait PackageRepository: Send + Sync {
    async fn list(&self) -> AppResult<Vec<Package>>;
    async fn delete(&self, uid: &str) -> AppResult<()>;
    async fn link_market(&self, uid: &str, market: &str, product_id: &str) -> AppResult<()>;
    async fn unlink_market(&self, uid: &str, market: &str, product_id: &str) -> AppResult<()>;
    async fn get_market_links(&self, uid: &str) -> AppResult<Vec<(String, String)>>;

    async fn get_or_create(&self, uid: &str) -> AppResult<Package>;
    async fn sync_display_name(&self, uid: &str) -> AppResult<()>;
    async fn upsert_version(&self, uid: &str, version: &str, file_name: &str, manifest_json: &str, zip_sha256: &str) -> AppResult<()>;
    async fn get_versions(&self, uid: &str) -> AppResult<Vec<PackageVersion>>;
    async fn delete_version(&self, uid: &str, version: &str) -> AppResult<()>;

    async fn get_by_uid(&self, uid: &str) -> AppResult<Option<Package>>;
    async fn token_has_access(&self, uid: &str, token: &str) -> AppResult<bool>;
    async fn get_package_for_token(&self, token: &str) -> AppResult<Option<Package>>;
}

#[async_trait]
pub trait MarketConfigRepository: Send + Sync {
    async fn list_all(&self) -> AppResult<Vec<MarketCredentials>>;
    async fn get(&self, market: &str) -> AppResult<Option<MarketCredentials>>;
    async fn upsert(&self, creds: &MarketCredentials) -> AppResult<()>;
    async fn delete(&self, market: &str) -> AppResult<()>;
}

pub struct VerifyResult {
    pub market: String,
    pub product_id: String,
}

#[async_trait]
pub trait MarketPort: Send + Sync {
    fn name(&self) -> &'static str;
    fn check_format(&self, key: &str) -> bool;
    async fn verify_key(&self, key: &str) -> AppResult<Option<String>>;
    async fn decrement_use(&self, _key: &str) -> AppResult<()> { Ok(()) }
}
