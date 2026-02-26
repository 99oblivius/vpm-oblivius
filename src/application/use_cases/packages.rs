use std::sync::Arc;

use crate::domain::{Package, PackageRepository, PackageVersion, generate_uid};
use crate::app_error::{AppError, AppResult};

#[derive(Clone)]
pub struct PackageUseCases {
    db: Arc<dyn PackageRepository>,
}

impl PackageUseCases {
    pub fn new(db: Arc<dyn PackageRepository>) -> Self {
        Self { db }
    }

    pub async fn create(&self, name: &str) -> AppResult<Package> {
        let uid = generate_uid();
        self.db.create(name, &uid).await?;
        self.db.get_by_uid(&uid).await?.ok_or(AppError::Internal("Failed to create package".into()))
    }

    pub async fn rename(&self, uid: &str, name: &str) -> AppResult<()> {
        self.db.change_name(uid, name).await
    }

    pub async fn list(&self) -> AppResult<Vec<Package>> {
        self.db.list().await
    }

    pub async fn delete(&self, uid: &str) -> AppResult<()> {
        self.db.delete(uid).await
    }

    pub async fn link_market(&self, uid: &str, market: &str, product_id: &str) -> AppResult<()> {
        self.db.link_market(uid, market, product_id).await
    }

    pub async fn add_version(&self, uid: &str, version: &str, file_name: &str) -> AppResult<()> {
        self.db.add_version(uid, version, file_name).await
    }

    pub async fn get_versions(&self, uid: &str) -> AppResult<Vec<PackageVersion>> {
        self.db.get_versions(uid).await
    }

    pub async fn update_version(&self, uid: &str, version: &str, file_name: &str) -> AppResult<()> {
        self.db.update_version(uid, version, file_name).await
    }

    pub async fn delete_version(&self, uid: &str, version: &str) -> AppResult<()> {
        self.db.delete_version(uid, version).await
    }

    pub async fn get_by_uid(&self, uid: &str) -> AppResult<Option<Package>> {
        self.db.get_by_uid(uid).await
    }

    pub async fn verify_access(&self, uid: &str, token: &str) -> AppResult<()> {
        if self.db.token_has_access(uid, token).await? {
            Ok(())
        } else {
            Err(AppError::InvalidCredentials)
        }
    }

    pub async fn get_package_for_token(&self, token: &str) -> AppResult<Option<Package>> {
        self.db.get_package_for_token(token).await
    }
}
