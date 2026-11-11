use std::{future::Future, io};

use tokio::net::TcpListener;

use crate::{app_state::AppState, routes};

pub async fn serve(
    listener: TcpListener,
    state: AppState,
    shutdown_signal: impl Future<Output = ()> + Send + 'static,
) -> io::Result<()> {
    let execution = state.execution.clone();
    let mcp_shutdown = tokio_util::sync::CancellationToken::new();

    let app = routes::router_with_shutdown(state.clone(), mcp_shutdown.clone());

    let shutdown_token = mcp_shutdown.clone();

    let result = axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_signal.await;

            execution.begin_shutdown();
            shutdown_token.cancel();
        })
        .await;

    mcp_shutdown.cancel();
    state.execution.shutdown().await;

    result
}
