use crate::config::{ConfigFile, ToolDefinition};

// Een register van tools die beschikbaar zijn in de configuratie
pub struct ToolRegistry {
    tools: Vec<ToolDefinition>,
}

// Implementatie van ToolRegistry
impl ToolRegistry {
    // Maak een nieuwe ToolRegistry van een ConfigFile
    pub fn new(config: ConfigFile) -> Self {
        Self {
            tools: config.tools,
        }
    }

    // Verkrijg een slice van de beschikbare tools
    pub fn tools(&self) -> &[ToolDefinition] {
        &self.tools
    }

    pub fn find(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.iter().find(|tool| tool.name == name)
    }
}

pub fn input_schema(tool: &ToolDefinition) -> serde_json::Value {
    use crate::config::ParameterType;
    use serde_json::{Map, Value, json};

    let properties = tool
        .parameters
        .iter()
        .map(|parameter| {
            let parameter_type = match &parameter.kind {
                ParameterType::String => "string",
            };

            let mut schema = json!({ "type": parameter_type });

            if let Some(allowed_values) = &parameter.allowed_values {
                schema["enum"] = json!(allowed_values);
            }

            (parameter.name.clone(), schema)
        })
        .collect::<Map<String, Value>>();

    let required = tool
        .parameters
        .iter()
        .filter(|parameter| parameter.required)
        .map(|parameter| parameter.name.clone())
        .collect::<Vec<_>>();

    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": properties,
        "required": required
    })
}
