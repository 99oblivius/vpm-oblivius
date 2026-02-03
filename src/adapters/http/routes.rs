mod api;
mod panel;
mod public;

pub fn router(state: AppState) -> Router {
    Router::new()
        .merge(public::router())
        .nest("/panel", panel::router())
        .nest("/api", api::router(state))
}
