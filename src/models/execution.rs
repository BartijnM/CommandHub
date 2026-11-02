use serde::Serialize;

use crate::execution::ExecutionResult;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionResponse {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration: u128,
}

#[derive(Serialize)]
pub struct ExecutionErrorResponse {
    pub code: &'static str,
    pub message: &'static str,
}

impl From<ExecutionResult> for ExecutionResponse {
    fn from(result: ExecutionResult) -> Self {
        Self {
            success: result.success,
            stdout: result.stdout,
            stderr: result.stderr,
            exit_code: result.exit_code,
            duration: result.duration.as_millis(),
        }
    }
}

#[cfg(test)]
#[path = "../tests/execution_model_tests.rs"]
mod tests;
