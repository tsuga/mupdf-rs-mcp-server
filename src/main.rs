//! MuPDF MCP Server entry point.
//!
//! This binary starts the MCP server using STDIO transport.

use mupdf_rs_mcp_server::MupdfServer;
use rmcp::ServiceExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging to stderr (important for STDIO transport)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mupdf_rs_mcp_server=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting MuPDF MCP Server v{}", env!("CARGO_PKG_VERSION"));

    // Create the server
    let server = MupdfServer::new();

    // Serve over STDIO
    let service = server.serve(rmcp::transport::stdio()).await?;

    // Wait for the service to complete
    service.waiting().await?;

    tracing::info!("MuPDF MCP Server stopped");
    Ok(())
}
