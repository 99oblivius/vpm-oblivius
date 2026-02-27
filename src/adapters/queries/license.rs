use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    adapters::queries::SqliteDatabase,
    app_error::{AppError, AppResult},
    domain::{License, LicenseRepository},
};

#[derive(Debug, Clone, sqlx::FromRow)]
struct LicenseRow {
    pub id: i64,
    pub license: String,
    pub token: String,
    pub package_id: i64,
    pub package_display_name: String,
    pub package_uid: String,
    pub source: String,
    pub active: bool,
    pub deleted: bool,
    pub use_count: i64,
    pub created_at: String,
}

impl From<LicenseRow> for License {
    fn from(row: LicenseRow) -> Self {
        Self {
            id: row.id,
            license: row.license,
            token: row.token,
            package_id: row.package_id,
            package_display_name: row.package_display_name,
            package_uid: row.package_uid,
            source: row.source,
            active: row.active,
            deleted: row.deleted,
            use_count: row.use_count,
            created_at: row.created_at,
        }
    }
}

#[async_trait]
impl LicenseRepository for SqliteDatabase {
    async fn get(&self, license: &str) -> AppResult<Option<String>> {
        let result = sqlx::query_scalar::<_, String>(
            "SELECT token FROM licenses WHERE license = $1 AND active = 1 AND deleted = 0",
        )
        .bind(license)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?;
        Ok(result)
    }

    async fn list(&self, cursor: &i64, page_size: &i64) -> AppResult<Vec<License>> {
        let rows = sqlx::query_as::<_, LicenseRow>(
            r#"
            SELECT
                l.id, l.license, l.token, l.package_id,
                p.display_name as package_display_name, p.uid as package_uid,
                l.source, l.active, l.deleted, l.use_count, l.created_at
            FROM licenses l
            JOIN packages p ON l.package_id = p.id
            WHERE l.id < $1 AND l.deleted = 0
            ORDER BY l.id DESC
            LIMIT $2
            "#,
        )
        .bind(cursor)
        .bind(page_size)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows.into_iter().map(License::from).collect())
    }

    async fn register(
        &self,
        license: &str,
        token: &str,
        uid: &str,
        source: &str,
        created_at: &DateTime<Utc>,
    ) -> AppResult<()> {
        let created_at_str = created_at.to_rfc3339();
        sqlx::query(
            r#"
            INSERT INTO licenses (license, token, package_id, source, use_count, created_at)
            SELECT $1, $2, id, $3, 1, $4 FROM packages WHERE uid = $5
            "#,
        )
        .bind(license)
        .bind(token)
        .bind(source)
        .bind(&created_at_str)
        .bind(uid)
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }

    async fn update(&self, license: &str, active: bool) -> AppResult<()> {
        sqlx::query("UPDATE licenses SET active = $2 WHERE license = $1")
            .bind(license)
            .bind(active)
            .execute(&self.pool)
            .await
            .map_err(AppError::from)?;
        Ok(())
    }

    async fn increment_use_count(&self, token: &str) -> AppResult<()> {
        sqlx::query("UPDATE licenses SET use_count = use_count + 1 WHERE token = $1 AND active = 1 AND deleted = 0")
            .bind(token)
            .execute(&self.pool)
            .await
            .map_err(AppError::from)?;
        Ok(())
    }

    async fn delete(&self, license: &str) -> AppResult<()> {
        sqlx::query("UPDATE licenses SET deleted = 1 WHERE license = $1")
            .bind(license)
            .execute(&self.pool)
            .await
            .map_err(AppError::from)?;
        Ok(())
    }

    async fn get_package_uid_by_market_product(
        &self,
        market: &str,
        product_id: &str,
    ) -> AppResult<Option<String>> {
        sqlx::query_scalar::<_, String>(
            r#"
            SELECT p.uid FROM packages p
            JOIN package_markets pm ON pm.package_id = p.id
            WHERE pm.market = $1 AND pm.product_id = $2
            "#,
        )
        .bind(market)
        .bind(product_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)
    }
}
