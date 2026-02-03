use crate::{
    adapters::{
        http::app_state::AppState,
        markets::Markets,
    },
    infra::{config::AppConfig, sqlite_database},
    use_cases::{
        redeem::RedeemUseCases,
        code::CodeUseCases,
    },
};
use tracing_subscriber::{EnvFilter, fmt};

use std::sync::Arc;

pub async fn init_state() -> anyhow::Result<AppState> {
    let config = AppConfig::from_env();

    let sqlite_arc = Arc::new(sqlite_database().await?);


    let markets = Markets::new();

    let code_use_cases = CodeUseCases::new(sqlite_arc.clone());
    let redeem_use_cases = RedeemUseCases::new(Arc::new(markets), sqlite_arc.clone());

    Ok(AppState {
        config: Arc::new(config),
        code_use_cases: Arc::new(code_use_cases),
    })
}

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "axum_trainer=debug,tower_http=debug".into());

    let console_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .pretty();

    tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .try_init()
        .ok();
}
