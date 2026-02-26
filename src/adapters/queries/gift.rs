use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::{
    adapters::queries::SqliteDatabase,
    app_error::{AppError, AppResult},
    domain::GiftRepository,
};

#[async_trait]
impl GiftRepository for SqliteDatabase {
    async fn code_create(
        &self,
        code: &str,
        token: &str,
        uid: &str,
        created_at: &DateTime<Utc>,
    ) -> AppResult<()> {
        let created_at_str = created_at.to_rfc3339();
        sqlx::query(
            r#"
            INSERT INTO licenses (license, token, package_id, created_at, source)
            SELECT $1, $2, id, $3, $4 FROM packages WHERE uid = $5
            "#,
        )
        .bind(code)
        .bind(token)
        .bind(&created_at_str)
        .bind("gift")
        .bind(uid)
        .execute(&self.pool)
        .await
        .map_err(AppError::from)
        .and_then(|r| {
            if r.rows_affected() == 0 {
                Err(AppError::NotFound)
            } else {
                Ok(())
            }
        })?;
        Ok(())
    }
}
