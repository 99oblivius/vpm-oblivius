use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing,
    Json,
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    adapters::http::app_state::AppState,
    app_error::{AppError, AppResult},
    domain::{Package, PackageVersion},
    use_cases::packages::PackageUseCases,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/packages", routing::post(package_create))
        .route("/packages/{uid}", routing::get(package_get).patch(package_rename).delete(package_delete))
        .route("/packages/{uid}/versions", routing::post(version_add))
        .route("/packages/{uid}/versions/{version}", routing::patch(version_update).delete(version_delete))
        .route("/packages/{uid}/markets", routing::post(market_link))
}

#[derive(Debug, Clone, Deserialize)]
struct CreatePayload {
    name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RenamePayload {
    name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AddVersionPayload {
    version: String,
    file_name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct UpdateVersionPayload {
    file_name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct LinkMarketPayload {
    market: String,
    product_id: String,
}


#[derive(Debug, Clone, Serialize)]
struct PackageDetailResponse {
    package: Package,
    versions: Vec<PackageVersion>,
}


async fn package_create(
    State(use_cases): State<Arc<PackageUseCases>>,
    Json(payload): Json<CreatePayload>,
) -> AppResult<impl IntoResponse> {
    info!("Package creation called: {}", payload.name);
    let package = use_cases.create(&payload.name).await?;
    Ok((StatusCode::CREATED, Json(package)))
}

async fn package_rename(
    State(use_cases): State<Arc<PackageUseCases>>,
    Path(uid): Path<String>,
    Json(payload): Json<RenamePayload>,
) -> AppResult<impl IntoResponse> {
    info!("Package rename called: {} -> {}", uid, payload.name);
    use_cases.rename(&uid, &payload.name).await?;
    Ok(StatusCode::OK)
}

async fn package_delete(
    State(use_cases): State<Arc<PackageUseCases>>,
    Path(uid): Path<String>,
) -> AppResult<impl IntoResponse> {
    info!("Package deletion called: {}", uid);
    use_cases.delete(&uid).await?;
    Ok(StatusCode::OK)
}

async fn version_add(
    State(use_cases): State<Arc<PackageUseCases>>,
    Path(uid): Path<String>,
    Json(payload): Json<AddVersionPayload>,
) -> AppResult<impl IntoResponse> {
    info!("Version add called: {} v{}", uid, payload.version);
    use_cases
        .add_version(&uid, &payload.version, &payload.file_name)
        .await?;
    Ok(StatusCode::CREATED)
}

async fn version_update(
    State(use_cases): State<Arc<PackageUseCases>>,
    Path((uid, version)): Path<(String, String)>,
    Json(payload): Json<UpdateVersionPayload>,
) -> AppResult<impl IntoResponse> {
    info!("Version update called: {} v{}", uid, version);
    use_cases
        .update_version(&uid, &version, &payload.file_name)
        .await?;
    Ok(StatusCode::OK)
}

async fn version_delete(
    State(use_cases): State<Arc<PackageUseCases>>,
    Path((uid, version)): Path<(String, String)>,
) -> AppResult<impl IntoResponse> {
    info!("Version deletion called: {} v{}", uid, version);
    use_cases.delete_version(&uid, &version).await?;
    Ok(StatusCode::OK)
}

async fn market_link(
    State(use_cases): State<Arc<PackageUseCases>>,
    Path(uid): Path<String>,
    Json(payload): Json<LinkMarketPayload>,
) -> AppResult<impl IntoResponse> {
    info!("Market link called: {} -> {}:{}", uid, payload.market, payload.product_id);
    use_cases
        .link_market(&uid, &payload.market, &payload.product_id)
        .await?;
    Ok(StatusCode::CREATED)
}

async fn package_get(
    State(use_cases): State<Arc<PackageUseCases>>,
    Path(uid): Path<String>,
) -> AppResult<impl IntoResponse> {
    info!("Package detail called: {}", uid);
    let package = use_cases
        .get_by_uid(&uid)
        .await?
        .ok_or(AppError::Internal(format!("Package not found: {}", uid)))?;
    let versions = use_cases.get_versions(&uid).await?;
    Ok((
        StatusCode::OK,
        Json(PackageDetailResponse { package, versions }),
    ))
}
