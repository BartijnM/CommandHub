use super::*;
use crate::config::{ ParameterDefinition, ParameterType };

#[test]
fn missing_required_parameter_is_rejected() {
    let tool = ToolDefinition {
        name: "example".to_owned(),
        description: "Testtool".to_owned(),
        command: "unused".to_owned(),
        arguments: vec!["{message}".to_owned()],
        parameters: vec![ParameterDefinition {
            name: "message".to_owned(),
            kind: ParameterType::String,
            required: true,
            allowed_values: None,
            allowed_roots: None,
        }],
        timeout: None,
    };

    let input = Map::new();

    assert_eq!(validate_parameters(&tool, &input), Err("Verplichte parameter ontbreekt"));
}

#[test]
fn present_required_parameter_is_accepted() {
    let tool = ToolDefinition {
        name: "example".to_owned(),
        description: "Testtool".to_owned(),
        command: "unused".to_owned(),
        arguments: vec!["{message}".to_owned()],
        parameters: vec![ParameterDefinition {
            name: "message".to_owned(),
            kind: ParameterType::String,
            required: true,
            allowed_values: None,
            allowed_roots: None,
        }],
        timeout: None,
    };

    let mut input = Map::new();
    input.insert("message".to_owned(), Value::String("Hallo".to_owned()));

    assert_eq!(validate_parameters(&tool, &input), Ok(()));
}

#[test]
fn non_string_parameter_values_are_rejected() {
    let tool = ToolDefinition {
        name: "example".to_owned(),
        description: "Testtool".to_owned(),
        command: "unused".to_owned(),
        arguments: vec!["{message}".to_owned()],
        parameters: vec![ParameterDefinition {
            name: "message".to_owned(),
            kind: ParameterType::String,
            required: true,
            allowed_values: None,
            allowed_roots: None,
        }],
        timeout: None,
    };

    for value in [
        serde_json::json!(42),
        serde_json::json!(true),
        serde_json::json!(null),
        serde_json::json!([]),
        serde_json::json!({}),
    ] {
        let mut input = Map::new();
        input.insert("message".to_owned(), value);

        assert_eq!(
            validate_parameters(&tool, &input),
            Err("Parameter moet een string zijn")
        );
    }
}

#[test]
fn unknown_parameter_is_rejected() {
    let tool = validation_test_tool(true);
    let input = serde_json::json!({
        "message": "Hallo",
        "command": "other-program"
    });

    assert_eq!(
        validate_parameters(&tool, input.as_object().unwrap()),
        Err("Onbekende parameter")
    );
}

#[test]
fn missing_optional_parameter_is_accepted() {
    let tool = validation_test_tool(false);

    assert_eq!(validate_parameters(&tool, &Map::new()), Ok(()));
}

#[test]
fn present_optional_string_is_accepted() {
    let tool = validation_test_tool(false);
    let input = serde_json::json!({ "message": "Hallo" });

    assert_eq!(
        validate_parameters(&tool, input.as_object().unwrap()),
        Ok(())
    );
}

#[test]
fn optional_parameter_with_wrong_type_is_rejected() {
    let tool = validation_test_tool(false);
    let input = serde_json::json!({ "message": 42 });

    assert_eq!(
        validate_parameters(&tool, input.as_object().unwrap()),
        Err("Parameter moet een string zijn")
    );
}

#[test]
fn parameterless_tool_accepts_empty_input() {
    let mut tool = validation_test_tool(false);
    tool.parameters.clear();
    tool.arguments.clear();

    assert_eq!(validate_parameters(&tool, &Map::new()), Ok(()));
}

#[test]
fn parameterless_tool_rejects_input_fields() {
    let mut tool = validation_test_tool(false);
    tool.parameters.clear();
    tool.arguments.clear();
    let input = serde_json::json!({ "message": "Hallo" });

    assert_eq!(
        validate_parameters(&tool, input.as_object().unwrap()),
        Err("Onbekende parameter")
    );
}

