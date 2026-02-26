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
        .route("/packages", routing::post(package_create))
        .route("/packages/{uid}", routing::get(package_get).patch(package_rename).delete(package_delete))
        .route(
            "/packages/{uid}/versions",
            routing::post(version_upload).layer(DefaultBodyLimit::max(MAX_UPLOAD_SIZE)),
        )
        .route("/packages/{uid}/versions/{version}", routing::delete(version_delete))
        .route("/packages/{uid}/markets", routing::post(market_link))
}

#[derive(Debug, Clone, Deserialize)]
struct CreatePayload {
    name: Bounded<128>,
}

#[derive(Debug, Clone, Deserialize)]
struct RenamePayload {
    name: Bounded<128>,
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

/// Multipart upload: expects fields `version` (text) and `file` (binary).
/// Overwrites existing version if it already exists.
async fn version_upload(
    State(use_cases): State<Arc<PackageUseCases>>,
    Path(uid): Path<String>,
    mut multipart: Multipart,
) -> AppResult<impl IntoResponse> {
    let mut version: Option<String> = None;
    let mut uploaded = false;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Internal(format!("Multipart error: {e}")))?
    {
        match field.name() {
            Some("version") => {
                let v = field
                    .text()
                    .await
                    .map_err(|e| AppError::Internal(format!("Failed to read version field: {e}")))?;
                if v.is_empty() || v.len() > 64 {
                    return Err(AppError::Internal("version must be 1-64 characters".into()));
                }
                version = Some(v);
            }
            Some("file") => {
                let ver = version.as_deref()
                    .ok_or_else(|| AppError::Internal("version field must appear before file".into()))?;

                let file_name = field
                    .file_name()
                    .ok_or_else(|| AppError::Internal("file field must have a filename".into()))?
                    .to_string();

                info!("Version upload: {} v{} ({})", uid, ver, file_name);

                let mut reader = FieldChunkReader(field);
                use_cases.upload_version(&uid, ver, &file_name, &mut reader).await?;
                uploaded = true;
            }
            _ => {} // skip unknown fields
        }
    }

    if !uploaded {
        return Err(AppError::Internal("Missing file field in upload".into()));
    }

    Ok(StatusCode::CREATED)
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
