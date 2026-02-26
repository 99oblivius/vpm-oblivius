use axum::{
    http::{get, patch},
    Router,
};

use crate::adapters::http::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/license", routing::get(license_list).patch(license_update))
}

#[derive(Debug, Clone, Deserialize)]
struct ListPayload {
    cursor: i64,
    page_size: i64,
}

#[derive(Debug, Clone, Serialize)]
struct ListResponse {
    licenses: LicenseRow,
}

#[derive(Debug, Clone, Deserialize)]
enum UpdateAction {
    ENABLE,
    DISABLE,
}

#[derive(Debug, Clone, Deserialize)]
struct UpdatePayload {
    code: String,
    update: UpdateAction,
}

#[derive(Debug, Clone, Deserialize)]
struct DeletePayload {
    code: String,
}

async fn license_list(
    State(license_use_cases): State<Arc<LicenseUseCases>>,
    Query(payload): Quer<ListPayload>,
) -> AppResult<impl IntoResponse> {
    let licenses = license_use_cases
        .list(payload.cursor, payload.page_size)
        .await?;
    Ok((StatusCode::OK, Json(json!(licenses))))
}

async fn license_update(
    State(license_use_cases): State<Arc<LicenseUseCases>>,
    Json(payload): Json<UpdatePayload>,
) -> AppResult<impl IntoResponse> {
    info!("License update called");
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
    info!("License deletion called");
    license_use_cases.delete(&payload.code).await?;
    Ok(StatusCode::OK)
}
