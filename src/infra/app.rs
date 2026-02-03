use crate::{adapters::http::app_state::AppState, infra::setup::init_tracing};
use axum::{
    http::{self, header, Method},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

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
        .merge(crate::adapters::http::routes::router(app_state))
        .with_state(app_state)
        .layer(cors)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &http::Request<_>| {
                let request_id = Uuid::new_v4();
                tracing::info_span!(
                    "http-request",
                    method = %request.method(),
                    uri = %request.uri(),
                    version = ?request.version(),
                    request_id = %request_id
                )
            }),
        )
}
