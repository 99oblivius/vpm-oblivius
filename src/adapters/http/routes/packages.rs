use std::{collections::HashMap, sync::Arc};
use axum::{
    Router,
    extract::{Query, Request, State},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    http::StatusCode,
};
use tower_http::services::ServeDir;

use crate::{
    adapters::http::app_state::AppState,
    domain::verify_token,
    use_cases::packages::PackageUseCases,
};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .nest_service("/packages", ServeDir::new("assets/packages"))
        .layer(middleware::from_fn_with_state(state, verify_vpm_token))
}

async fn verify_vpm_token(
    State(package_use_cases): State<Arc<PackageUseCases>>,
    Query(params): Query<HashMap<String, String>>,
    request: Request,
    next: Next,
) -> Response {
    let Some(token) = params.get("token") else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    if !verify_token(token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let path = request.uri().path();
    let Some(uid) = path.strip_prefix("/packages/").and_then(|p| p.split('/').next()) else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    if package_use_cases.verify_access(uid, token).await.is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    next.run(request).await
}
