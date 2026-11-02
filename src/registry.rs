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
}