fn validation_test_tool(required: bool) -> ToolDefinition {
    ToolDefinition {
        name: "example".to_owned(),
        description: "Testtool".to_owned(),
        command: "unused".to_owned(),
        arguments: vec!["{message}".to_owned()],
        parameters: vec![ParameterDefinition {
            name: "message".to_owned(),
            kind: ParameterType::String,
            required,
            allowed_values: None,
            allowed_roots: None,
        }],
        timeout: None,
    }
}

#[test]
fn placeholder_is_replaced_and_argument_order_is_preserved() {
    let mut tool = validation_test_tool(true);
    tool.arguments = vec![
        "before".to_owned(),
        "{message}".to_owned(),
        "after".to_owned(),
    ];
    let input = serde_json::json!({ "message": "Hallo wereld" });

    assert_eq!(
        build_arguments(&tool, input.as_object().unwrap()),
        Ok(vec![
            "before".to_owned(),
            "Hallo wereld".to_owned(),
            "after".to_owned(),
        ])
    );
}

#[test]
fn repeated_placeholder_produces_separate_arguments() {
    let mut tool = validation_test_tool(true);
    tool.arguments = vec!["{message}".to_owned(), "{message}".to_owned()];
    let input = serde_json::json!({ "message": "Hallo" });

    assert_eq!(
        build_arguments(&tool, input.as_object().unwrap()),
        Ok(vec!["Hallo".to_owned(), "Hallo".to_owned()])
    );
}

#[test]
fn missing_optional_placeholder_is_omitted() {
    let mut tool = validation_test_tool(false);
    tool.arguments = vec!["before".to_owned(), "{message}".to_owned()];

    assert_eq!(
        build_arguments(&tool, &Map::new()),
        Ok(vec!["before".to_owned()])
    );
}

#[test]
fn empty_string_remains_an_argument() {
    let tool = validation_test_tool(true);
    let input = serde_json::json!({ "message": "" });

    assert_eq!(
        build_arguments(&tool, input.as_object().unwrap()),
        Ok(vec![String::new()])
    );
}

#[test]
fn parameterless_arguments_remain_unchanged() {
    let mut tool = validation_test_tool(false);
    tool.parameters.clear();
    tool.arguments = vec!["--version".to_owned()];

    assert_eq!(
        build_arguments(&tool, &Map::new()),
        Ok(vec!["--version".to_owned()])
    );
}

#[test]
fn argument_building_rejects_invalid_input() {
    let tool = validation_test_tool(true);

    for (input, expected_error) in [
        (
            serde_json::json!({}),
            "Verplichte parameter ontbreekt",
        ),
        (
            serde_json::json!({ "message": 42 }),
            "Parameter moet een string zijn",
        ),
        (
            serde_json::json!({ "message": "Hallo", "extra": true }),
            "Onbekende parameter",
        ),
    ] {
        assert_eq!(
            build_arguments(&tool, input.as_object().unwrap()),
            Err(expected_error)
        );
    }
}

#[test]
fn string_with_nul_is_rejected() {
    let tool = validation_test_tool(true);
    let input = serde_json::json!({ "message": "Hallo\u{0000}wereld" });
    let input = input.as_object().unwrap();

    assert_eq!(
        validate_parameters(&tool, input),
        Err("Parameter mag geen NUL-teken bevatten")
    );
    assert_eq!(
        build_arguments(&tool, input),
        Err("Parameter mag geen NUL-teken bevatten")
    );
}

#[test]
fn unicode_string_is_preserved() {
    let tool = validation_test_tool(true);
    let input = serde_json::json!({ "message": "Goedemorgen 🌍 café" });

    assert_eq!(
        build_arguments(&tool, input.as_object().unwrap()),
        Ok(vec!["Goedemorgen 🌍 café".to_owned()])
    );
}

#[test]
fn configured_allowed_value_is_accepted() {
    let mut tool = validation_test_tool(true);
    tool.parameters[0].allowed_values =
        Some(vec!["short".to_owned(), "long".to_owned()]);
    let input = serde_json::json!({ "message": "short" });

    assert_eq!(
        build_arguments(&tool, input.as_object().unwrap()),
        Ok(vec!["short".to_owned()])
    );
}

