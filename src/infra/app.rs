use axum::{
    extract::DefaultBodyLimit,
    http::{self, header, Method, Uri},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

use crate::{adapters::http::app_state::AppState, infra::setup::init_tracing};

/// Redact query parameters whose name contains "token" or "secret" (case-insensitive).
fn sanitize_uri(uri: &Uri) -> String {
    let Some(query) = uri.query() else {
        return uri.to_string();
    };

    let sanitized: Vec<String> = query
        .split('&')
        .map(|pair| {
            let key = pair.split('=').next().unwrap_or("");
            let lower = key.to_ascii_lowercase();
            if lower.contains("token") || lower.contains("secret") {
                format!("{key}=[REDACTED]")
            } else {
                pair.to_string()
            }
        })
        .collect();

    format!("{}?{}", uri.path(), sanitized.join("&"))
}

pub fn create_app(app_state: AppState) -> Router {
    init_tracing();

    let origins: Vec<http::HeaderValue> = app_state
        .config
        .cors_origins
        .iter()
        .map(|o| o.parse().expect("Invalid CORS origin"))
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::POST, Method::GET, Method::PATCH, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE])
        .allow_credentials(false);

    Router::new()
        .merge(crate::adapters::http::routes::router(app_state.clone()))
        .with_state(app_state)
        .layer(cors)
        .layer(DefaultBodyLimit::max(1024 * 64)) // 64 KB max request body
        .layer(crate::adapters::http::rate_limit::per_ip(100, 200))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &http::Request<_>| {
                let request_id = Uuid::new_v4();
                let safe_uri = sanitize_uri(request.uri());
                tracing::info_span!(
                    "http-request",
                    method = %request.method(),
                    uri = %safe_uri,
                    version = ?request.version(),
                    request_id = %request_id
                )
            }),
        )
}
