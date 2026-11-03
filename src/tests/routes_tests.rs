use std::sync::Arc;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

use crate::{
    config::{ConfigFile, ParameterDefinition, ParameterType, ToolDefinition},
    registry::ToolRegistry,
};

use super::router;

fn post_request(tool: &str, input: impl Into<Body>) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(format!("/tools/{tool}"))
        .header("content-type", "application/json")
        .body(input.into())
        .expect("testrequest moet geldig zijn")
}

async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("response moet leesbaar zijn");
    serde_json::from_slice(&body).expect("response moet JSON zijn")
}

fn test_registry() -> Arc<ToolRegistry> {
    let config = ConfigFile {
        tools: vec![ToolDefinition {
            name: "list_files".to_owned(),
            description: "List files in a directory".to_owned(),
            command: "private-command".to_owned(),
            arguments: vec!["{path}".to_owned()],
            parameters: vec![ParameterDefinition {
                name: "path".to_owned(),
                kind: ParameterType::String,
                required: true,
            }],
            timeout: None,
        }],
    };

    Arc::new(ToolRegistry::new(config))
}

#[tokio::test]
// Test of de /tools route de juiste tool metadata en input schema retourneert
async fn tools_route_returns_tool_metadata_and_input_schema() {
    let response = router(crate::app_state::AppState {
        registry: test_registry(),
        execution: Arc::new(crate::execution::ExecutionService::new()),
    })
    .oneshot(
        Request::builder()
            .method("GET")
            .uri("/tools")
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let actual: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let expected = json!({
        "tools": [
            {
                "name": "list_files",
                "description": "List files in a directory",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string"
                        }
                    },
                    "required": ["path"]
                }
            }
        ]
    });

    assert_eq!(actual, expected);
}

#[tokio::test]
async fn executes_parameterless_tool_via_http() {
    let registry = Arc::new(ToolRegistry::new(ConfigFile {
        tools: vec![ToolDefinition {
            name: "git_versie".to_owned(),
            description: "Toon de Git-versie".to_owned(),
            command: "git".to_owned(),
            arguments: vec!["--version".to_owned()],
            parameters: Vec::new(),
            timeout: Some(10_000),
        }],
    }));

    let state = crate::app_state::AppState {
        registry,
        execution: Arc::new(crate::execution::ExecutionService::new()),
    };

    let response = router(state)
        .oneshot(post_request("git_versie", "{}"))
        .await
        .expect("de router moet een response geven");

    assert_eq!(response.status(), StatusCode::OK);

    let result = response_json(response).await;

    assert_eq!(result["success"], json!(true));
    assert_eq!(result["exitCode"], json!(0));
    assert_eq!(result["stderr"], json!(""));
    assert!(
        result["stdout"]
            .as_str()
            .expect("stdout moet tekst zijn")
            .starts_with("git version ")
    );
    assert!(result["duration"].is_u64());
}

#[tokio::test]
async fn rejects_oversized_request_body_with_413() {
    let bodies = [
        format!("{{}}{}", " ".repeat(16 * 1024)),
        "x".repeat(16 * 1024 + 1),
    ];

    for body in bodies {
        let state = crate::app_state::AppState {
            registry: test_registry(),
            execution: Arc::new(crate::execution::ExecutionService::new()),
        };

        let response = router(state)
            .oneshot(post_request("onbekend", body))
            .await
            .expect("de router moet een response geven");

        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);

        let actual = response_json(response).await;

        assert_eq!(
            actual,
            json!({
                "code": "request_too_large",
                "message": "De requestbody mag maximaal 16 KiB groot zijn"
            })
        );
    }
}

#[tokio::test]
async fn accepts_request_body_of_exactly_16_kib() {
    let state = crate::app_state::AppState {
        registry: test_registry(),
        execution: Arc::new(crate::execution::ExecutionService::new()),
    };

    let input = format!("{{}}{}", " ".repeat(16 * 1024 - 2));
    assert_eq!(input.len(), 16 * 1024);

    let response = router(state)
        .oneshot(post_request("onbekend", input))
        .await
        .expect("de router moet een response geven");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let actual = response_json(response).await;

    assert_eq!(
        actual,
        json!({
            "code": "unknown_tool",
            "message": "Onbekende tool"
        })
    );
}

#[tokio::test]
async fn rejects_invalid_input_via_http() {
    let cases = [
        ("{", Some("application/json"), "invalid_json"),
        ("{}", None, "invalid_json"),
        ("[]", Some("application/json"), "invalid_input"),
        ("null", Some("application/json"), "invalid_input"),
        (
            r#"{"command":"ongewenst"}"#,
            Some("application/json"),
            "invalid_input",
        ),
    ];

    for (input, content_type, expected_code) in cases {
        let state = crate::app_state::AppState {
            registry: test_registry(),
            execution: Arc::new(crate::execution::ExecutionService::new()),
        };

        let mut request = Request::builder().method("POST").uri("/tools/onbekend");

        if let Some(content_type) = content_type {
            request = request.header("content-type", content_type);
        }

        let response = router(state)
            .oneshot(
                request
                    .body(Body::from(input))
                    .expect("het testrequest moet geldig zijn"),
            )
            .await
            .expect("de router moet een response geven");

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "verwachtte 400 voor invoer {input} met contenttype {content_type:?}"
        );

        let actual = response_json(response).await;

        assert_eq!(actual["code"], json!(expected_code));
    }
}

