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
fn accepts_empty_json_object() {
    assert!(validate_parameterless_input(&json!({})).is_ok());
}

#[test]
fn rejects_fields_and_other_json_types() {
    let inputs = [
        json!({"command": "ongewenst"}),
        json!({"argument": "waarde"}),
        json!([]),
        json!(null),
        json!("tekst"),
        json!(42),
        json!(true),
    ];

    for input in inputs {
        let (status, Json(error)) = validate_parameterless_input(&input)
            .expect_err("alleen een leeg JSON-object mag worden geaccepteerd");

        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "onverwachte status voor invoer: {input}"
        );
        assert_eq!(error.code, "invalid_input");
    }
}
