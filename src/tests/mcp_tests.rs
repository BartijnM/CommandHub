use super::{ execution_error, execution_result };
use crate::execution::{ ExecutionError, ExecutionResult };
use serde_json::json;
use std::{ io, time::Duration };

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

        let expected =
            json!({
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
        (ExecutionError::CapacityExceeded, "capacity_exceeded", "Uitvoercapaciteit bereikt"),
        (ExecutionError::TimedOut, "execution_timeout", "Uitvoering verlopen"),
        (
            ExecutionError::OutputLimitExceeded,
            "output_limit_exceeded",
            "Uitvoerlimiet overschreden",
        ),
        (ExecutionError::Cancelled, "execution_cancelled", "Uitvoering geannuleerd"),
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
        assert_eq!(response.structured_content, Some(json!({ "code": code, "message": message })));

        let serialized = serde_json::to_value(&response).unwrap();
        let text = serialized["content"][0]["text"].as_str().unwrap();
        let text_output: serde_json::Value = serde_json::from_str(text).unwrap();

        assert_eq!(text_output, json!({ "code": code, "message": message }));
        assert!(!serialized.to_string().contains(secret));
    }
}

fn transport_router() -> axum::Router {
    use std::sync::Arc;

    let state = crate::app_state::AppState {
        registry: Arc::new(
            crate::registry::ToolRegistry::new(crate::config::ConfigFile { tools: Vec::new() })
        ),
        execution: Arc::new(crate::execution::ExecutionService::new()),
    };

    crate::routes::router(state)
}

fn initialize_request(
    host: &str,
    origin: Option<&str>,
    body_size: Option<usize>
) -> axum::http::Request<axum::body::Body> {
    let mut body =
        json!({
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
    }).to_string();

    if let Some(size) = body_size {
        assert!(size >= body.len());
        body.push_str(&" ".repeat(size - body.len()));
    }

    let mut request = axum::http::Request
        ::builder()
        .method("POST")
        .uri("/mcp")
        .header("host", host)
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream");

    if let Some(origin) = origin {
        request = request.header("origin", origin);
    }

    request.body(axum::body::Body::from(body)).unwrap()
}

#[tokio::test]
async fn transport_rejects_disallowed_hosts() {
    use tower::ServiceExt;

    for host in ["example.com:3000", "localhost:4000"] {
        let response = transport_router()
            .oneshot(initialize_request(host, None, None)).await
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

    for origin in ["https://example.com", "http://localhost:4000", "null"] {
        let response = transport_router()
            .oneshot(initialize_request("127.0.0.1:3000", Some(origin), None)).await
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
        .oneshot(initialize_request("127.0.0.1:3000", None, Some(16 * 1024 + 1))).await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::PAYLOAD_TOO_LARGE);
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
            .oneshot(initialize_request(host, origin, None)).await
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
        .oneshot(initialize_request("127.0.0.1:3000", None, Some(16 * 1024))).await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    assert!(response.headers().contains_key("mcp-session-id"));
}

#[tokio::test]
async fn transport_cancellation_closes_active_mcp_stream() {
    use std::sync::Arc;

    use axum::{ body::{ Body, to_bytes }, http::{ Request, StatusCode } };
    use tokio_util::sync::CancellationToken;
    use tower::ServiceExt;

    let state = crate::app_state::AppState {
        registry: Arc::new(
            crate::registry::ToolRegistry::new(crate::config::ConfigFile { tools: Vec::new() })
        ),
        execution: Arc::new(crate::execution::ExecutionService::new()),
    };

    let shutdown = CancellationToken::new();
    let app = crate::routes::router_with_shutdown(state, shutdown.clone());

    let initialized = app
        .clone()
        .oneshot(initialize_request("127.0.0.1:3000", None, None)).await
        .unwrap();

    assert_eq!(initialized.status(), StatusCode::OK);

    let session_id = initialized
        .headers()
        .get("mcp-session-id")
        .expect("initialisatie moet een sessie opleveren")
        .clone();

    // Lees de initialisatieresponse volledig voordat de sessie verdergaat.
    tokio::time
        ::timeout(Duration::from_secs(2), to_bytes(initialized.into_body(), 64 * 1024)).await
        .expect("initialisatieresponse moet tijdig afronden")
        .expect("initialisatieresponse moet leesbaar zijn");

    let notification = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("host", "127.0.0.1:3000")
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream")
        .header("mcp-session-id", session_id.clone())
        .header("mcp-protocol-version", "2025-11-25")
        .body(
            Body::from(
                json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized"
            }).to_string()
            )
        )
        .unwrap();

    let response = app.clone().oneshot(notification).await.unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let request = Request::builder()
        .method("GET")
        .uri("/mcp")
        .header("host", "127.0.0.1:3000")
        .header("accept", "text/event-stream")
        .header("mcp-session-id", session_id)
        .header("mcp-protocol-version", "2025-11-25")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let reading = to_bytes(response.into_body(), 64 * 1024);
    tokio::pin!(reading);

    assert!(
        tokio::time::timeout(Duration::from_millis(50), &mut reading).await.is_err(),
        "de stream moet vóór annulering open blijven"
    );

    shutdown.cancel();

    tokio::time
        ::timeout(Duration::from_secs(2), &mut reading).await
        .expect("de MCP-stream moet na annulering sluiten")
        .expect("de MCP-stream moet zonder bodyfout sluiten");
}