#[tokio::test]
async fn nonzero_exit_code_returns_http_200_with_success_false() {
    let registry = Arc::new(ToolRegistry::new(ConfigFile {
        tools: vec![ToolDefinition {
            name: "git_fout".to_owned(),
            description: "Test een niet-succesvolle procesafloop".to_owned(),
            command: "git".to_owned(),
            arguments: vec!["--invalid-commandhub-test".to_owned()],
            parameters: Vec::new(),
            timeout: Some(10_000),
        }],
    }));

    let state = crate::app_state::AppState {
        registry,
        execution: Arc::new(crate::execution::ExecutionService::new()),
    };

    let response = router(state)
        .oneshot(post_request("git_fout", "{}"))
        .await
        .expect("de router moet een response geven");

    assert_eq!(response.status(), StatusCode::OK);

    let result = response_json(response).await;

    assert_eq!(result["success"], json!(false));
    assert!(
        result["exitCode"]
            .as_i64()
            .expect("de exitcode moet een geheel getal zijn")
            != 0
    );
    assert!(
        !result["stderr"]
            .as_str()
            .expect("stderr moet tekst zijn")
            .is_empty()
    );
}

#[cfg(any(windows, unix))]
#[tokio::test]
async fn timeout_returns_http_504() {
    #[cfg(windows)]
    let (command, arguments) = (
        "ping",
        vec!["-n".to_owned(), "30".to_owned(), "127.0.0.1".to_owned()],
    );

    #[cfg(unix)]
    let (command, arguments) = ("sleep", vec!["30".to_owned()]);

    let registry = Arc::new(ToolRegistry::new(ConfigFile {
        tools: vec![ToolDefinition {
            name: "langzaam".to_owned(),
            description: "Test een timeout via HTTP".to_owned(),
            command: command.to_owned(),
            arguments,
            parameters: Vec::new(),
            timeout: Some(100),
        }],
    }));

    let state = crate::app_state::AppState {
        registry,
        execution: Arc::new(crate::execution::ExecutionService::new()),
    };

    let response = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        router(state).oneshot(post_request("langzaam", "{}")),
    )
    .await
    .expect("de HTTP-timeoutafhandeling moet tijdig afronden")
    .expect("de router moet een response geven");

    assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);

    let actual = response_json(response).await;

    assert_eq!(
        actual,
        json!({
            "code": "execution_timeout",
            "message": "Uitvoering verlopen"
        })
    );
}

#[tokio::test]
async fn output_limit_returns_http_502() {
    let testprogramma =
        std::env::current_exe().expect("het pad naar het testprogramma moet beschikbaar zijn");

    let registry = Arc::new(ToolRegistry::new(ConfigFile {
        tools: vec![ToolDefinition {
            name: "grote_output".to_owned(),
            description: "Test de outputlimiet via HTTP".to_owned(),
            command: testprogramma.to_string_lossy().into_owned(),
            arguments: vec![
                "--exact".to_owned(),
                "execution::tests::writes_output_above_limit".to_owned(),
                "--ignored".to_owned(),
                "--nocapture".to_owned(),
            ],
            parameters: Vec::new(),
            timeout: Some(10_000),
        }],
    }));

    let state = crate::app_state::AppState {
        registry,
        execution: Arc::new(crate::execution::ExecutionService::new()),
    };

    let response = tokio::time::timeout(
        std::time::Duration::from_secs(6),
        router(state).oneshot(post_request("grote_output", "{}")),
    )
    .await
    .expect("de outputlimietafhandeling moet tijdig afronden")
    .expect("de router moet een response geven");

    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);

    let actual = response_json(response).await;

    assert_eq!(
        actual,
        json!({
            "code": "output_limit_exceeded",
            "message": "Uitvoerlimiet overschreden"
        })
    );
}

#[tokio::test]
async fn process_start_failure_returns_safe_http_500() {
    let registry = Arc::new(ToolRegistry::new(ConfigFile {
        tools: vec![ToolDefinition {
            name: "startfout".to_owned(),
            description: "Test een mislukte processtart via HTTP".to_owned(),
            command: "\0".to_owned(),
            arguments: Vec::new(),
            parameters: Vec::new(),
            timeout: Some(10_000),
        }],
    }));

    let state = crate::app_state::AppState {
        registry,
        execution: Arc::new(crate::execution::ExecutionService::new()),
    };

    let response = router(state)
        .oneshot(post_request("startfout", "{}"))
        .await
        .expect("de router moet een response geven");

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let actual = response_json(response).await;

    assert_eq!(
        actual,
        json!({
            "code": "execution_failed",
            "message": "Uitvoering kon niet worden voltooid"
        })
    );
}

#[tokio::test]
async fn rejects_tool_with_parameters_via_http() {
    let state = crate::app_state::AppState {
        registry: test_registry(),
        execution: Arc::new(crate::execution::ExecutionService::new()),
    };

    let response = router(state)
        .oneshot(post_request("list_files", "{}"))
        .await
        .expect("de router moet een response geven");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let actual = response_json(response).await;

    assert_eq!(
        actual,
        json!({
            "code": "parameters_not_supported",
            "message": "Deze tool ondersteunt nog geen parameters"
        })
    );
}

#[tokio::test]
async fn rejects_http_execution_after_shutdown_begins() {
    let registry = Arc::new(ToolRegistry::new(ConfigFile {
        tools: vec![ToolDefinition {
            name: "git_versie".to_owned(),
            description: "Toon de Git-versie".to_owned(),
            command: "git".to_owned(),
            arguments: vec!["--version".to_owned()],
            parameters: Vec::new(),
            timeout: Some(10_000),
        }],
    }));

    let execution = Arc::new(crate::execution::ExecutionService::new());

    execution.begin_shutdown();

    let state = crate::app_state::AppState {
        registry,
        execution,
    };

    let response = router(state)
        .oneshot(post_request("git_versie", "{}"))
        .await
        .expect("de router moet een response geven");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);

    let actual = response_json(response).await;

    assert_eq!(
        actual,
        json!({
            "code": "execution_cancelled",
            "message": "Uitvoering geannuleerd"
        })
    );
}
