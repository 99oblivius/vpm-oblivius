use std::sync::Arc;

use axum::{
    Router,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use axum_extra::extract::CookieJar;

use crate::adapters::{
    http::app_state::{AppState, AuthState},
    utils::jwt,
};
use crate::infra::config::AppConfig;

mod gift;
mod license;
mod markets;
mod packages;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .merge(gift::router())
        .merge(license::router())
        .merge(markets::router())
        .merge(packages::router())
        .layer(middleware::from_fn_with_state(state, authentication))
}

async fn authentication(
    State(config): State<Arc<AppConfig>>,
    State(auth): State<AuthState>,
    jar: CookieJar,
    request: Request,
    next: Next,
) -> Response {
    let token = extract_bearer(&request)
        .or_else(|| jar.get("admin_token").map(|c| c.value().to_string()));

    let Some(token) = token else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    if jwt::validate_access_token(&config.jwt_secret, &token, auth.issued_after()).is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    next.run(request).await
}

fn extract_bearer(request: &Request) -> Option<String> {
    request
        .headers()
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(|s| s.to_string())
}
