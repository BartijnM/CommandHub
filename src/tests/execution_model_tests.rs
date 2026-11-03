use std::time::Duration;

use serde_json::json;

use super::ExecutionResponse;
use crate::execution::ExecutionResult;

#[test]
fn converts_core_result_to_http_json() {
    let result = ExecutionResult {
        success: false,
        stdout: "uitvoer".to_owned(),
        stderr: "foutuitvoer".to_owned(),
        exit_code: None,
        duration: Duration::from_millis(125),
    };

    let response = ExecutionResponse::from(result);
    let actual = serde_json::to_value(response)
        .expect("het uitvoeringsresultaat moet naar JSON omgezet kunnen worden");

    assert_eq!(
        actual,
        json!({
            "success": false,
            "stdout": "uitvoer",
            "stderr": "foutuitvoer",
            "exitCode": null,
            "duration": 125
        })
    );
}
