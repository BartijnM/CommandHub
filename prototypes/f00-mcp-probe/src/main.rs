use std::{error::Error, sync::Arc};

use axum::Router;
use rmcp::{
    tool, tool_router,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, session::local::LocalSessionManager,
        tower::StreamableHttpService,
    },
};

#[derive(Clone)]
struct ProbeServer;

#[tool_router(server_handler)]
impl ProbeServer {
    #[tool(description = "Geeft een vaste tekst terug als MCP-test.")]
    async fn hello(&self) -> String {
        "De MCP-testtool werkt.".to_owned()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mcp_service = StreamableHttpService::new(
        || Ok(ProbeServer),
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default(),
    );

    let app = Router::new().route_service("/mcp", mcp_service);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await?;

    println!("MCP-proefserver luistert op http://127.0.0.1:3001/mcp");

    axum::serve(listener, app).await?;

    Ok(())
}
