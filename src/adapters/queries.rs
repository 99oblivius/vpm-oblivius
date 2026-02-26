use sqlx::SqlitePool;
use crate::app_error::AppError;

pub mod gift;
pub mod license;
pub mod market_config;
pub mod packages;

pub struct SqliteDatabase {
    pool: SqlitePool,
}

impl SqliteDatabase {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        AppError::Database(value.to_string())
    }
}
