use tracing::info;

use vpm::infra::{
    app::create_app,
    setup::init_state,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = init_state().await?;

    let app = create_app(state);

    let serve_addr = state.config.serve_addr;
    let listener = tokio::net::TcpListener::bind(serve_addr).await?;
    info!("Server started listening on {serve_addr}!");

    axum::serve(listener, app).await?;

    Ok(())
}
