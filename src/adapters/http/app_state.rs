use std::sync::Arc;

use axum::extract::FromRef;

use crate::{infra::config::AppConfig, use_cases::code::CodeUseCases};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub code_use_cases: Arc<CodeUseCases>,
}

impl FromRef<AppState> for Arc<CodeUseCases> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.code_use_cases.clone()
    }
}
