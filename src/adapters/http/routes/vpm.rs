use std::{collections::HashMap, sync::Arc};

use axum::{Router, extract::{Query, State}, http::StatusCode, response::IntoResponse, routing, Json};
use ::http::HeaderMap;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize)]
struct VpmListing {
    name: String,
    author: String,
    url: String,
    packages: HashMap<String, VpmPackageEntry>,
}

#[derive(Serialize)]
struct VpmPackageEntry {
    versions: HashMap<String, VpmVersionManifest>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct VpmVersionManifest {
    name: String,
    display_name: String,
    version: String,
    url: String,
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

    let mut version_map = HashMap::new();
    for v in &versions {
        let download_url = format!(
            "{base_url}/packages/{}/{}?token={}",
            package.uid, v.file_name, query.token
        );
        version_map.insert(
            v.version.clone(),
            VpmVersionManifest {
                name: package.uid.clone(),
                display_name: package.name.clone(),
                version: v.version.clone(),
                url: download_url,
            },
        );
    }

    let mut packages = HashMap::new();
    packages.insert(
        package.uid.clone(),
        VpmPackageEntry {
            versions: version_map,
        },
    );

    let listing = VpmListing {
        name: "Oblivius Packages".to_string(),
        author: "Oblivius".to_string(),
        url: listing_url,
        packages,
    };

    Json(listing).into_response()
}
