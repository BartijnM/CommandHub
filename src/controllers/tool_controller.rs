use std::sync::Arc;

use axum::{Json, extract::State};
use serde_json::{Value, json};

use crate::registry::{ToolRegistry, input_schema};

pub async fn list_tools(State(registry): State<Arc<ToolRegistry>>) -> Json<Value> {
    let tools = registry
        .tools()
        .iter()
        .map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": input_schema(tool)
            })
        })
        .collect::<Vec<_>>();

    Json(json!({ "tools": tools }))
}
