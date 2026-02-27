use std::path::{Path, PathBuf};
use std::sync::Arc;

use sha2::{Sha256, Digest};
use tokio::io::AsyncWriteExt;

use crate::domain::{Package, PackageRepository, PackageVersion, extract_manifest};
use crate::app_error::{AppError, AppResult};

#[derive(Debug, Clone, serde::Serialize)]
pub struct UploadResult {
    pub uid: String,
    pub version: String,
    pub display_name: String,
}

fn compute_sha256(path: &Path) -> AppResult<String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)
        .map_err(|e| AppError::Internal(format!("Failed to open file for hashing: {e}")))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)
            .map_err(|e| AppError::Internal(format!("Failed to read file for hashing: {e}")))?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[derive(Clone)]
pub struct PackageUseCases {
    db: Arc<dyn PackageRepository>,
    packages_dir: PathBuf,
}

impl PackageUseCases {
    pub fn new(db: Arc<dyn PackageRepository>, packages_dir: PathBuf) -> Self {
        Self { db, packages_dir }
    }

    pub async fn list(&self) -> AppResult<Vec<Package>> {
        self.db.list().await
    }

    pub async fn delete(&self, uid: &str) -> AppResult<()> {
        self.db.delete(uid).await?;
        let dir = self.packages_dir.join(uid);
        if dir.exists() {
            tokio::fs::remove_dir_all(&dir).await
                .map_err(|e| AppError::Internal(format!("Failed to remove package directory: {e}")))?;
        }
        Ok(())
    }

    pub async fn link_market(&self, uid: &str, market: &str, product_id: &str) -> AppResult<()> {
        self.db.link_market(uid, market, product_id).await
    }

    pub async fn unlink_market(&self, uid: &str, market: &str, product_id: &str) -> AppResult<()> {
        self.db.unlink_market(uid, market, product_id).await
    }

    pub async fn get_market_links(&self, uid: &str) -> AppResult<Vec<(String, String)>> {
        self.db.get_market_links(uid).await
    }

    /// Upload a .zip file. Extracts package.json from the zip to derive
    /// UID, display name, version, and full manifest. Auto-creates the
    /// package on first upload. Enforces semver. Syncs display_name from
    /// the latest version's manifest.
    pub async fn upload_version(
        &self,
        file_name: &str,
        chunks: &mut (dyn ChunkReader + Send),
    ) -> AppResult<UploadResult> {
        // Validate extension
        let lower = file_name.to_ascii_lowercase();
        if !lower.ends_with(".zip") {
            return Err(AppError::Internal("Only .zip files are accepted".into()));
        }

        // Write chunks to temp file
        let tmp_dir = self.packages_dir.join(".tmp");
        tokio::fs::create_dir_all(&tmp_dir).await
            .map_err(|e| AppError::Internal(format!("Failed to create temp directory: {e}")))?;

        let tmp_name = format!(".upload-{}.tmp", uuid::Uuid::new_v4());
        let tmp_path = tmp_dir.join(&tmp_name);

        if let Err(e) = self.write_chunks(&tmp_path, chunks).await {
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(e);
        }

        // Extract manifest from zip (blocking) — validates semver
        let tmp_path_clone = tmp_path.clone();
        let manifest = tokio::task::spawn_blocking(move || extract_manifest(&tmp_path_clone))
            .await
            .map_err(|e| AppError::Internal(format!("spawn_blocking failed: {e}")))?
            .map_err(|e| {
                let p = tmp_path.clone();
                tokio::spawn(async move { let _ = tokio::fs::remove_file(&p).await; });
                e
            })?;

        // Compute SHA-256 (blocking)
        let tmp_path_for_hash = tmp_path.clone();
        let sha256 = tokio::task::spawn_blocking(move || compute_sha256(&tmp_path_for_hash))
            .await
            .map_err(|e| AppError::Internal(format!("spawn_blocking failed: {e}")))?
            .map_err(|e| {
                let p = tmp_path.clone();
                tokio::spawn(async move { let _ = tokio::fs::remove_file(&p).await; });
                e
            })?;

        // Get or create package
        let package = self.db.get_or_create(&manifest.name).await?;

        // Move temp file to final location
        let dir = self.packages_dir.join(&package.uid);
        tokio::fs::create_dir_all(&dir).await
            .map_err(|e| AppError::Internal(format!("Failed to create package directory: {e}")))?;

        let final_name = format!("{}-v{}.zip", package.uid, manifest.version);
        let final_path = dir.join(&final_name);

        tokio::fs::rename(&tmp_path, &final_path).await
            .map_err(|e| AppError::Internal(format!("Failed to finalize file: {e}")))?;

        // Store manifest JSON as string
        let manifest_json_str = serde_json::to_string(&manifest.full_json)
            .map_err(|e| AppError::Internal(format!("Failed to serialize manifest: {e}")))?;

        // Upsert version in DB
        self.db.upsert_version(
            &package.uid,
            &manifest.version,
            &final_name,
            &manifest_json_str,
            &sha256,
        ).await?;

        // Sync display_name from latest version
        self.db.sync_display_name(&package.uid).await?;

        Ok(UploadResult {
            uid: package.uid,
            version: manifest.version,
            display_name: manifest.display_name,
        })
    }

    async fn write_chunks(
        &self,
        path: &Path,
        chunks: &mut (dyn ChunkReader + Send),
    ) -> AppResult<()> {
        let mut file = tokio::fs::File::create(path).await
            .map_err(|e| AppError::Internal(format!("Failed to create temp file: {e}")))?;

        while let Some(chunk) = chunks.next_chunk().await? {
            file.write_all(&chunk).await
                .map_err(|e| AppError::Internal(format!("Failed to write chunk: {e}")))?;
        }

        file.sync_all().await
            .map_err(|e| AppError::Internal(format!("Failed to sync file to disk: {e}")))?;

        Ok(())
    }

    pub async fn get_versions(&self, uid: &str) -> AppResult<Vec<PackageVersion>> {
        let mut versions = self.db.get_versions(uid).await?;
        // Sort by semver descending
        versions.sort_by(|a, b| {
            let va = semver::Version::parse(&a.version).ok();
            let vb = semver::Version::parse(&b.version).ok();
            vb.cmp(&va)
        });
        Ok(versions)
    }

    pub async fn delete_version(&self, uid: &str, version: &str) -> AppResult<()> {
        // Get the version's file_name before deleting from DB
        let versions = self.db.get_versions(uid).await?;
        let file_name = versions.iter()
            .find(|v| v.version == version)
            .map(|v| v.file_name.clone());

        self.db.delete_version(uid, version).await?;

        // Sync display_name in case latest was deleted
        self.db.sync_display_name(uid).await?;

        // Remove file from disk
        if let Some(name) = file_name {
            let file_path = self.packages_dir.join(uid).join(&name);
            let _ = tokio::fs::remove_file(&file_path).await;
        }

        Ok(())
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

/// Abstraction for reading file chunks from a multipart upload.
/// Implemented by the adapter layer to avoid coupling the use case to axum.
#[async_trait::async_trait]
pub trait ChunkReader {
    async fn next_chunk(&mut self) -> AppResult<Option<bytes::Bytes>>;
}
