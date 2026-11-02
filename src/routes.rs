use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};

use crate::controllers::{
    execution_controller::execute_tool, status_controller::status, tool_controller::list_tools,
};

// Bouw de HTTP-routes met gedeelde applicatiestate.
pub fn router(state: crate::app_state::AppState) -> Router {
    Router::new()
        .route("/status", get(status))
        .route("/tools", get(list_tools))
        .route(
            "/tools/{toolname}",
            post(execute_tool).layer(DefaultBodyLimit::max(16 * 1024)),
        )
        .with_state(state)
}

#[cfg(test)]
#[path = "tests/routes_tests.rs"]
mod tests;
