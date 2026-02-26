use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing,
    Json,
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use tracing::info;

use crate::{
    adapters::http::{app_state::AppState, bounded::Bounded},
    app_error::AppResult,
    use_cases::gift::GiftUseCases,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/gift", routing::post(gift_create))
}

#[derive(Debug, Clone, Deserialize)]
struct CreatePayload {
    package_id: Bounded<128>,
}

#[derive(Debug, Clone, Serialize)]
struct CreateResponse {
    code: String,
    token: String,
    created_at: DateTime<Utc>,
}

async fn gift_create(
    State(gift_use_cases): State<Arc<GiftUseCases>>,
    Json(payload): Json<CreatePayload>,
) -> AppResult<impl IntoResponse> {
    info!("Gift creation called");

    let (code, token, created_at) = gift_use_cases.create(&payload.package_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateResponse { code, token, created_at }),
    ))
}
