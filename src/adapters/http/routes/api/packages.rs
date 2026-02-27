use std::sync::Arc;

use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing,
    Json,
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    adapters::http::{app_state::AppState, bounded::Bounded},
    app_error::{AppError, AppResult},
    domain::{Package, PackageVersion},
    use_cases::packages::{ChunkReader, PackageUseCases},
};

/// Adapter that wraps an axum multipart Field as a ChunkReader.
struct FieldChunkReader<'a>(axum::extract::multipart::Field<'a>);

#[async_trait::async_trait]
impl ChunkReader for FieldChunkReader<'_> {
    async fn next_chunk(&mut self) -> AppResult<Option<bytes::Bytes>> {
        self.0
            .chunk()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to read upload chunk: {e}")))
    }
}

const MAX_UPLOAD_SIZE: usize = 5 * 1024 * 1024 * 1024; // 5 GB

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/packages/upload",
            routing::post(package_upload).layer(DefaultBodyLimit::max(MAX_UPLOAD_SIZE)),
        )
        .route("/packages/{uid}", routing::get(package_get).delete(package_delete))
        .route("/packages/{uid}/versions/{version}", routing::delete(version_delete))
        .route("/packages/{uid}/markets", routing::post(market_link))
}

#[derive(Debug, Clone, Deserialize)]
struct LinkMarketPayload {
    market: Bounded<128>,
    product_id: Bounded<128>,
}

#[derive(Debug, Clone, Serialize)]
struct PackageDetailResponse {
    package: Package,
    versions: Vec<PackageVersion>,
}

/// Multipart upload: expects a single `file` field containing a .zip.
/// Version, UID, and display name are extracted from the zip's package.json.
/// Auto-creates the package on first upload.
async fn package_upload(
    State(use_cases): State<Arc<PackageUseCases>>,
    mut multipart: Multipart,
) -> AppResult<impl IntoResponse> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Internal(format!("Multipart error: {e}")))?
    {
        if field.name() == Some("file") {
            let file_name = field
                .file_name()
                .ok_or_else(|| AppError::Internal("file field must have a filename".into()))?
                .to_string();

            info!("Package upload: {}", file_name);

            let mut reader = FieldChunkReader(field);
            let result = use_cases.upload_version(&file_name, &mut reader).await?;
            return Ok((StatusCode::CREATED, Json(result)));
        }
    }

    Err(AppError::Internal("Missing file field in upload".into()))
}

async fn package_delete(
    State(use_cases): State<Arc<PackageUseCases>>,
    Path(uid): Path<String>,
) -> AppResult<impl IntoResponse> {
    info!("Package deletion called: {}", uid);
    use_cases.delete(&uid).await?;
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
        .ok_or(AppError::NotFound)?;
    let versions = use_cases.get_versions(&uid).await?;
    Ok((
        StatusCode::OK,
        Json(PackageDetailResponse { package, versions }),
    ))
}