#[test]
#[ignore = "Wordt uitsluitend als geconfigureerd childproces gestart"]
fn mcp_echo_helper() {
    println!("COMMANDHUB_MCP_TEST_OK");
}

#[tokio::test]
async fn mcp_client_discovers_and_executes_configured_tool() {
    use std::sync::Arc;

    use rmcp::{
        ServiceExt,
        model::CallToolRequestParams,
        transport::StreamableHttpClientTransport,
    };
    use tokio_util::sync::CancellationToken;

    let config = crate::config::ConfigFile {
        tools: vec![crate::config::ToolDefinition {
            name: "test_echo".to_owned(),
            description: "Gecontroleerde MCP-testtool".to_owned(),
            command: std::env::current_exe().unwrap().into_os_string().into_string().unwrap(),
            arguments: vec![
                "--exact".to_owned(),
                "mcp::tests::mcp_echo_helper".to_owned(),
                "--ignored".to_owned(),
                "--nocapture".to_owned()
            ],
            parameters: Vec::new(),
            timeout: Some(5_000),
        }],
    };

    config.validate().unwrap();

    let state = crate::app_state::AppState {
        registry: Arc::new(crate::registry::ToolRegistry::new(config)),
        execution: Arc::new(crate::execution::ExecutionService::new()),
    };

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();

    let shutdown = CancellationToken::new();
    let _shutdown_guard = shutdown.clone().drop_guard();

    let mut service = super::service(state.clone(), shutdown.clone());

    // Alleen deze test luistert op een door het OS gekozen vrije poort.
    service.config.allowed_hosts = vec![address.to_string()];

    let app = axum::Router::new().route_service("/mcp", service);
    let server_shutdown = shutdown.clone();

    let server = tokio::spawn(async move {
        axum::serve(listener, app).with_graceful_shutdown(server_shutdown.cancelled_owned()).await
    });

    let outcome = tokio::time::timeout(Duration::from_secs(15), async {
        let transport = StreamableHttpClientTransport::from_uri(format!("http://{address}/mcp"));

        let client = ().serve(transport).await.unwrap();

        let tools = client.list_all_tools().await.unwrap();

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name.as_ref(), "test_echo");
        assert_eq!(tools[0].description.as_deref(), Some("Gecontroleerde MCP-testtool"));
        assert_eq!(
            tools[0].input_schema.as_ref(),
            crate::registry
                ::input_schema(state.registry.find("test_echo").unwrap())
                .as_object()
                .unwrap()
        );

        let response = client.call_tool(CallToolRequestParams::new("test_echo")).await.unwrap();

        assert_eq!(response.is_error, Some(false));

        let output = response.structured_content.unwrap();
        assert_eq!(output["success"], true);
        assert_eq!(output["exitCode"], 0);
        assert!(output["stdout"].as_str().unwrap().contains("COMMANDHUB_MCP_TEST_OK"));
        let invalid_arguments = json!({ "extra": "niet toegestaan" }).as_object().unwrap().clone();

        let rejected = client
            .call_tool(
                CallToolRequestParams::new("test_echo").with_arguments(invalid_arguments)
            ).await
            .expect("ongeldige toolinvoer moet een toolresultaat opleveren");

        assert_eq!(rejected.is_error, Some(true));

        let error_output = rejected.structured_content.unwrap();
        assert_eq!(error_output["code"], "invalid_parameters");
        assert!(
            error_output.get("stdout").is_none(),
            "een validatiefout moet geen procesresultaat opleveren"
        );

        let unknown = client.call_tool(CallToolRequestParams::new("bestaat_niet")).await;

        assert!(unknown.is_err(), "een onbekende tool moet een protocolfout opleveren");

        // Na afgewezen requests moet de verbinding bruikbaar blijven.
        let recovered = client
            .call_tool(CallToolRequestParams::new("test_echo")).await
            .expect("een geldige vervolgaanroep moet slagen");

        assert_eq!(recovered.is_error, Some(false));
        assert_eq!(recovered.structured_content.unwrap()["exitCode"], 0);

        client.cancel().await.unwrap();
    }).await;

    shutdown.cancel();
    state.execution.shutdown().await;

    tokio::time
        ::timeout(Duration::from_secs(5), server).await
        .expect("testserver moet tijdig stoppen")
        .expect("servertaak mag niet panikeren")
        .expect("testserver moet zonder IO-fout stoppen");

    outcome.expect("MCP-ketentest moet binnen 15 seconden afronden");
}

#[test]
#[ignore = "Wordt uitsluitend als geconfigureerd childproces gestart"]
fn mcp_wait_helper() {
    let arguments: Vec<String> = std::env::args().collect();

    let pid_file = arguments
        .iter()
        .find_map(|argument| {
            argument.strip_prefix("commandhub-mcp-pid-file=")
        })
        .expect("de helper moet een PID-bestand ontvangen");

    std::fs::write(pid_file, std::process::id().to_string())
        .expect("de helper moet zijn PID kunnen opslaan");

    // Vangnet: ook bij een mislukte test blijft dit proces niet eindeloos draaien.
    std::thread::sleep(Duration::from_secs(30));
}
