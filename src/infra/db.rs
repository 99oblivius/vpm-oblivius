use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tracing::info;

pub async fn init_db(url: &str) -> anyhow::Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    info!("Connected to database {0}!", url);
    Ok(pool)
}
