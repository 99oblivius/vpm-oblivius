use axum::Router;

use crate::adapters::http::app_state::AppState;

mod public;
mod packages;
mod panel;
mod api;
mod vpm;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .merge(public::router())
        .merge(vpm::router())
        .merge(packages::router(state.clone()))
        .nest("/panel", panel::router(state.clone()))
        .nest("/api", api::router(state))
}
