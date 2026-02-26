use std::sync::Arc;
use axum::{
    extract::{State, Query},
    http::StatusCode,
    response::IntoResponse,
    routing,
    Json,
    Router,
};
use serde::Deserialize;
use tracing::info;

use crate::{
    adapters::http::{app_state::AppState, bounded::Bounded},
    use_cases::license::LicenseUseCases,
    app_error::AppResult,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/license", routing::get(license_list).patch(license_update).delete(license_delete))
}

#[derive(Debug, Clone, Deserialize)]
struct ListPayload {
    cursor: i64,
    page_size: i64,
}

#[derive(Debug, Clone, Deserialize)]
enum UpdateAction {
    ENABLE,
    DISABLE,
}

#[derive(Debug, Clone, Deserialize)]
struct UpdatePayload {
    code: Bounded<512>,
    update: UpdateAction,
}

#[derive(Debug, Clone, Deserialize)]
struct DeletePayload {
    code: Bounded<512>,
}

async fn license_list(
    State(license_use_cases): State<Arc<LicenseUseCases>>,
    Query(payload): Query<ListPayload>,
) -> AppResult<impl IntoResponse> {
    let page_size = payload.page_size.clamp(1, 1000);
    info!("License list called: {} {}", &payload.cursor, &page_size);
    let licenses = license_use_cases
        .list(&payload.cursor, &page_size)
        .await?;
    Ok((StatusCode::OK, Json(licenses)))
}

async fn license_update(
    State(license_use_cases): State<Arc<LicenseUseCases>>,
    Json(payload): Json<UpdatePayload>,
) -> AppResult<impl IntoResponse> {
    info!("License update called: {}", payload.code);
    match payload.update {
        UpdateAction::ENABLE => license_use_cases.enable(&payload.code).await?,
        UpdateAction::DISABLE => license_use_cases.disable(&payload.code).await?,
    }
    Ok(StatusCode::OK)
}

async fn license_delete(
    State(license_use_cases): State<Arc<LicenseUseCases>>,
    Json(payload): Json<DeletePayload>,
) -> AppResult<impl IntoResponse> {
    info!("License deletion called: {}", payload.code);
    license_use_cases.delete(&payload.code).await?;
    Ok(StatusCode::OK)
}
