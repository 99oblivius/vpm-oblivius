use std::sync::{Arc, atomic::{AtomicU64, Ordering}};

use axum::extract::FromRef;

use crate::{
    domain::MarketCredentialStore,
    infra::config::AppConfig,
    use_cases::{gift::GiftUseCases, license::LicenseUseCases, packages::PackageUseCases},
};

/// Tracks the minimum `iat` for valid tokens. Bumped on logout to revoke all prior tokens.
#[derive(Clone)]
pub struct AuthState(Arc<AtomicU64>);

impl AuthState {
    pub fn new() -> Self {
        Self(Arc::new(AtomicU64::new(0)))
    }

    pub fn issued_after(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }

    pub fn revoke_all(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.0.store(now, Ordering::Relaxed);
    }
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub auth: AuthState,
    pub credential_store: Arc<MarketCredentialStore>,
    pub gift_use_cases: Arc<GiftUseCases>,
    pub license_use_cases: Arc<LicenseUseCases>,
    pub package_use_cases: Arc<PackageUseCases>,
}

impl FromRef<AppState> for Arc<AppConfig> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.config.clone()
    }
}

impl FromRef<AppState> for AuthState {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.auth.clone()
    }
}

impl FromRef<AppState> for Arc<GiftUseCases> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.gift_use_cases.clone()
    }
}

impl FromRef<AppState> for Arc<LicenseUseCases> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.license_use_cases.clone()
    }
}

impl FromRef<AppState> for Arc<PackageUseCases> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.package_use_cases.clone()
    }
}

impl FromRef<AppState> for Arc<MarketCredentialStore> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.credential_store.clone()
    }
}
