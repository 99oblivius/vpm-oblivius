pub mod app_error_impl;
pub mod app_state;
pub mod bounded;
pub mod rate_limit;
pub mod routes;

use http::{HeaderMap, header};

/// Derive the public-facing base URL from the request's Host header,
/// validated against the configured CORS origins.
pub fn base_url_from_headers(headers: &HeaderMap, cors_origins: &[String]) -> String {
    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost");
    let scheme = if host.starts_with("localhost") || host.starts_with("127.0.0.1") {
        "http"
    } else {
        "https"
    };
    let url = format!("{scheme}://{host}");
    if cors_origins.iter().any(|o| o == &url) {
        url
    } else {
        cors_origins.first().cloned().unwrap_or(url)
    }
}
