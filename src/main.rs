use std::sync::Arc;

use anyhow::Result;
use gareji_mcp::{FixtureBackend, GarejiMcpServer};
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let server = GarejiMcpServer::new(Arc::new(FixtureBackend::default()));
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
