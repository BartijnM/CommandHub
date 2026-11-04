use super::*;

#[test]
fn macos_example_config_is_valid() {
    let config: ConfigFile = serde_json::from_str(include_str!("../../config/tools.macos.json"))
        .expect("macOS-voorbeeld moet geldige JSON zijn");
    config.validate().expect("macOS-voorbeeld moet geldig zijn");
}

#[test]
fn blank_tool_names_commands_and_parameter_names_are_rejected() {
    for blank in ["", " ", "\t\n"] {
        let config = ConfigFile {
            tools: vec![test_tool(blank, None)],
        };
        assert!(
            config.validate().is_err(),
            "lege toolnaam moet worden afgewezen"
        );
        let mut tool = test_tool("test_tool", None);
        tool.command = blank.to_owned();
        assert!(
            ConfigFile { tools: vec![tool] }.validate().is_err(),
            "leeg command moet worden afgewezen"
        );
        let mut tool = test_tool("test_tool", None);
        tool.parameters = vec![test_parameter(blank)];
        assert!(
            ConfigFile { tools: vec![tool] }.validate().is_err(),
            "lege parameternaam moet worden afgewezen"
        );
    }
}

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
        allowed_values: None,
        allowed_roots: None,
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

#[test]
fn empty_allowed_values_are_rejected() {
    let mut tool = test_tool("example", None);
    let mut parameter = test_parameter("message");
    parameter.allowed_values = Some(Vec::new());
    tool.parameters = vec![parameter];

    let error = ConfigFile { tools: vec![tool] }
        .validate()
        .expect_err("lege lijst moet worden afgewezen");

    assert!(error.contains("Toegestane waarden"));
    assert!(error.contains("mogen niet leeg zijn"));
}

#[test]
fn nonempty_allowed_values_are_accepted() {
    let mut tool = test_tool("example", None);
    tool.arguments = vec!["{message}".to_owned()];
    let mut parameter = test_parameter("message");
    parameter.allowed_values = Some(vec!["short".to_owned(), "long".to_owned()]);
    tool.parameters = vec![parameter];

    assert!(ConfigFile { tools: vec![tool] }.validate().is_ok());
}

#[test]
fn missing_allowed_values_becomes_none() {
    let parameter: ParameterDefinition =
        serde_json::from_str(r#"{"name":"message","type":"string","required":true}"#)
            .expect("parameter zonder allowed_values moet leesbaar blijven");

    assert_eq!(parameter.allowed_values, None);
}

#[test]
fn allowed_values_are_read_from_json() {
    let parameter: ParameterDefinition = serde_json::from_str(
        r#"{
            "name":"message",
            "type":"string",
            "required":true,
            "allowed_values":["short","long"]
        }"#,
    )
    .expect("parameter met allowed_values moet leesbaar zijn");

    assert_eq!(
        parameter.allowed_values,
        Some(vec!["short".to_owned(), "long".to_owned()])
    );
}

#[test]
fn allowed_value_with_nul_is_rejected() {
    let mut tool = test_tool("example", None);
    let mut parameter = test_parameter("message");
    parameter.allowed_values = Some(vec!["Hallo\u{0000}wereld".to_owned()]);
    tool.parameters = vec![parameter];

    let error = ConfigFile { tools: vec![tool] }
        .validate()
        .expect_err("toegestane waarde met NUL moet worden afgewezen");

    assert!(error.contains("mogen geen NUL-teken bevatten"));
}

#[test]
fn empty_string_as_allowed_value_is_accepted() {
    let mut tool = test_tool("example", None);
    tool.arguments = vec!["{message}".to_owned()];
    let mut parameter = test_parameter("message");
    parameter.allowed_values = Some(vec![String::new()]);
    tool.parameters = vec![parameter];

    assert!(ConfigFile { tools: vec![tool] }.validate().is_ok());
}

#[test]
fn empty_allowed_roots_are_rejected() {
    let mut tool = test_tool("example", None);
    let mut parameter = test_parameter("path");
    parameter.allowed_roots = Some(Vec::new());
    tool.parameters = vec![parameter];

    assert_eq!(
        ConfigFile { tools: vec![tool] }.validate(),
        Err("Toegestane roots mogen niet leeg zijn".to_owned())
    );
}

#[test]
fn relative_allowed_root_is_rejected() {
    let mut tool = test_tool("example", None);
    let mut parameter = test_parameter("path");
    parameter.allowed_roots = Some(vec![std::path::PathBuf::from("relative")]);
    tool.parameters = vec![parameter];

    assert_eq!(
        ConfigFile { tools: vec![tool] }.validate(),
        Err("Toegestane root moet absoluut zijn".to_owned())
    );
}

#[test]
fn existing_absolute_directory_is_accepted_as_root() {
    let mut tool = test_tool("example", None);
    tool.arguments = vec!["{path}".to_owned()];
    let mut parameter = test_parameter("path");
    parameter.allowed_roots = Some(vec![
        std::env::current_dir().expect("projectdirectory moet beschikbaar zijn"),
    ]);
    tool.parameters = vec![parameter];

    assert!(ConfigFile { tools: vec![tool] }.validate().is_ok());
}

#[test]
fn file_is_rejected_as_allowed_root() {
    let mut tool = test_tool("example", None);
    let mut parameter = test_parameter("path");
    parameter.allowed_roots = Some(vec![
        std::env::current_dir()
            .expect("projectdirectory moet beschikbaar zijn")
            .join("Cargo.toml"),
    ]);
    tool.parameters = vec![parameter];

    assert_eq!(
        ConfigFile { tools: vec![tool] }.validate(),
        Err("Toegestane root moet een directory zijn".to_owned())
    );
}

