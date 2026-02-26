use async_trait::async_trait;

use crate::{
    adapters::queries::SqliteDatabase,
    app_error::{AppError, AppResult},
    domain::{Package, PackageRepository, PackageVersion},
};

#[derive(Debug, Clone, sqlx::FromRow)]
struct PackageRow {
    pub id: i64,
    pub name: String,
    pub uid: String,
    pub created_at: String,
}

impl From<PackageRow> for Package {
    fn from(row: PackageRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
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
    pub created_at: String,
}

impl From<VersionRow> for PackageVersion {
    fn from(row: VersionRow) -> Self {
        Self {
            id: row.id,
            version: row.version,
            file_name: row.file_name,
            created_at: row.created_at,
        }
    }
}

#[async_trait]
impl PackageRepository for SqliteDatabase {
    async fn create(&self, name: &str, uid: &str) -> AppResult<()> {
        sqlx::query("INSERT INTO packages (name, uid) VALUES ($1, $2)")
            .bind(name)
            .bind(uid)
            .execute(&self.pool)
            .await
            .map_err(AppError::from)?;
        Ok(())
    }

    async fn change_name(&self, uid: &str, name: &str) -> AppResult<()> {
        sqlx::query("UPDATE packages SET name = $2 WHERE uid = $1")
            .bind(uid)
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(AppError::from)?;
        Ok(())
    }

    async fn list(&self) -> AppResult<Vec<Package>> {
        let rows = sqlx::query_as::<_, PackageRow>(
            "SELECT id, name, uid, created_at FROM packages ORDER BY name",
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

    async fn upsert_version(&self, uid: &str, version: &str, file_name: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO package_versions (package_id, version, file_name)
            SELECT id, $2, $3 FROM packages WHERE uid = $1
            ON CONFLICT(package_id, version) DO UPDATE SET file_name = $3
            "#,
        )
        .bind(uid)
        .bind(version)
        .bind(file_name)
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }

    async fn get_versions(&self, uid: &str) -> AppResult<Vec<PackageVersion>> {
        let rows = sqlx::query_as::<_, VersionRow>(
            r#"
            SELECT pv.id, pv.version, pv.file_name, pv.created_at
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
            "SELECT id, name, uid, created_at FROM packages WHERE uid = $1",
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
            SELECT p.id, p.name, p.uid, p.created_at
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
