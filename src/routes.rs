use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};

use crate::controllers::{
    execution_controller::execute_tool, status_controller::status, tool_controller::list_tools,
};

pub fn router_with_shutdown(
    state: crate::app_state::AppState,
    shutdown: tokio_util::sync::CancellationToken,
) -> Router {
    Router::new()
        .route_service("/mcp", crate::mcp::service(state.clone(), shutdown))
        .route("/status", get(status))
        .route("/tools", get(list_tools))
        .route(
            "/tools/{toolname}",
            post(execute_tool).layer(DefaultBodyLimit::max(16 * 1024)),
        )
        .with_state(state)
}

#[cfg(test)]
pub(crate) use tests::router;

#[cfg(test)]
#[path = "tests/routes_tests.rs"]
mod tests;