#[test]
fn optional_placeholder_at_end_is_accepted() {
    let mut tool = test_tool("example", None);
    let mut optional = test_parameter("optional");
    optional.required = false;

    tool.parameters = vec![test_parameter("required"), optional];
    tool.arguments = vec!["{required}".to_owned(), "{optional}".to_owned()];

    assert!(ConfigFile { tools: vec![tool] }.validate().is_ok());
}

#[test]
fn required_placeholder_after_optional_is_rejected() {
    let mut tool = test_tool("example", None);
    let mut optional = test_parameter("optional");
    optional.required = false;

    tool.parameters = vec![optional, test_parameter("required")];
    tool.arguments = vec!["{optional}".to_owned(), "{required}".to_owned()];

    assert_eq!(
        ConfigFile { tools: vec![tool] }.validate(),
        Err("Optionele argumenten moeten achteraan staan in tool 'example'".to_owned())
    );
}

#[test]
fn fixed_argument_after_optional_is_rejected() {
    let mut tool = test_tool("example", None);
    let mut optional = test_parameter("optional");
    optional.required = false;

    tool.parameters = vec![optional];
    tool.arguments = vec!["{optional}".to_owned(), "fixed".to_owned()];

    assert_eq!(
        ConfigFile { tools: vec![tool] }.validate(),
        Err("Optionele argumenten moeten achteraan staan in tool 'example'".to_owned())
    );
}

#[test]
fn second_optional_placeholder_is_rejected() {
    let mut tool = test_tool("example", None);
    let mut first = test_parameter("first");
    first.required = false;
    let mut second = test_parameter("second");
    second.required = false;

    tool.parameters = vec![first, second];
    tool.arguments = vec!["{first}".to_owned(), "{second}".to_owned()];

    assert_eq!(
        ConfigFile { tools: vec![tool] }.validate(),
        Err("Optionele argumenten moeten achteraan staan in tool 'example'".to_owned())
    );
}

#[test]
fn fixed_options_with_optional_placeholder_are_rejected() {
    for option in ["--output", "-o", "/OUT", "--"] {
        let mut tool = test_tool("example", None);
        let mut optional = test_parameter("optional");
        optional.required = false;

        tool.parameters = vec![optional];
        tool.arguments = vec![option.to_owned(), "{optional}".to_owned()];

        assert_eq!(
            ConfigFile { tools: vec![tool] }.validate(),
            Err(
                "Optionele argumenten mogen niet worden gecombineerd met vaste opties in tool 'example'"
                    .to_owned()
            ),
            "vaste optie {option} moet worden afgewezen"
        );
    }
}

#[test]
fn fixed_option_with_required_placeholder_is_accepted() {
    let mut tool = test_tool("example", None);
    tool.parameters = vec![test_parameter("output")];
    tool.arguments = vec!["--output".to_owned(), "{output}".to_owned()];

    assert!(ConfigFile { tools: vec![tool] }.validate().is_ok());
}

#[test]
fn fixed_positional_argument_with_optional_placeholder_is_accepted() {
    let mut tool = test_tool("example", None);
    let mut optional = test_parameter("optional");
    optional.required = false;

    tool.parameters = vec![optional];
    tool.arguments = vec!["show".to_owned(), "{optional}".to_owned()];

    assert!(ConfigFile { tools: vec![tool] }.validate().is_ok());
}

#[test]
fn parameter_without_placeholder_is_rejected() {
    let mut tool = test_tool("example", None);
    tool.parameters = vec![test_parameter("message")];

    assert_eq!(
        ConfigFile { tools: vec![tool] }.validate(),
        Err("Parameter 'message' in tool 'example' heeft geen placeholder".to_owned())
    );
}

#[test]
fn path_parameter_without_placeholder_is_rejected() {
    let mut tool = test_tool("example", None);
    let mut parameter = test_parameter("path");
    parameter.allowed_roots = Some(vec![std::env::current_dir().unwrap()]);
    tool.parameters = vec![parameter];

    assert_eq!(
        ConfigFile { tools: vec![tool] }.validate(),
        Err("Parameter 'path' in tool 'example' heeft geen placeholder".to_owned())
    );
}

#[test]
fn repeated_parameter_placeholder_is_accepted() {
    let mut tool = test_tool("example", None);
    tool.parameters = vec![test_parameter("message")];
    tool.arguments = vec!["{message}".to_owned(), "{message}".to_owned()];

    assert!(ConfigFile { tools: vec![tool] }.validate().is_ok());
}

#[test]
fn nonexistent_allowed_root_is_rejected() {
    let mut tool = test_tool("example", None);
    tool.arguments = vec!["{path}".to_owned()];

    let mut parameter = test_parameter("path");
    parameter.allowed_roots = Some(vec![
        std::env::current_dir()
            .unwrap()
            .join("Cargo.toml")
            .join("cannot-exist"),
    ]);
    tool.parameters = vec![parameter];

    assert_eq!(
        ConfigFile { tools: vec![tool] }.validate(),
        Err("Toegestane root kan niet worden opgelost".to_owned())
    );
}

#[test]
fn f04_example_config_is_valid() {
    let config: ConfigFile = serde_json::from_str(include_str!("../../config/tools.json")).unwrap();
    config.validate().expect("F04-voorbeeld moet geldig zijn");
}
