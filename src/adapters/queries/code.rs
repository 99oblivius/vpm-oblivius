use crate::app_error::{AppError, AppResult};

#[async_trait]
impl CodeStore for SqliteDatabase {
    async fn create(&self) -> AppResult<()> {



        sqlx::query!(
            "INSERT INTO codes (code, created_at) VALUES ($1, $2)",
            "...",
            utc::now()
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(())
    }
}
