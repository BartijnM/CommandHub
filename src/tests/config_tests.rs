use super::*;

fn test_tool(name: &str, timeout: Option<u64>) -> ToolDefinition {
    ToolDefinition {
        name: name.to_owned(),
        description: "Test tool".to_owned(),
        command: "test-command".to_owned(),
        arguments: Vec::new(),
        parameters: Vec::new(),
        timeout,
    }
}

fn test_parameter(name: &str) -> ParameterDefinition {
    ParameterDefinition {
        name: name.to_owned(),
        kind: ParameterType::String,
        required: true,
    }
}

#[test]
// Test dat het ontbreken van de timeout resulteert in None
fn missing_timeout_becomes_none() {
    let json = r#"
        {
            "tools": [
                {
                    "name": "git_version",
                    "description": "Show the installed Git version",
                    "command": "git",
                    "arguments": ["--version"],
                    "parameters": []
                }
            ]
        }
        "#;

    let config: ConfigFile =
        serde_json::from_str(json).expect("geldige configuratie moet worden ingelezen");

    assert_eq!(config.tools[0].timeout, None);
}

#[test]
// Test dat onbekende velden in de tooldefinitie worden afgewezen
fn unknown_tool_field_is_rejected() {
    let json = r#"
        {
            "tools": [
                {
                    "name": "git_version",
                    "description": "Show the installed Git version",
                    "command": "git",
                    "arguments": ["--version"],
                    "parameters": [],
                    "unexpected": true
                }
            ]
        }
        "#;

    assert!(serde_json::from_str::<ConfigFile>(json).is_err());
}

#[test]
// Test dat dubbele toolnamen worden afgewezen
fn duplicate_tool_names_are_rejected() {
    let config = ConfigFile {
        tools: vec![test_tool("duplicate", None), test_tool("duplicate", None)],
    };

    assert!(config.validate().is_err());
}

#[test]
// Test dat timeouts buiten het toegestane bereik worden afgewezen
fn timeout_outside_allowed_range_is_rejected() {
    for timeout in [Some(0), Some(60_001)] {
        let config = ConfigFile {
            tools: vec![test_tool("test_tool", timeout)],
        };

        assert!(config.validate().is_err());
    }
}

#[test]
// Test dat een geldige volledige placeholder wordt geaccepteerd
fn valid_full_placeholder_is_accepted() {
    let mut tool = test_tool("list_files", None);
    tool.arguments = vec!["{path}".to_owned()];
    tool.parameters = vec![test_parameter("path")];

    let config = ConfigFile { tools: vec![tool] };

    assert!(config.validate().is_ok());
}

#[test]
// Test dat een onbekende placeholder wordt afgewezen
fn unknown_placeholder_is_rejected() {
    let mut tool = test_tool("list_files", None);
    tool.arguments = vec!["{missing}".to_owned()];
    tool.parameters = vec![test_parameter("path")];

    let config = ConfigFile { tools: vec![tool] };

    assert!(config.validate().is_err());
}

#[test]
// Test dat een gedeeltelijke placeholder wordt afgewezen
fn partial_placeholder_is_rejected() {
    let mut tool = test_tool("list_files", None);
    tool.arguments = vec!["prefix-{path}".to_owned()];
    tool.parameters = vec![test_parameter("path")];

    let config = ConfigFile { tools: vec![tool] };

    assert!(config.validate().is_err());
}

#[test]
// Test dat dubbele parameter namen worden afgewezen
fn duplicate_parameter_names_are_rejected() {
    let mut tool = test_tool("list_files", None);
    tool.parameters = vec![test_parameter("path"), test_parameter("path")];

    let config = ConfigFile { tools: vec![tool] };

    assert!(config.validate().is_err());
}

#[test]
// Test dat ongeldige JSON wordt afgewezen
fn invalid_json_is_rejected() {
    let result = serde_json::from_str::<ConfigFile>(r#"{"tools":["#);

    assert!(result.is_err());
}

#[test]
// Test dat ontbrekende verplichte velden in de tooldefinitie worden afgewezen
fn missing_required_tool_field_is_rejected() {
    let json = r#"
    {
        "tools": [
            {
                "name": "git_version",
                "description": "Show the installed Git version",
                "arguments": [],
                "parameters": []
            }
        ]
    }
    "#;

    let result = serde_json::from_str::<ConfigFile>(json);

    assert!(result.is_err());
}

#[test]
// Test dat het ontbreken van het configuratiebestand resulteert in een NotFound-fout
fn missing_config_file_returns_not_found() {
    let path =
        std::env::temp_dir().join(format!("commandhub-missing-config-{}", std::process::id()));

    let error = load_from_path(&path).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
}

#[test]
// Test dat een ongeldige JSON in het configuratiebestand resulteert in een InvalidData-fout
fn invalid_json_file_returns_invalid_data() {
    let path = std::env::temp_dir().join(format!(
        "commandhub-invalid-json-{}.json",
        std::process::id()
    ));

    std::fs::write(&path, r#"{"tools:["#).expect("testbestand moet geschreven kunnen worden");

    let error = load_from_path(&path).expect_err("kapotte JSON moet worden afgewezen");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("Ongeldige JSON"));

    std::fs::remove_file(path).expect("testbestand moet opgeruimd kunnen worden");
}

#[test]
// Test dat een configuratiebestand met dubbele toolnamen resulteert in een InvalidData-fout
fn invalid_config_file_returns_invalid_data() {
    let path = std::env::temp_dir().join(format!(
        "commandhub-invalid-config-{}.json",
        std::process::id()
    ));

    let json = r#"{"tools":[
            {"name":"duplicate","description":"First","command":"test","arguments":[],"parameters":[]},
            {"name":"duplicate","description":"Second","command":"test","arguments":[],"parameters":[]}
        ]}"#;

    std::fs::write(&path, json).expect("testbestand moet geschreven kunnen worden");

    let error = load_from_path(&path).expect_err("dubbele toolnamen moeten worden afgewezen");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("Ongeldige configuratie"));

    std::fs::remove_file(path).expect("testbestand moet opgeruimd kunnen worden");
}
