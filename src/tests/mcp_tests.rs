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

        assert_eq!(text_output, json!({ "code": code, "message": message }));
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

    request.body(axum::body::Body::from(body)).unwrap()
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

    for origin in ["https://example.com", "http://localhost:4000", "null"] {
        let response = transport_router()
            .oneshot(initialize_request("127.0.0.1:3000", Some(origin), None))
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
        .oneshot(initialize_request("127.0.0.1:3000", None, Some(16 * 1024)))
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);

    assert!(response.headers().contains_key("mcp-session-id"));
}

#[tokio::test]
async fn transport_cancellation_closes_active_mcp_stream() {
    use std::sync::Arc;

    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use tokio_util::sync::CancellationToken;
    use tower::ServiceExt;

    let state = crate::app_state::AppState {
        registry: Arc::new(crate::registry::ToolRegistry::new(
            crate::config::ConfigFile { tools: Vec::new() },
        )),
        execution: Arc::new(crate::execution::ExecutionService::new()),
    };

    let shutdown = CancellationToken::new();
    let app = crate::routes::router_with_shutdown(state, shutdown.clone());

    let initialized = app
        .clone()
        .oneshot(initialize_request("127.0.0.1:3000", None, None))
        .await
        .unwrap();

    assert_eq!(initialized.status(), StatusCode::OK);

    let session_id = initialized
        .headers()
        .get("mcp-session-id")
        .expect("initialisatie moet een sessie opleveren")
        .clone();

    // Lees de initialisatieresponse volledig voordat de sessie verdergaat.
    tokio::time::timeout(
        Duration::from_secs(2),
        to_bytes(initialized.into_body(), 64 * 1024),
    )
    .await
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
        .body(Body::from(
            json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized"
            })
            .to_string(),
        ))
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
        tokio::time::timeout(Duration::from_millis(50), &mut reading)
            .await
            .is_err(),
        "de stream moet vóór annulering open blijven"
    );

    shutdown.cancel();

    tokio::time::timeout(Duration::from_secs(2), &mut reading)
        .await
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
        ServiceExt, model::CallToolRequestParams, transport::StreamableHttpClientTransport,
    };
    use tokio_util::sync::CancellationToken;

    let project = std::env::current_dir().unwrap();
    let allowed_root = project.join("src");

    let config = crate::config::ConfigFile {
        tools: vec![
            crate::config::ToolDefinition {
                name: "test_echo".to_owned(),
                description: "Gecontroleerde MCP-testtool".to_owned(),
                command: std::env::current_exe()
                    .unwrap()
                    .into_os_string()
                    .into_string()
                    .unwrap(),
                arguments: vec![
                    "--exact".to_owned(),
                    "mcp::tests::mcp_echo_helper".to_owned(),
                    "--ignored".to_owned(),
                    "--nocapture".to_owned(),
                ],
                parameters: Vec::new(),
                timeout: Some(5_000),
            },
            crate::config::ToolDefinition {
                name: "test_version".to_owned(),
                description: "Rust-versie met gecontroleerde parameter".to_owned(),
                command: "rustc".to_owned(),
                arguments: vec!["{detail}".to_owned()],
                parameters: vec![crate::config::ParameterDefinition {
                    name: "detail".to_owned(),
                    kind: crate::config::ParameterType::String,
                    required: true,
                    allowed_values: Some(vec!["--version".to_owned(), "-vV".to_owned()]),
                    allowed_roots: None,
                }],
                timeout: Some(5_000),
            },
            crate::config::ToolDefinition {
                name: "test_path".to_owned(),
                description: "Repository opzoeken binnen de toegestane root".to_owned(),
                command: "git".to_owned(),
                arguments: vec![
                    "-C".to_owned(),
                    "{path}".to_owned(),
                    "rev-parse".to_owned(),
                    "--show-toplevel".to_owned(),
                ],
                parameters: vec![crate::config::ParameterDefinition {
                    name: "path".to_owned(),
                    kind: crate::config::ParameterType::String,
                    required: true,
                    allowed_values: None,
                    allowed_roots: Some(vec![allowed_root.clone()]),
                }],
                timeout: Some(5_000),
            },
        ],
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
        axum::serve(listener, app)
            .with_graceful_shutdown(server_shutdown.cancelled_owned())
            .await
    });

    let outcome = tokio::time::timeout(Duration::from_secs(15), async {
        let transport = StreamableHttpClientTransport::from_uri(format!("http://{address}/mcp"));

        let client = ().serve(transport).await.unwrap();

        let server_info = client
            .peer_info()
            .expect("MCP-onderhandeling moet serverinformatie opleveren");

        println!(
            "Onderhandelde MCP-protocolversie: {}",
            server_info.protocol_version.as_str()
        );

        let tools = client.list_all_tools().await.unwrap();

        assert_eq!(tools.len(), 3);
        assert_eq!(tools[0].name.as_ref(), "test_echo");
        assert_eq!(
            tools[0].description.as_deref(),
            Some("Gecontroleerde MCP-testtool")
        );
        assert_eq!(
            tools[0].input_schema.as_ref(),
            crate::registry::input_schema(state.registry.find("test_echo").unwrap())
                .as_object()
                .unwrap()
        );

        let response = client
            .call_tool(CallToolRequestParams::new("test_echo"))
            .await
            .unwrap();

        assert_eq!(response.is_error, Some(false));

        let output = response.structured_content.unwrap();
        assert_eq!(output["success"], true);
        assert_eq!(output["exitCode"], 0);
        assert!(
            output["stdout"]
                .as_str()
                .unwrap()
                .contains("COMMANDHUB_MCP_TEST_OK")
        );
        let invalid_arguments = json!({ "extra": "niet toegestaan" })
            .as_object()
            .unwrap()
            .clone();

        let rejected = client
            .call_tool(CallToolRequestParams::new("test_echo").with_arguments(invalid_arguments))
            .await
            .expect("ongeldige toolinvoer moet een toolresultaat opleveren");

        assert_eq!(rejected.is_error, Some(true));

        let error_output = rejected.structured_content.unwrap();
        assert_eq!(error_output["code"], "invalid_parameters");
        assert!(
            error_output.get("stdout").is_none(),
            "een validatiefout moet geen procesresultaat opleveren"
        );

        let unknown = client
            .call_tool(CallToolRequestParams::new("bestaat_niet"))
            .await;

        assert!(
            unknown.is_err(),
            "een onbekende tool moet een protocolfout opleveren"
        );

        // Na afgewezen requests moet de verbinding bruikbaar blijven.
        let recovered = client
            .call_tool(CallToolRequestParams::new("test_echo"))
            .await
            .expect("een geldige vervolgaanroep moet slagen");

        assert_eq!(recovered.is_error, Some(false));
        assert_eq!(recovered.structured_content.unwrap()["exitCode"], 0);

        {
            use tower::ServiceExt as TowerServiceExt;

            for (input, expected_error) in [
                (json!({ "detail": "-vV" }), None),
                (json!({}), Some("Verplichte parameter ontbreekt")),
                (
                    json!({ "detail": 42 }),
                    Some("Parameter moet een string zijn"),
                ),
                (
                    json!({ "detail": "--help" }),
                    Some("Parameterwaarde is niet toegestaan"),
                ),
                (
                    json!({ "detail": "-vV", "extra": true }),
                    Some("Onbekende parameter"),
                ),
            ] {
                let mcp_response = client
                    .call_tool(
                        CallToolRequestParams::new("test_version")
                            .with_arguments(input.as_object().unwrap().clone()),
                    )
                    .await
                    .expect("MCP-call moet een toolresultaat opleveren");

                let rest_request = axum::http::Request::builder()
                    .method("POST")
                    .uri("/tools/test_version")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(input.to_string()))
                    .unwrap();

                let rest_response = crate::routes::router(state.clone())
                    .oneshot(rest_request)
                    .await
                    .unwrap();

                let rest_status = rest_response.status();
                let body = axum::body::to_bytes(rest_response.into_body(), 64 * 1024)
                    .await
                    .unwrap();

                let rest_output: serde_json::Value = serde_json::from_slice(&body).unwrap();
                let mcp_output = mcp_response.structured_content.unwrap();

                if let Some(message) = expected_error {
                    assert_eq!(mcp_response.is_error, Some(true));
                    assert_eq!(rest_status, axum::http::StatusCode::BAD_REQUEST);

                    let expected = json!({
                        "code": "invalid_parameters",
                        "message": message
                    });

                    assert_eq!(mcp_output, expected);
                    assert_eq!(rest_output, expected);
                } else {
                    assert_eq!(mcp_response.is_error, Some(false));
                    assert_eq!(rest_status, axum::http::StatusCode::OK);

                    assert_eq!(mcp_output["success"], true);
                    assert_eq!(mcp_output["exitCode"], 0);
                    assert!(mcp_output["stdout"].as_str().unwrap().starts_with("rustc "));

                    for field in ["success", "stdout", "stderr", "exitCode"] {
                        assert_eq!(
                            mcp_output[field], rest_output[field],
                            "REST en MCP verschillen voor veld {field}"
                        );
                    }

                    assert!(mcp_output["duration"].is_u64());
                    assert!(rest_output["duration"].is_u64());
                }
            }
        }

        {
            use tower::ServiceExt as TowerServiceExt;

            let cases = [
                (allowed_root.clone(), None),
                (allowed_root.join("tests").join(".."), None),
                (project.clone(), Some("Pad ligt buiten toegestane roots")),
                (
                    allowed_root.join(".."),
                    Some("Pad ligt buiten toegestane roots"),
                ),
                (
                    std::path::PathBuf::from("src"),
                    Some("Pad moet absoluut zijn"),
                ),
                (
                    project.join("Cargo.toml").join("cannot-exist"),
                    Some("Pad kan niet worden opgelost"),
                ),
            ];

            for (path, expected_error) in cases {
                let input = json!({ "path": path });

                let mcp_response = client
                    .call_tool(
                        CallToolRequestParams::new("test_path")
                            .with_arguments(input.as_object().unwrap().clone()),
                    )
                    .await
                    .expect("MCP-padcall moet een toolresultaat opleveren");

                let request = axum::http::Request::builder()
                    .method("POST")
                    .uri("/tools/test_path")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(input.to_string()))
                    .unwrap();

                let rest_response = crate::routes::router(state.clone())
                    .oneshot(request)
                    .await
                    .unwrap();

                let rest_status = rest_response.status();
                let body = axum::body::to_bytes(rest_response.into_body(), 64 * 1024)
                    .await
                    .unwrap();

                let rest_output: serde_json::Value = serde_json::from_slice(&body).unwrap();
                let mcp_output = mcp_response.structured_content.unwrap();

                if let Some(message) = expected_error {
                    let expected = json!({
                        "code": "invalid_parameters",
                        "message": message
                    });

                    assert_eq!(mcp_response.is_error, Some(true));
                    assert_eq!(rest_status, axum::http::StatusCode::BAD_REQUEST);
                    assert_eq!(mcp_output, expected);
                    assert_eq!(rest_output, expected);
                } else {
                    assert_eq!(mcp_response.is_error, Some(false));
                    assert_eq!(rest_status, axum::http::StatusCode::OK);
                    assert_eq!(mcp_output["success"], true);
                    assert_eq!(mcp_output["exitCode"], 0);

                    let reported_root =
                        std::path::Path::new(mcp_output["stdout"].as_str().unwrap().trim())
                            .canonicalize()
                            .unwrap();

                    assert_eq!(reported_root, project.canonicalize().unwrap());

                    for field in ["success", "stdout", "stderr", "exitCode"] {
                        assert_eq!(
                            mcp_output[field], rest_output[field],
                            "REST en MCP verschillen voor veld {field}"
                        );
                    }
                }
            }
        }

        client.cancel().await.unwrap();
    })
    .await;

    shutdown.cancel();
    state.execution.shutdown().await;

    tokio::time::timeout(Duration::from_secs(5), server)
        .await
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
        .find_map(|argument| argument.strip_prefix("commandhub-mcp-pid-file="))
        .expect("de helper moet een PID-bestand ontvangen");

    std::fs::write(pid_file, std::process::id().to_string())
        .expect("de helper moet zijn PID kunnen opslaan");

    // Vangnet: ook bij een mislukte test blijft dit proces niet eindeloos draaien.
    std::thread::sleep(Duration::from_secs(30));
}

