use crate::{
    adapters::queries::SqliteDatabase,
    infra::db::init_db,
};

pub mod app;
pub mod config;
pub mod db;
pub mod setup;

pub async fn sqlite_database() -> anyhow::Result<SqliteDatabase> {
    let pool = init_db().await?;
    let database = SqliteDatabase::new(pool);
    Ok(database)
}
