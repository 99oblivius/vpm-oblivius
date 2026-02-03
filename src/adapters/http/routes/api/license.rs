use axum::{
    http::{get, patch},
    Router,
};

use crate::adapters::http::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/license", get(license_get).patch(license_update))
}
