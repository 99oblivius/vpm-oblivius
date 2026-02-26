use tracing::info;
use vpm_oblivius::infra::{app::create_app, setup::init_state};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = init_state().await?;
    let serve_addr = state.config.serve_addr.clone();

    let app = create_app(state);

    let listener = tokio::net::TcpListener::bind(&serve_addr).await?;
    info!("Server started listening on {serve_addr}!");

    axum::serve(listener, app).await?;

    Ok(())
}
