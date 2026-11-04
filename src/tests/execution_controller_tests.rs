use std::io;

use axum::{body::to_bytes, response::IntoResponse};
use serde_json::json;

use super::*;

#[tokio::test]
async fn unknown_tool_returns_404_with_json_error() {
    let response = execution_error_response(ExecutionError::UnknownTool).into_response();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = to_bytes(response.into_body(), 1024)
        .await
        .expect("de foutresponse moet leesbaar zijn");

    let actual: serde_json::Value =
        serde_json::from_slice(&body).expect("de foutresponse moet JSON zijn");

    assert_eq!(
        actual,
        json!({
            "code": "unknown_tool",
            "message": "Onbekende tool"
        })
    );
}

#[tokio::test]
async fn process_start_failure_does_not_leak_internal_details() {
    let error =
        ExecutionError::ProcessStartFailed(io::Error::other("intern-pad-en-geheime-details"));

    let response = execution_error_response(error).into_response();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = to_bytes(response.into_body(), 1024)
        .await
        .expect("de foutresponse moet leesbaar zijn");

    let actual: serde_json::Value =
        serde_json::from_slice(&body).expect("de foutresponse moet JSON zijn");

    assert_eq!(
        actual,
        json!({
            "code": "execution_failed",
            "message": "Uitvoering kon niet worden voltooid"
        })
    );
}

#[test]
fn input_object_accepts_empty_and_populated_objects() {
    for input in [
        json!({}),
        json!({ "message": "Hallo" }),
        json!({ "message": 42 }),
    ] {
        let fields = input_object(&input).expect("JSON-object moet worden geaccepteerd");

        assert_eq!(fields, input.as_object().unwrap());
    }
}

#[test]
fn input_object_rejects_other_json_types() {
    for input in [
        json!([]),
        json!(null),
        json!("tekst"),
        json!(42),
        json!(true),
    ] {
        let (status, Json(error)) = input_object(&input).expect_err("invoer moet een object zijn");

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(error.code, "invalid_input");
        assert_eq!(error.message, "Verwacht een JSON-object");
    }
}

#[tokio::test]
async fn invalid_parameters_return_400_with_json_error() {
    let response = execution_error_response(ExecutionError::InvalidParameters(
        "Parameterwaarde is niet toegestaan",
    ))
    .into_response();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), 1024)
        .await
        .expect("foutresponse moet leesbaar zijn");
    let actual: serde_json::Value =
        serde_json::from_slice(&body).expect("foutresponse moet JSON zijn");

    assert_eq!(
        actual,
        json!({
            "code": "invalid_parameters",
            "message": "Parameterwaarde is niet toegestaan"
        })
    );
}