#[test]
fn values_outside_allowed_list_are_rejected() {
    let mut tool = validation_test_tool(true);
    tool.parameters[0].allowed_values =
        Some(vec!["short".to_owned(), "long".to_owned()]);

    for value in ["other", "--help", "Short", "short ", ""] {
        let input = serde_json::json!({ "message": value });

        assert_eq!(
            build_arguments(&tool, input.as_object().unwrap()),
            Err("Parameterwaarde is niet toegestaan")
        );
    }
}

#[test]
fn missing_optional_parameter_with_allowed_values_is_accepted() {
    let mut tool = validation_test_tool(false);
    tool.parameters[0].allowed_values = Some(vec!["short".to_owned()]);

    assert_eq!(build_arguments(&tool, &Map::new()), Ok(Vec::new()));
}

#[test]
fn free_string_starting_with_hyphen_is_rejected() {
    let tool = validation_test_tool(true);

    for value in ["--help", "-n", "-", "-42"] {
        let input = serde_json::json!({ "message": value });

        assert_eq!(
            build_arguments(&tool, input.as_object().unwrap()),
            Err("Vrije parameterwaarde mag niet met '-' beginnen")
        );
    }
}

#[test]
fn explicitly_allowed_option_value_is_accepted() {
    let mut tool = validation_test_tool(true);
    tool.parameters[0].allowed_values = Some(vec!["--help".to_owned()]);
    let input = serde_json::json!({ "message": "--help" });

    assert_eq!(
        build_arguments(&tool, input.as_object().unwrap()),
        Ok(vec!["--help".to_owned()])
    );
}

#[test]
fn path_inside_root_becomes_canonical_argument() {
    let root = std::env::current_dir().unwrap();
    let path = root.join("src").join("..").join("Cargo.toml");
    let mut tool = validation_test_tool(true);
    tool.parameters[0].allowed_roots = Some(vec![root]);

    let input = serde_json::json!({
        "message": path.to_str().unwrap()
    });
    let expected = path.canonicalize().unwrap();

    assert_eq!(
        build_arguments(&tool, input.as_object().unwrap()),
        Ok(vec![expected.into_os_string().into_string().unwrap()])
    );
}

#[test]
fn path_outside_root_is_rejected() {
    let project = std::env::current_dir().unwrap();
    let mut tool = validation_test_tool(true);
    tool.parameters[0].allowed_roots = Some(vec![project.join("src")]);

    let input = serde_json::json!({
        "message": project.join("Cargo.toml").to_str().unwrap()
    });

    assert_eq!(
        build_arguments(&tool, input.as_object().unwrap()),
        Err("Pad ligt buiten toegestane roots")
    );
}

#[test]
fn parent_traversal_outside_root_is_rejected() {
    let project = std::env::current_dir().unwrap();
    let mut tool = validation_test_tool(true);
    tool.parameters[0].allowed_roots = Some(vec![project.join("src")]);

    let input = serde_json::json!({
        "message": project
            .join("src")
            .join("..")
            .join("Cargo.toml")
            .to_str()
            .unwrap()
    });

    assert_eq!(
        build_arguments(&tool, input.as_object().unwrap()),
        Err("Pad ligt buiten toegestane roots")
    );
}

#[test]
fn relative_client_path_is_rejected() {
    let mut tool = validation_test_tool(true);
    tool.parameters[0].allowed_roots =
        Some(vec![std::env::current_dir().unwrap()]);
    let input = serde_json::json!({ "message": "Cargo.toml" });

    assert_eq!(
        build_arguments(&tool, input.as_object().unwrap()),
        Err("Pad moet absoluut zijn")
    );
}

