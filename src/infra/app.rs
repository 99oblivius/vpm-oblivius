use axum::Router;
use http;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

use crate::{
    adapters::http::app_state::AppState,
    infra::setup::init_tracing,
};

pub fn create_app(app_state: AppState) -> Router {
    init_tracing();

    let cors = CorsLayer::new()
        .allow_origin(
            "http://10.17.0.106"
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .allow_methods([http::Method::POST, http::Method::GET, http::Method::PATCH, http::Method::DELETE])
        .allow_headers([])
        .allow_credentials(false);

    Router::new()
        .merge(crate::adapters::http::routes::router())
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
