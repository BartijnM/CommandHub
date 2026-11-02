use std::sync::Arc;

use crate::{execution::ExecutionService, registry::ToolRegistry};

#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<ToolRegistry>,
    pub execution: Arc<ExecutionService>,
}

impl axum::extract::FromRef<AppState> for Arc<ToolRegistry> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.registry)
    }
}
