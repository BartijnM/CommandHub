use std::{future::Future, io};

use tokio::net::TcpListener;

use crate::{app_state::AppState, routes};

// De signaalbron blijft buiten de server: main gebruikt Ctrl+C, tests een kanaal.
pub async fn serve(
    listener: TcpListener,
    state: AppState,
    shutdown_signal: impl Future<Output = ()> + Send + 'static,
) -> io::Result<()> {
    let execution = state.execution.clone();
    let result = axum::serve(listener, routes::router(state.clone()))
        .with_graceful_shutdown(async move {
            shutdown_signal.await;
            execution.begin_shutdown();
        })
        .await;

    state.execution.shutdown().await;
    result
}
