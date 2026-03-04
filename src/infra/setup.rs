use std::sync::Arc;

use tracing_subscriber::{filter::EnvFilter, fmt, prelude::*};

use crate::{
    adapters::{
        http::app_state::{AppState, AuthState},
        markets::{Gumroad, Jinxxy, Payhip},
    },
    domain::MarketCredentialStore,
    infra::{config::AppConfig, sqlite_database},
    use_cases::{
        gift::GiftUseCases,
        license::{LicenseUseCases, Markets},
        packages::PackageUseCases,
    },
};

pub async fn init_state() -> anyhow::Result<AppState> {
    let config = AppConfig::from_env();

    let sqlite_arc = Arc::new(sqlite_database(&config.database_url).await?);

    let credential_store = Arc::new(
        MarketCredentialStore::load(sqlite_arc.clone()).await?,
    );

    let mut markets = Markets::new();
    if let Some(url) = &config.payhip_base_url {
        markets = markets.add(Box::new(Payhip::new(credential_store.clone(), url.clone())));
    }
    if let Some(url) = &config.jinxxy_base_url {
        markets = markets.add(Box::new(Jinxxy::new(credential_store.clone(), url.clone())));
    }
    if let Some(url) = &config.gumroad_base_url {
        markets = markets.add(Box::new(Gumroad::new(credential_store.clone(), url.clone())));
    }

    let gift_use_cases = GiftUseCases::new(sqlite_arc.clone());
    let license_use_cases = LicenseUseCases::new(sqlite_arc.clone(), Arc::new(markets));
    let package_use_cases = PackageUseCases::new(sqlite_arc.clone(), config.packages_dir.clone().into());

    Ok(AppState {
        config: Arc::new(config),
        auth: AuthState::new(),
        credential_store,
        gift_use_cases: Arc::new(gift_use_cases),
        license_use_cases: Arc::new(license_use_cases),
        package_use_cases: Arc::new(package_use_cases),
    })
}

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "vpm_oblivius=info,tower_http=info".into());

    let console_layer = fmt::layer().with_target(false).with_level(true).compact();

    tracing_subscriber::registry()
        .with(filter)
        .with(console_layer)
        .try_init()
        .ok();
}
