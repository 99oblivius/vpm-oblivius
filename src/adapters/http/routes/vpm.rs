use std::sync::Arc;

use axum::{Router, extract::{Query, State}, http::StatusCode, response::IntoResponse, routing, Json};
use ::http::HeaderMap;
use serde::Deserialize;

use crate::{
    adapters::http::{self, app_state::AppState},
    infra::config::AppConfig,
    use_cases::packages::PackageUseCases,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/index.json", routing::get(vpm_index))
        .layer(crate::adapters::http::rate_limit::per_ip(20, 40))
}

#[derive(Debug, Deserialize)]
struct IndexQuery {
    token: String,
}

async fn vpm_index(
    Query(query): Query<IndexQuery>,
    headers: HeaderMap,
    State(config): State<Arc<AppConfig>>,
    State(package_use_cases): State<Arc<PackageUseCases>>,
) -> impl IntoResponse {
    let package = match package_use_cases.get_package_for_token(&query.token).await {
        Ok(Some(pkg)) => pkg,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let versions = package_use_cases
        .get_versions(&package.uid)
        .await
        .unwrap_or_default();

    let base_url = http::base_url_from_headers(&headers, &config.cors_origins);
    let listing_url = format!("{base_url}/index.json?token={}", query.token);

    let mut version_map = serde_json::Map::new();
    for v in &versions {
        let download_url = format!(
            "{base_url}/packages/{}/{}?token={}",
            package.uid, v.file_name, query.token
        );

        // Parse stored manifest_json, or build minimal fallback
        let mut manifest: serde_json::Value = serde_json::from_str(&v.manifest_json)
            .unwrap_or_else(|_| serde_json::json!({
                "name": package.uid,
                "displayName": package.display_name,
                "version": v.version,
            }));

        // Inject/override url and zipSHA256
        if let Some(obj) = manifest.as_object_mut() {
            obj.insert("url".to_string(), serde_json::Value::String(download_url));
            if !v.zip_sha256.is_empty() {
                obj.insert("zipSHA256".to_string(), serde_json::Value::String(v.zip_sha256.clone()));
            }
        }

        version_map.insert(v.version.clone(), manifest);
    }

    let listing = serde_json::json!({
        "name": "Oblivius Packages",
        "author": "Oblivius",
        "url": listing_url,
        "packages": {
            &package.uid: {
                "versions": version_map,
            }
        }
    });

    Json(listing).into_response()
}
