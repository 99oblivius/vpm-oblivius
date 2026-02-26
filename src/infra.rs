use crate::{
    adapters::queries::SqliteDatabase,
    infra::db::init_db,
};

pub mod app;
pub mod config;
pub mod db;
pub mod setup;

pub async fn sqlite_database(database_url: &str) -> anyhow::Result<SqliteDatabase> {
    let pool = init_db(database_url).await?;
    let database = SqliteDatabase::new(pool);
    Ok(database)
}
