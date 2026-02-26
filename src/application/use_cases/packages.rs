use std::path::PathBuf;
use std::sync::Arc;

use tokio::io::AsyncWriteExt;

use crate::domain::{Package, PackageRepository, PackageVersion, generate_uid};
use crate::app_error::{AppError, AppResult};

/// Allowed file extensions for package version files.
const ALLOWED_EXTENSIONS: &[&str] = &[".zip", ".unitypackage", ".tar.gz", ".tgz"];

fn validate_file_name(file_name: &str) -> AppResult<()> {
    if file_name.is_empty() || file_name.len() > 255 {
        return Err(AppError::Internal("Invalid file name length".into()));
    }
    if file_name.contains('/') || file_name.contains('\\') || file_name.contains("..") {
        return Err(AppError::Internal("Invalid characters in file name".into()));
    }
    let lower = file_name.to_ascii_lowercase();
    if !ALLOWED_EXTENSIONS.iter().any(|ext| lower.ends_with(ext)) {
        return Err(AppError::Internal(
            format!("File name must end with one of: {}", ALLOWED_EXTENSIONS.join(", ")),
        ));
    }
    Ok(())
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

    /// Upload a version file, streaming chunks to disk.
    /// Creates the package subdirectory on demand.
    /// Overwrites existing version if it already exists.
    pub async fn upload_version(
        &self,
        uid: &str,
        version: &str,
        file_name: &str,
        chunks: &mut (dyn ChunkReader + Send),
    ) -> AppResult<()> {
        validate_file_name(file_name)?;

        // Ensure the package exists.
        self.db.get_by_uid(uid).await?
            .ok_or(AppError::NotFound)?;

        // Resolve and create target directory: {packages_dir}/{uid}/
        let dir = self.packages_dir.join(uid);
        tokio::fs::create_dir_all(&dir).await
            .map_err(|e| AppError::Internal(format!("Failed to create package directory: {e}")))?;

        let file_path = dir.join(file_name);

        // Write to a uniquely-named temp file, then rename for atomic replacement.
        // Random suffix prevents races if two uploads target the same version concurrently.
        let tmp_name = format!(".upload-{}.tmp", uuid::Uuid::new_v4());
        let tmp_path = dir.join(tmp_name);

        match self.write_and_finalize(&tmp_path, &file_path, chunks).await {
            Ok(()) => {}
            Err(e) => {
                let _ = tokio::fs::remove_file(&tmp_path).await;
                return Err(e);
            }
        }

        // Upsert DB record (insert or update if version exists).
        self.db.upsert_version(uid, version, file_name).await?;

        Ok(())
    }

    async fn write_and_finalize(
        &self,
        tmp_path: &std::path::Path,
        final_path: &std::path::Path,
        chunks: &mut (dyn ChunkReader + Send),
    ) -> AppResult<()> {
        let mut file = tokio::fs::File::create(tmp_path).await
            .map_err(|e| AppError::Internal(format!("Failed to create temp file: {e}")))?;

        while let Some(chunk) = chunks.next_chunk().await? {
            file.write_all(&chunk).await
                .map_err(|e| AppError::Internal(format!("Failed to write chunk: {e}")))?;
        }

        file.sync_all().await
            .map_err(|e| AppError::Internal(format!("Failed to sync file to disk: {e}")))?;
        drop(file);

        tokio::fs::rename(tmp_path, final_path).await
            .map_err(|e| AppError::Internal(format!("Failed to finalize file: {e}")))?;

        Ok(())
    }

    pub async fn get_versions(&self, uid: &str) -> AppResult<Vec<PackageVersion>> {
        self.db.get_versions(uid).await
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

/// Abstraction for reading file chunks from a multipart upload.
/// Implemented by the adapter layer to avoid coupling the use case to axum.
#[async_trait::async_trait]
pub trait ChunkReader {
    async fn next_chunk(&mut self) -> AppResult<Option<bytes::Bytes>>;
}