#[test]
fn nonexistent_client_path_is_rejected() {
    let project = std::env::current_dir().unwrap();
    let missing = project
        .join("Cargo.toml")
        .join("cannot-exist-under-a-file");
    let mut tool = validation_test_tool(true);
    tool.parameters[0].allowed_roots = Some(vec![project]);

    let input = serde_json::json!({
        "message": missing.to_str().unwrap()
    });

    assert_eq!(
        build_arguments(&tool, input.as_object().unwrap()),
        Err("Pad kan niet worden opgelost")
    );
}

#[test]
fn sibling_directory_is_not_inside_root() {
    let project = std::env::current_dir().unwrap();
    let mut tool = validation_test_tool(true);
    tool.parameters[0].allowed_roots =
        Some(vec![project.join("src").join("controllers")]);

    let input = serde_json::json!({
        "message": project
            .join("src")
            .join("models")
            .to_str()
            .unwrap()
    });

    assert_eq!(
        build_arguments(&tool, input.as_object().unwrap()),
        Err("Pad ligt buiten toegestane roots")
    );
}

struct PathFixture {
    base: std::path::PathBuf,
}

impl PathFixture {
    fn new(name: &str) -> Self {
        let base = std::env::temp_dir().join(format!(
            "commandhub-validation-{name}-{}",
            std::process::id()
        ));

        std::fs::create_dir(&base).expect("testdirectory moet nieuw zijn");
        std::fs::create_dir(base.join("data")).unwrap();
        std::fs::create_dir(base.join("data-extra")).unwrap();
        std::fs::write(base.join("data").join("inside.txt"), "inside").unwrap();
        std::fs::write(base.join("data-extra").join("outside.txt"), "outside").unwrap();

        Self { base }
    }
}

impl Drop for PathFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.base);
    }
}

#[test]
fn similar_directory_prefix_is_rejected() {
    let fixture = PathFixture::new("prefix");
    let mut tool = validation_test_tool(true);
    tool.parameters[0].allowed_roots = Some(vec![fixture.base.join("data")]);

    let input = serde_json::json!({
        "message": fixture.base.join("data-extra").join("outside.txt")
    });

    assert_eq!(
        build_arguments(&tool, input.as_object().unwrap()),
        Err("Pad ligt buiten toegestane roots")
    );
}

#[cfg(unix)]
#[test]
fn symlink_targets_are_checked_against_root() {
    use std::os::unix::fs::symlink;

    let fixture = PathFixture::new("symlink");
    let root = fixture.base.join("data");
    let inside = root.join("inside.txt");
    let outside = fixture.base.join("data-extra").join("outside.txt");

    symlink(&inside, root.join("inside-link")).unwrap();
    symlink(&outside, root.join("outside-link")).unwrap();

    let mut tool = validation_test_tool(true);
    tool.parameters[0].allowed_roots = Some(vec![root.clone()]);

    let input = serde_json::json!({
        "message": root.join("inside-link")
    });
    assert_eq!(
        build_arguments(&tool, input.as_object().unwrap()),
        Ok(vec![
            inside
                .canonicalize()
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap()
        ])
    );

    let input = serde_json::json!({
        "message": root.join("outside-link")
    });
    assert_eq!(
        build_arguments(&tool, input.as_object().unwrap()),
        Err("Pad ligt buiten toegestane roots")
    );
}

#[test]
fn free_option_and_response_file_prefixes_are_rejected() {
    let tool = validation_test_tool(true);

    for value in ["/HELP", "/OUT:file", "@options.txt"] {
        let input = serde_json::json!({ "message": value });

        assert_eq!(
            build_arguments(&tool, input.as_object().unwrap()),
            Err("Vrije parameterwaarde heeft een verboden prefix")
        );
    }
}

#[test]
fn explicitly_allowed_prefixed_values_are_accepted() {
    for value in ["/HELP", "@options.txt"] {
        let mut tool = validation_test_tool(true);
        tool.parameters[0].allowed_values = Some(vec![value.to_owned()]);
        let input = serde_json::json!({ "message": value });

        assert_eq!(
            build_arguments(&tool, input.as_object().unwrap()),
            Ok(vec![value.to_owned()])
        );
    }
}