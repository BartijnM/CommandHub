use serde::Deserialize;

use std::collections::HashSet;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFile {
    pub tools: Vec<ToolDefinition>,
}

impl ConfigFile {
    pub fn validate(&self) -> Result<(), String> {
        let mut names = HashSet::new();



        for tool in &self.tools {
            if tool.name.trim().is_empty() {
                return Err("Toolnaam mag niet leeg zijn".to_owned());
            }
            if tool.command.trim().is_empty() {
                return Err(format!(
                    "Command voor tool '{}' mag niet leeg zijn",
                    tool.name
                ));
            }
            if !names.insert(tool.name.as_str()) {
                return Err(format!("Dubbele toolnaam: {}", tool.name));
            }

            // Controleer of de timeout, indien aanwezig, binnen het bereik van 1 tot 60000 ms ligt
            if let Some(timeout) = tool.timeout
                && !(1..=60_000).contains(&timeout)
            {
                return Err(format!(
                    "Timeout voor tool '{}' moet tussen 1 en 60000 ms liggen",
                    tool.name
                ));
            }

            let mut parameter_names: HashSet<&str> = HashSet::new();

            // Controleer op dubbele parameter namen
            for parameter in &tool.parameters {              
                if parameter.name.trim().is_empty() {
                    return Err(format!(
                        "Parameternaam in tool '{}' mag niet leeg zijn",
                        tool.name
                    ));
                }
                if let Some(allowed_values) = &parameter.allowed_values {
                    if allowed_values.is_empty() {
                        return Err(format!(
                            "Toegestane waarden voor parameter '{}' in tool '{}' mogen niet leeg zijn",
                            parameter.name, tool.name
                        ));
                    }
                    if allowed_values.iter().any(|value| value.contains('\0')) {
                        return Err(format!(
                            "Toegestane waarden voor parameter '{}' in tool '{}' mogen geen NUL-teken bevatten",
                            parameter.name, tool.name
                        ));
                    }
                }
                if let Some(roots) = &parameter.allowed_roots {
                    if roots.is_empty() {
                        return Err("Toegestane roots mogen niet leeg zijn".to_owned());
                    }

                    for root in roots {
                        if !root.is_absolute() {
                            return Err("Toegestane root moet absoluut zijn".to_owned());
                        }

                        let canonical_root = root
                            .canonicalize()
                            .map_err(|_| "Toegestane root kan niet worden opgelost".to_owned())?;

                        if !canonical_root.is_dir() {
                            return Err("Toegestane root moet een directory zijn".to_owned());
                        }
                    }
                }
                if !parameter_names.insert(parameter.name.as_str()) {
                    return Err(format!("Dubbele parameter in tool '{}'", tool.name));
                }
            }

            let mut optional_argument_seen: bool = false;

            // Controleer of alle placeholders in de argumenten verwijzen naar bestaande parameters
            for argument in &tool.arguments {
                let is_optional = tool.parameters.iter().any(|parameter| {
                    !parameter.required
                        && argument == &format!("{{{}}}", parameter.name)
                });

                if optional_argument_seen {
                    return Err(format!(
                        "Optionele argumenten moeten achteraan staan in tool '{}'",
                        tool.name
                    ));
                }

                if is_optional {
                    let has_fixed_option = tool.arguments.iter().any(|argument| {
                        !argument.starts_with('{')
                            && (argument.starts_with('-') || argument.starts_with('/'))
                    });

                    if has_fixed_option {
                        return Err(format!(
                            "Optionele argumenten mogen niet worden gecombineerd met vaste opties in tool '{}'",
                            tool.name
                        ));
                    }
                }

                optional_argument_seen |= is_optional;

                if !argument.contains('{') && !argument.contains('}') {
                    continue;
                }

                // Controleer of de placeholder een volledig argument is, d.w.z. het moet beginnen met '{' en eindigen met '}'
                let Some(after_open_brace) = argument.strip_prefix('{') else {
                    return Err(format!(
                        "Placeholder moet een volledig argument zijn in tool '{}'",
                        tool.name
                    ));
                };

                // Controleer of de placeholder eindigt met '}'
                let Some(parameter_name) = after_open_brace.strip_suffix('}') else {
                    return Err(format!(
                        "Placeholder moet een volledig argument zijn in tool '{}'",
                        tool.name
                    ));
                };

                // Controleer of de placeholder geen lege naam heeft en geen extra '{' of '}' bevat
                if parameter_name.is_empty()
                    || parameter_name.contains('{')
                    || parameter_name.contains('}')
                {
                    return Err(format!("Ongeldige placeholder in tool '{}'", tool.name));
                }

                // Controleer of de placeholder verwijst naar een bestaande parameter
                if !parameter_names.contains(parameter_name) {
                    return Err(format!(
                        "Placeholder verwijst naar een onbekende parameter in tool '{}'",
                        tool.name
                    ));
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
// De definitie van een tool
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub command: String,
    pub arguments: Vec<String>,
    pub parameters: Vec<ParameterDefinition>,
    pub timeout: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
// De parameterdefinitie voor een tool
pub struct ParameterDefinition {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: ParameterType,
    pub required: bool,
    pub allowed_values: Option<Vec<String>>,
    pub allowed_roots: Option<Vec<std::path::PathBuf>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
// De parameter types die momenteel worden ondersteund
pub enum ParameterType {
    String,
}

/// Laad de configuratie uit het bestand dat wordt opgegeven door de omgevingsvariabele COMMANDHUB_CONFIG,
pub fn load() -> Result<ConfigFile, std::io::Error> {
    let path = std::env::var_os("COMMANDHUB_CONFIG")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("config/tools.json"));

    load_from_path(&path)
}

/// Laad de configuratie uit het opgegeven pad
fn load_from_path(path: &std::path::Path) -> Result<ConfigFile, std::io::Error> {
    let contents = std::fs::read_to_string(path).map_err(|error| {
        std::io::Error::new(
            error.kind(),
            format!(
                "Kan configuratiebestand {} niet lezen: {error}",
                path.display()
            ),
        )
    })?;

    // Parse de JSON en valideer de configuratie
    let config: ConfigFile = serde_json::from_str(&contents).map_err(|error| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "Ongeldige JSON in configuratiebestand {}: {error}",
                path.display()
            ),
        )
    })?;

    // Valideer de configuratie
    config.validate().map_err(|error| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "Ongeldige configuratie in bestand {}: {error}",
                path.display()
            ),
        )
    })?;

    Ok(config)
}

#[cfg(test)]
#[path = "tests/config_tests.rs"]
mod tests;
