mod code;
mod license;

pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(code::router())
        .merge(license::router())
        .layer(middleware::from_fn_with_state(state, validate_login))
}
