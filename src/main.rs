// Copyright 2025 oblivius
// SPDX-License-Identifier: AGPL-3.0-only

use tracing::info;
use vpm_oblivius::infra::{app::create_app, setup::init_state};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = init_state().await?;
    let serve_addr = state.config.serve_addr.clone();

    let app = create_app(state);

    let listener = tokio::net::TcpListener::bind(&serve_addr).await?;
    info!("Server started listening on {serve_addr}!");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shut down gracefully");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received SIGINT"),
        _ = terminate => info!("Received SIGTERM"),
    }
}