async fn wait_for_mcp_pid(path: &std::path::Path) -> u32 {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match std::fs::read_to_string(path) {
                Ok(contents) => {
                    if let Ok(pid) = contents.trim().parse::<u32>()
                        && pid > 0
                    {
                        return pid;
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => panic!("PID-bestand lezen mislukt: {error}"),
            }

            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("MCP-helper moet binnen vijf seconden zijn PID schrijven")
}

#[tokio::test]
async fn protocol_cancellation_stops_process_and_preserves_capacity() {
    use std::sync::Arc;

    use rmcp::{
        ServiceExt,
        model::{CallToolRequest, CallToolRequestParams, ClientRequest},
        service::PeerRequestOptions,
        transport::StreamableHttpClientTransport,
    };
    use tokio_util::sync::CancellationToken;

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let pid_file = std::env::temp_dir().join(format!(
        "commandhub-mcp-cancel-{}-{unique}.pid",
        std::process::id()
    ));

    let executable = std::env::current_exe()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap();

    let mut tools = Vec::new();

    for (name, helper) in [
        ("test_wait", "mcp::tests::mcp_wait_helper"),
        ("test_echo", "mcp::tests::mcp_echo_helper"),
    ] {
        let mut arguments = vec![
            "--exact".to_owned(),
            helper.to_owned(),
            "--ignored".to_owned(),
            "--nocapture".to_owned(),
        ];

        if name == "test_wait" {
            arguments.extend([
                "--skip".to_owned(),
                format!("commandhub-mcp-pid-file={}", pid_file.display()),
            ]);
        }

        tools.push(crate::config::ToolDefinition {
            name: name.to_owned(),
            description: "Gecontroleerde annuleringstest".to_owned(),
            command: executable.clone(),
            arguments,
            parameters: Vec::new(),
            timeout: Some(30_000),
        });
    }

    let config = crate::config::ConfigFile { tools };
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
    service.config.allowed_hosts = vec![address.to_string()];

    let app = axum::Router::new().route_service("/mcp", service);
    let server_shutdown = shutdown.clone();

    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(server_shutdown.cancelled_owned())
            .await
    });

    let outcome = tokio::time::timeout(Duration::from_secs(25), async {
        let transport = StreamableHttpClientTransport::from_uri(format!("http://{address}/mcp"));

        let client = ().serve(transport).await.unwrap();

        // Vier rondes maken een achtergebleven capaciteitspermit zichtbaar.
        for _ in 0..4 {
            let request = ClientRequest::CallToolRequest(CallToolRequest::new(
                CallToolRequestParams::new("test_wait"),
            ));

            let handle = client
                .send_cancellable_request(request, PeerRequestOptions::no_options())
                .await
                .expect("langlopende toolcall moet verstuurd worden");

            let pid = wait_for_mcp_pid(&pid_file).await;

            handle
                .cancel(Some("Gecontroleerde testannulering".to_owned()))
                .await
                .expect("annuleringsbericht moet verstuurd worden");

            crate::process_checks::assert_stopped(pid).await;
            std::fs::remove_file(&pid_file).unwrap();

            let recovered = client
                .call_tool(CallToolRequestParams::new("test_echo"))
                .await
                .expect("vervolgaanroep moet na annulering werken");

            assert_eq!(recovered.is_error, Some(false));
            assert_eq!(recovered.structured_content.unwrap()["exitCode"], 0);
        }

        client.cancel().await.unwrap();
    })
    .await;

    shutdown.cancel();

    tokio::time::timeout(Duration::from_secs(5), state.execution.shutdown())
        .await
        .expect("uitvoeringsservice moet tijdig stoppen");

    tokio::time::timeout(Duration::from_secs(5), server)
        .await
        .expect("testserver moet tijdig stoppen")
        .expect("servertaak mag niet panikeren")
        .expect("testserver moet zonder IO-fout stoppen");

    outcome.expect("annuleringstest moet binnen 25 seconden afronden");
}

