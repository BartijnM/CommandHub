use super::{execution_error, execution_result};
use crate::execution::{ExecutionError, ExecutionResult};
use serde_json::json;
use std::{io, time::Duration};

#[test]
fn process_results_preserve_fields_and_indicate_failure() {
    for success in [true, false] {
        let response = execution_result(ExecutionResult {
            success,
            stdout: "testuitvoer".to_owned(),
            stderr: "testmelding".to_owned(),
            exit_code: Some(if success { 0 } else { 1 }),
            duration: Duration::from_millis(42),
        });

        let expected = json!({
            "success": success,
            "stdout": "testuitvoer",
            "stderr": "testmelding",
            "exitCode": if success { 0 } else { 1 },
            "duration": 42
        });

        assert_eq!(response.is_error, Some(!success));
        assert_eq!(response.structured_content, Some(expected.clone()));

        let serialized = serde_json::to_value(&response).unwrap();
        assert_eq!(serialized["content"][0]["type"], "text");

        let text = serialized["content"][0]["text"].as_str().unwrap();
        let text_output: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(text_output, expected);
    }
}

#[test]
fn missing_exit_code_is_returned_as_null() {
    let response = execution_result(ExecutionResult {
        success: false,
        stdout: String::new(),
        stderr: String::new(),
        exit_code: None,
        duration: Duration::ZERO,
    });

    let output = response.structured_content.unwrap();

    assert!(output.as_object().unwrap().contains_key("exitCode"));
    assert_eq!(output["exitCode"], serde_json::Value::Null);
    assert_eq!(response.is_error, Some(true));
}

#[test]
fn unknown_tool_returns_protocol_error() {
    let error = execution_error(ExecutionError::UnknownTool).unwrap_err();

    assert_eq!(error.code, rmcp::model::ErrorCode::METHOD_NOT_FOUND);
    assert_eq!(error.message.as_ref(), "Onbekende tool");
    assert!(error.data.is_none());
}

#[test]
fn execution_errors_return_safe_tool_results() {
    let secret = "gevoelige OS-details die niet naar de client mogen";

    let cases = [
        (
            ExecutionError::InvalidParameters("Verplichte parameter ontbreekt"),
            "invalid_parameters",
            "Verplichte parameter ontbreekt",
        ),
        (
            ExecutionError::CapacityExceeded,
            "capacity_exceeded",
            "Uitvoercapaciteit bereikt",
        ),
        (
            ExecutionError::TimedOut,
            "execution_timeout",
            "Uitvoering verlopen",
        ),
        (
            ExecutionError::OutputLimitExceeded,
            "output_limit_exceeded",
            "Uitvoerlimiet overschreden",
        ),
        (
            ExecutionError::Cancelled,
            "execution_cancelled",
            "Uitvoering geannuleerd",
        ),
        (
            ExecutionError::ProcessStartFailed(io::Error::other(secret)),
            "execution_failed",
            "Uitvoering kon niet worden voltooid",
        ),
        (
            ExecutionError::OutputReadFailed(io::Error::other(secret)),
            "execution_failed",
            "Uitvoering kon niet worden voltooid",
        ),
        (
            ExecutionError::ProcessWaitFailed(io::Error::other(secret)),
            "execution_failed",
            "Uitvoering kon niet worden voltooid",
        ),
        (
            ExecutionError::ProcessKillFailed(io::Error::other(secret)),
            "execution_failed",
            "Uitvoering kon niet worden voltooid",
        ),
    ];

    for (error, code, message) in cases {
        let response = execution_error(error).unwrap();

        assert_eq!(response.is_error, Some(true));
        assert_eq!(
            response.structured_content,
            Some(json!({ "code": code, "message": message }))
        );

        let serialized = serde_json::to_value(&response).unwrap();
        let text = serialized["content"][0]["text"].as_str().unwrap();
        let text_output: serde_json::Value = serde_json::from_str(text).unwrap();

        assert_eq!(
            text_output,
            json!({ "code": code, "message": message })
        );
        assert!(!serialized.to_string().contains(secret));
    }
}

fn transport_router() -> axum::Router {
    use std::sync::Arc;

    let state = crate::app_state::AppState {
        registry: Arc::new(crate::registry::ToolRegistry::new(
            crate::config::ConfigFile { tools: Vec::new() },
        )),
        execution: Arc::new(crate::execution::ExecutionService::new()),
    };

    crate::routes::router(state)
}

fn initialize_request(
    host: &str,
    origin: Option<&str>,
    body_size: Option<usize>,
) -> axum::http::Request<axum::body::Body> {
    let mut body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {
                "name": "commandhub-test",
                "version": "1.0.0"
            }
        }
    })
    .to_string();

    if let Some(size) = body_size {
        assert!(size >= body.len());
        body.push_str(&" ".repeat(size - body.len()));
    }

    let mut request = axum::http::Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("host", host)
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream");

    if let Some(origin) = origin {
        request = request.header("origin", origin);
    }

    request
        .body(axum::body::Body::from(body))
        .unwrap()
}

#[tokio::test]
async fn transport_rejects_disallowed_hosts() {
    use tower::ServiceExt;

    for host in ["example.com:3000", "localhost:4000"] {
        let response = transport_router()
            .oneshot(initialize_request(host, None, None))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            axum::http::StatusCode::FORBIDDEN,
            "Host {host} moet worden afgewezen"
        );
    }
}

#[tokio::test]
async fn transport_rejects_disallowed_origins() {
    use tower::ServiceExt;

    for origin in [
        "https://example.com",
        "http://localhost:4000",
        "null",
    ] {
        let response = transport_router()
            .oneshot(initialize_request(
                "127.0.0.1:3000",
                Some(origin),
                None,
            ))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            axum::http::StatusCode::FORBIDDEN,
            "Origin {origin} moet worden afgewezen"
        );
    }
}

#[tokio::test]
async fn transport_rejects_oversized_body() {
    use tower::ServiceExt;

    let response = transport_router()
        .oneshot(initialize_request(
            "127.0.0.1:3000",
            None,
            Some(16 * 1024 + 1),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::PAYLOAD_TOO_LARGE
    );
}

#[tokio::test]
async fn transport_accepts_allowed_hosts_and_origins() {
    use tower::ServiceExt;

    for (host, origin) in [
        ("127.0.0.1:3000", None),
        ("localhost:3000", None),
        ("127.0.0.1:3000", Some("http://127.0.0.1:3000")),
        ("localhost:3000", Some("http://localhost:3000")),
    ] {
        let response = transport_router()
            .oneshot(initialize_request(host, origin, None))
            .await
            .unwrap();

        assert_eq!(
            response.status(),
            axum::http::StatusCode::OK,
            "Host {host} met Origin {origin:?} moet worden toegelaten"
        );

        assert!(
            response.headers().contains_key("mcp-session-id"),
            "Initialisatie met protocol 2025-11-25 moet een sessie opleveren"
        );
    }
}

#[tokio::test]
async fn transport_accepts_body_of_exactly_16_kib() {
    use tower::ServiceExt;

    let response = transport_router()
        .oneshot(initialize_request(
            "127.0.0.1:3000",
            None,
            Some(16 * 1024),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::OK
    );

    assert!(response.headers().contains_key("mcp-session-id"));
}