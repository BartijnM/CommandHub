use std::sync::Arc;

use axum::{Json, extract::State};
use serde_json::{Map, Value, json};

use crate::{config::ParameterType, registry::ToolRegistry};

// Handler voor de /tools route die de beschikbare tools retourneert
pub async fn list_tools(State(registry): State<Arc<ToolRegistry>>) -> Json<Value> {
    // Maak een JSON-array van de beschikbare tools met hun metadata en input schema
    let tools = registry
        .tools()
        .iter()
        .map(|tool| {
            // Maak een JSON-object voor de input schema van de tool
            let properties = tool
                .parameters
                .iter()
                .map(|parameter| {
                    let parameter_type = match &parameter.kind {
                        ParameterType::String => "string",
                    };

                    (parameter.name.clone(), json!({ "type": parameter_type }))
                })
                .collect::<Map<String, Value>>();

            // Maak een JSON-array van de verplichte parameters van de tool
            let required = tool
                .parameters
                .iter()
                .filter(|parameter| parameter.required)
                .map(|parameter| parameter.name.clone())
                .collect::<Vec<_>>();

            // Maak een JSON-object voor de tool met zijn metadata en input schema
            json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": {
                    "type": "object",
                    "properties": properties,
                    "required": required
                }
            })
        })
        .collect::<Vec<_>>();

    // Retourneer een JSON-object met de beschikbare tools
    Json(json!({ "tools": tools }))
}
