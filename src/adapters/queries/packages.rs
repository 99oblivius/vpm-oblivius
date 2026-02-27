use async_trait::async_trait;

use crate::{
    adapters::queries::SqliteDatabase,
    app_error::{AppError, AppResult},
    domain::{Package, PackageRepository, PackageVersion},
};

#[derive(Debug, Clone, sqlx::FromRow)]
struct PackageRow {
    pub id: i64,
    pub display_name: String,
    pub uid: String,
    pub created_at: String,
}

impl From<PackageRow> for Package {
    fn from(row: PackageRow) -> Self {
        Self {
            id: row.id,
            display_name: row.display_name,
            uid: row.uid,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct VersionRow {
    pub id: i64,
    pub version: String,
    pub file_name: String,
    pub manifest_json: String,
    pub zip_sha256: String,
    pub created_at: String,
}

impl From<VersionRow> for PackageVersion {
    fn from(row: VersionRow) -> Self {
        Self {
            id: row.id,
            version: row.version,
            file_name: row.file_name,
            manifest_json: row.manifest_json,
            zip_sha256: row.zip_sha256,
            created_at: row.created_at,
        }
    }
}

#[async_trait]
impl PackageRepository for SqliteDatabase {
    async fn list(&self) -> AppResult<Vec<Package>> {
        let rows = sqlx::query_as::<_, PackageRow>(
            "SELECT id, display_name, uid, created_at FROM packages ORDER BY display_name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows.into_iter().map(Package::from).collect())
    }

    async fn delete(&self, uid: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM packages WHERE uid = $1")
            .bind(uid)
            .execute(&self.pool)
            .await
            .map_err(AppError::from)?;
        Ok(())
    }

    async fn link_market(&self, uid: &str, market: &str, product_id: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO package_markets (package_id, market, product_id)
            SELECT id, $2, $3 FROM packages WHERE uid = $1
            "#,
        )
        .bind(uid)
        .bind(market)
        .bind(product_id)
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }

    async fn unlink_market(&self, uid: &str, market: &str, product_id: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            DELETE FROM package_markets
            WHERE package_id = (SELECT id FROM packages WHERE uid = $1)
              AND market = $2
              AND product_id = $3
            "#,
        )
        .bind(uid)
        .bind(market)
        .bind(product_id)
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }

    async fn get_market_links(&self, uid: &str) -> AppResult<Vec<(String, String)>> {
        let rows = sqlx::query_as::<_, (String, String)>(
            r#"
            SELECT pm.market, pm.product_id
            FROM package_markets pm
            JOIN packages p ON pm.package_id = p.id
            WHERE p.uid = $1
            ORDER BY pm.market
            "#,
        )
        .bind(uid)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;
        Ok(rows)
    }

    async fn get_or_create(&self, uid: &str) -> AppResult<Package> {
        sqlx::query("INSERT OR IGNORE INTO packages (uid) VALUES ($1)")
            .bind(uid)
            .execute(&self.pool)
            .await
            .map_err(AppError::from)?;
        self.get_by_uid(uid)
            .await?
            .ok_or_else(|| AppError::Internal("Failed to get_or_create package".into()))
    }

    async fn sync_display_name(&self, uid: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE packages SET display_name = COALESCE(
                (SELECT json_extract(pv.manifest_json, '$.displayName')
                 FROM package_versions pv
                 WHERE pv.package_id = packages.id
                 ORDER BY pv.created_at DESC LIMIT 1),
                display_name
            ) WHERE uid = $1
            "#,
        )
        .bind(uid)
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }

    async fn upsert_version(&self, uid: &str, version: &str, file_name: &str, manifest_json: &str, zip_sha256: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO package_versions (package_id, version, file_name, manifest_json, zip_sha256)
            SELECT id, $2, $3, $4, $5 FROM packages WHERE uid = $1
            ON CONFLICT(package_id, version) DO UPDATE SET file_name = $3, manifest_json = $4, zip_sha256 = $5, created_at = datetime('now')
            "#,
        )
        .bind(uid)
        .bind(version)
        .bind(file_name)
        .bind(manifest_json)
        .bind(zip_sha256)
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }

    async fn get_versions(&self, uid: &str) -> AppResult<Vec<PackageVersion>> {
        let rows = sqlx::query_as::<_, VersionRow>(
            r#"
            SELECT pv.id, pv.version, pv.file_name, pv.manifest_json, pv.zip_sha256, pv.created_at
            FROM package_versions pv
            JOIN packages p ON pv.package_id = p.id
            WHERE p.uid = $1
            ORDER BY pv.created_at DESC
            "#,
        )
        .bind(uid)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows.into_iter().map(PackageVersion::from).collect())
    }

    async fn delete_version(&self, uid: &str, version: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            DELETE FROM package_versions
            WHERE package_id = (SELECT id FROM packages WHERE uid = $1)
              AND version = $2
            "#,
        )
        .bind(uid)
        .bind(version)
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }

    async fn get_by_uid(&self, uid: &str) -> AppResult<Option<Package>> {
        let row = sqlx::query_as::<_, PackageRow>(
            "SELECT id, display_name, uid, created_at FROM packages WHERE uid = $1",
        )
        .bind(uid)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(row.map(Package::from))
    }

    async fn token_has_access(&self, uid: &str, token: &str) -> AppResult<bool> {
        let result = sqlx::query(
            r#"
            SELECT 1 FROM licenses l
            JOIN packages p ON l.package_id = p.id
            WHERE l.token = $1
              AND p.uid = $2
              AND l.active = 1
              AND l.deleted = 0
            "#,
        )
        .bind(token)
        .bind(uid)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(result.is_some())
    }

    async fn get_package_for_token(&self, token: &str) -> AppResult<Option<Package>> {
        let row = sqlx::query_as::<_, PackageRow>(
            r#"
            SELECT p.id, p.display_name, p.uid, p.created_at
            FROM packages p
            JOIN licenses l ON l.package_id = p.id
            WHERE l.token = $1 AND l.active = 1 AND l.deleted = 0
            "#,
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(row.map(Package::from))
    }
}
