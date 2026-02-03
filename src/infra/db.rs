use std::env;
use tracing::info;
use chrono::Utc;
use sqlx::{SqlitePool, FromRow, sqlite::SqlitePoolOptions};

#[derive(Clone, FromRow, Debug)]
struct Code {
    id: i64,
    created_at: Utc
}

pub async fn init_db() -> anyhow::Result<SqlitePool> {
    let url = env::var("DATABASE_URL").unwrap_or("./data/keys.db".into());
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await?;

    info!("Conected to database {0}!", url);
    pool
}
