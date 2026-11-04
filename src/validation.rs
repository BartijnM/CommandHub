use crate::config::ToolDefinition;
use serde_json::{Map, Value};

pub fn validate_parameters(
    tool: &ToolDefinition,
    input: &Map<String, Value>,
) -> Result<(), &'static str> {
    for name in input.keys() {
        if !tool.parameters.iter().any(|parameter| parameter.name == *name) {
            return Err("Onbekende parameter");
        }
    }    
    for parameter in &tool.parameters {
        if parameter.required && !input.contains_key(&parameter.name) {
            return Err("Verplichte parameter ontbreekt");
        }
        if let Some(value) = input.get(&parameter.name) {
            match parameter.kind {
                crate::config::ParameterType::String => {
                    let value = value
                        .as_str()
                        .ok_or("Parameter moet een string zijn")?;

                    if value.contains('\0') {
                        return Err("Parameter mag geen NUL-teken bevatten");
                    }
                    if parameter.allowed_values.is_none()
                        && parameter.allowed_roots.is_none()
                        && (value.starts_with('/') || value.starts_with('@'))
                    {
                        return Err("Vrije parameterwaarde heeft een verboden prefix");
                    }
                    if parameter.allowed_values.is_none() && value.starts_with('-') {
                        return Err("Vrije parameterwaarde mag niet met '-' beginnen");
                    }
                    if let Some(allowed_values) = &parameter.allowed_values {
                        if !allowed_values.iter().any(|allowed| allowed == value) {
                            return Err("Parameterwaarde is niet toegestaan");
                        }
                    }
                }
            }
        }        
    }

    Ok(())
}

fn validate_path(
    value: &str,
    roots: &[std::path::PathBuf],
) -> Result<std::path::PathBuf, &'static str> {
    let path = std::path::Path::new(value);

    if !path.is_absolute() {
        return Err("Pad moet absoluut zijn");
    }

    let canonical_path = path
        .canonicalize()
        .map_err(|_| "Pad kan niet worden opgelost")?;

    for root in roots {
        let canonical_root = root
            .canonicalize()
            .map_err(|_| "Toegestane root kan niet worden opgelost")?;

        if canonical_path.starts_with(&canonical_root) {
            return Ok(canonical_path);
        }
    }

    Err("Pad ligt buiten toegestane roots")
}

pub fn build_arguments(
    tool: &ToolDefinition,
    input: &Map<String, Value>,
) -> Result<Vec<String>, &'static str> {
    validate_parameters(tool, input)?;

    let mut arguments = Vec::new();

    for argument in &tool.arguments {
        if let Some(name) = argument
            .strip_prefix('{')
            .and_then(|name| name.strip_suffix('}'))
        {
            if let Some(value) = input.get(name) {
                let value = value.as_str().ok_or("Parameter moet een string zijn")?;
                let parameter = tool
                    .parameters
                    .iter()
                    .find(|parameter| parameter.name == name)
                    .ok_or("Onbekende placeholder")?;

                if let Some(roots) = &parameter.allowed_roots {
                    let path = validate_path(value, roots)?;
                    let path = path
                        .into_os_string()
                        .into_string()
                        .map_err(|_| "Pad kan niet als tekst worden doorgegeven")?;

                    arguments.push(path);
                } else {
                    arguments.push(value.to_owned());
                }
            }
        } else {
            arguments.push(argument.clone());
        }
    }

    Ok(arguments)
}

#[cfg(test)]
#[path = "tests/validation_tests.rs"]
mod tests;