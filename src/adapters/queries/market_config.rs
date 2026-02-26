use async_trait::async_trait;

use crate::{
    adapters::queries::SqliteDatabase,
    app_error::{AppError, AppResult},
    domain::{MarketConfigRepository, MarketCredentials},
};

#[async_trait]
impl MarketConfigRepository for SqliteDatabase {
    async fn list_all(&self) -> AppResult<Vec<MarketCredentials>> {
        let rows = sqlx::query_as::<_, MarketCredentialsRow>(
            "SELECT market, base_url, api_key, active, updated_at FROM market_configs",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(rows.into_iter().map(MarketCredentials::from).collect())
    }

    async fn get(&self, market: &str) -> AppResult<Option<MarketCredentials>> {
        let row = sqlx::query_as::<_, MarketCredentialsRow>(
            "SELECT market, base_url, api_key, active, updated_at FROM market_configs WHERE market = $1",
        )
        .bind(market)
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::from)?;

        Ok(row.map(MarketCredentials::from))
    }

    async fn upsert(&self, creds: &MarketCredentials) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO market_configs (market, base_url, api_key, active, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT(market) DO UPDATE SET
                base_url = excluded.base_url,
                api_key = excluded.api_key,
                active = excluded.active,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&creds.market)
        .bind(&creds.base_url)
        .bind(&creds.api_key)
        .bind(creds.active)
        .bind(&creds.updated_at)
        .execute(&self.pool)
        .await
        .map_err(AppError::from)?;
        Ok(())
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct MarketCredentialsRow {
    market: String,
    base_url: String,
    api_key: String,
    active: bool,
    updated_at: String,
}

impl From<MarketCredentialsRow> for MarketCredentials {
    fn from(row: MarketCredentialsRow) -> Self {
        Self {
            market: row.market,
            base_url: row.base_url,
            api_key: row.api_key,
            active: row.active,
            updated_at: row.updated_at,
        }
    }
}
