use axum::{Json, http::StatusCode};

use crate::{execution::ExecutionError, models::execution::ExecutionErrorResponse};

pub async fn execute_tool(
    axum::extract::State(state): axum::extract::State<crate::app_state::AppState>,
    axum::extract::Path(tool_name): axum::extract::Path<String>,
    input: Result<Json<serde_json::Value>, axum::extract::rejection::JsonRejection>,
) -> Result<
    Json<crate::models::execution::ExecutionResponse>,
    (StatusCode, Json<ExecutionErrorResponse>),
> {
    let Json(input) = input.map_err(json_rejection_response)?;

    validate_parameterless_input(&input)?;

    let result = state
        .execution
        .execute_parameterless(&state.registry, &tool_name)
        .await
        .map_err(execution_error_response)?;

    Ok(Json(crate::models::execution::ExecutionResponse::from(
        result,
    )))
}

fn json_rejection_response(
    rejection: axum::extract::rejection::JsonRejection,
) -> (StatusCode, Json<ExecutionErrorResponse>) {
    if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(ExecutionErrorResponse {
                code: "request_too_large",
                message: "De requestbody mag maximaal 16 KiB groot zijn",
            }),
        );
    }

    (
        StatusCode::BAD_REQUEST,
        Json(ExecutionErrorResponse {
            code: "invalid_json",
            message: "Verwacht geldige JSON met Content-Type application/json",
        }),
    )
}

fn validate_parameterless_input(
    input: &serde_json::Value,
) -> Result<(), (StatusCode, Json<ExecutionErrorResponse>)> {
    match input {
        serde_json::Value::Object(fields) if fields.is_empty() => Ok(()),
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(ExecutionErrorResponse {
                code: "invalid_input",
                message: "Verwacht een leeg JSON-object voor een parameterloze tool",
            }),
        )),
    }
}

fn execution_error_response(error: ExecutionError) -> (StatusCode, Json<ExecutionErrorResponse>) {
    let (status, code, message) = match error {
        ExecutionError::UnknownTool => (StatusCode::NOT_FOUND, "unknown_tool", "Onbekende tool"),
        ExecutionError::ParametersNotSupported => (
            StatusCode::BAD_REQUEST,
            "parameters_not_supported",
            "Deze tool ondersteunt nog geen parameters",
        ),
        ExecutionError::CapacityExceeded => (
            StatusCode::SERVICE_UNAVAILABLE,
            "capacity_exceeded",
            "Uitvoercapaciteit bereikt",
        ),
        ExecutionError::TimedOut => (
            StatusCode::GATEWAY_TIMEOUT,
            "execution_timeout",
            "Uitvoering verlopen",
        ),
        ExecutionError::OutputLimitExceeded => (
            StatusCode::BAD_GATEWAY,
            "output_limit_exceeded",
            "Uitvoerlimiet overschreden",
        ),
        ExecutionError::Cancelled => (
            StatusCode::SERVICE_UNAVAILABLE,
            "execution_cancelled",
            "Uitvoering geannuleerd",
        ),
        ExecutionError::ProcessStartFailed(_)
        | ExecutionError::OutputReadFailed(_)
        | ExecutionError::ProcessWaitFailed(_)
        | ExecutionError::ProcessKillFailed(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "execution_failed",
            "Uitvoering kon niet worden voltooid",
        ),
    };

    (status, Json(ExecutionErrorResponse { code, message }))
}

#[cfg(test)]
#[path = "../tests/execution_controller_tests.rs"]
mod tests;
