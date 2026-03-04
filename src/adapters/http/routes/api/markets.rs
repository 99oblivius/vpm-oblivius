use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing,
    Json,
    Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    adapters::http::{app_state::AppState, bounded::Bounded},
    app_error::{AppError, AppResult},
    domain::{MarketCredentialStore, MarketCredentials},
    use_cases::license::LicenseUseCases,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/markets", routing::get(markets_list))
        .route("/markets/{name}", routing::patch(market_update).delete(market_delete))
}

#[derive(Debug, Serialize)]
struct MarketInfo {
    market: String,
    active: bool,
    updated_at: String,
}

async fn markets_list(
    State(store): State<Arc<MarketCredentialStore>>,
) -> impl IntoResponse {
    let markets: Vec<MarketInfo> = store
        .list()
        .into_iter()
        .map(|c| MarketInfo {
            market: c.market,
            active: c.active,
            updated_at: c.updated_at,
        })
        .collect();

    Json(markets)
}

#[derive(Debug, Deserialize)]
struct UpdatePayload {
    api_key: Option<Bounded<2048>>,
    active: Option<bool>,
}

async fn market_delete(
    State(store): State<Arc<MarketCredentialStore>>,
    Path(name): Path<String>,
) -> AppResult<impl IntoResponse> {
    info!("Market delete: {}", name);
    store.delete(&name).await?;
    Ok(StatusCode::OK)
}

async fn market_update(
    State(store): State<Arc<MarketCredentialStore>>,
    State(license_use_cases): State<Arc<LicenseUseCases>>,
    Path(name): Path<String>,
    Json(payload): Json<UpdatePayload>,
) -> AppResult<impl IntoResponse> {
    if !license_use_cases.available_market_names().contains(&name.as_str()) {
        return Err(AppError::BadRequest(format!("Unknown market: {name}")));
    }
    info!("Market config update: {}", name);

    let now = Utc::now().to_rfc3339();

    let mut creds = store.get(&name).unwrap_or(MarketCredentials {
        market: name,
        api_key: String::new(),
        active: false,
        updated_at: String::new(),
    });

    if let Some(key) = payload.api_key {
        creds.api_key = key.into();
    }
    if let Some(active) = payload.active {
        creds.active = active;
    }

    creds.updated_at = now;
    store.update(creds).await?;

    Ok(StatusCode::OK)
}