#[tokio::test]
async fn transport_rejects_invalid_json_rpc_bodies() {
    use tower::ServiceExt;

    for body in ["{", "null", "[]", "{}"] {
        let mut request = initialize_request("127.0.0.1:3000", None, None);
        *request.body_mut() = axum::body::Body::from(body);

        let response = transport_router().oneshot(request).await.unwrap();

        assert_eq!(
            response.status(),
            axum::http::StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "ongeldige JSON-RPC-body moet worden afgewezen: {body}"
        );
    }
}

#[tokio::test]
async fn transport_requires_supported_content_negotiation() {
    use axum::http::{HeaderValue, StatusCode, header};
    use tower::ServiceExt;

    for (name, value, expected_status) in [
        (
            header::ACCEPT,
            "application/json",
            StatusCode::NOT_ACCEPTABLE,
        ),
        (
            header::CONTENT_TYPE,
            "text/plain",
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
    ] {
        let mut request = initialize_request("127.0.0.1:3000", None, None);
        request
            .headers_mut()
            .insert(name, HeaderValue::from_static(value));

        let response = transport_router().oneshot(request).await.unwrap();

        assert_eq!(response.status(), expected_status);
    }
}

#[tokio::test]
async fn transport_rejects_unsupported_protocol_version_header() {
    use axum::http::{HeaderValue, StatusCode};
    use tower::ServiceExt;

    let mut request = initialize_request("127.0.0.1:3000", None, None);
    request.headers_mut().insert(
        "mcp-protocol-version",
        HeaderValue::from_static("1900-01-01"),
    );

    let response = transport_router().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn transport_rejects_unknown_session() {
    use axum::http::{HeaderValue, StatusCode};
    use tower::ServiceExt;

    let mut request = initialize_request("127.0.0.1:3000", None, None);
    request.headers_mut().insert(
        "mcp-session-id",
        HeaderValue::from_static("commandhub-unknown-session"),
    );

    let response = transport_router().oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
