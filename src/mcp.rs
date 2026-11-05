use crate::app_state::AppState;

#[derive(Clone)]
pub struct McpServer {
    state: AppState,
}

impl McpServer {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
}

impl rmcp::ServerHandler for McpServer {
    fn get_info(&self) -> rmcp::model::ServerConfig {
        let mut config = rmcp::model::ServerConfig::default();

        config.capabilities = rmcp::model::ServerCapabilities::builder()
            .enable_tools()
            .build();

        config
    }

    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, rmcp::ErrorData> {
        let tools = self
            .state
            .registry
            .tools()
            .iter()
            .map(|tool| {
                let schema = crate::registry::input_schema(tool);
                let properties = schema
                    .as_object()
                    .expect("input_schema levert altijd een JSON-object")
                    .clone();

                rmcp::model::Tool::new(tool.name.clone(), tool.description.clone(), properties)
            })
            .collect::<Vec<_>>();

        Ok(rmcp::model::ListToolsResult::with_all_items(tools))
    }

    async fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::CallToolResponse, rmcp::ErrorData> {
        let arguments = request.arguments.unwrap_or_default();

        let result = self
            .state
            .execution
            .execute_with_parameters(&self.state.registry, request.name.as_ref(), &arguments)
            .await;

        let response = match result {
            Ok(result) => execution_result(result),
            Err(error) => execution_error(error)?,
        };

        Ok(response.into())
    }
}

fn execution_result(result: crate::execution::ExecutionResult) -> rmcp::model::CallToolResult {
    let is_error = !result.success;

    let output = serde_json::json!({
        "success": result.success,
        "stdout": result.stdout,
        "stderr": result.stderr,
        "exitCode": result.exit_code,
        "duration": result.duration.as_millis()
    });

    let content = vec![rmcp::model::ContentBlock::text(output.to_string())];

    let mut response = if is_error {
        rmcp::model::CallToolResult::error(content)
    } else {
        rmcp::model::CallToolResult::success(content)
    };

    response.structured_content = Some(output);

    response
}

fn execution_error(
    error: crate::execution::ExecutionError,
) -> Result<rmcp::model::CallToolResult, rmcp::ErrorData> {
    use crate::execution::ExecutionError;

    let (code, message) = match error {
        ExecutionError::UnknownTool => {
            return Err(rmcp::ErrorData::new(
                rmcp::model::ErrorCode::METHOD_NOT_FOUND,
                "Onbekende tool",
                None,
            ));
        }
        ExecutionError::InvalidParameters(message) => ("invalid_parameters", message),
        ExecutionError::CapacityExceeded => ("capacity_exceeded", "Uitvoercapaciteit bereikt"),
        ExecutionError::TimedOut => ("execution_timeout", "Uitvoering verlopen"),
        ExecutionError::OutputLimitExceeded => {
            ("output_limit_exceeded", "Uitvoerlimiet overschreden")
        }
        ExecutionError::Cancelled => ("execution_cancelled", "Uitvoering geannuleerd"),
        ExecutionError::ProcessStartFailed(_)
        | ExecutionError::OutputReadFailed(_)
        | ExecutionError::ProcessWaitFailed(_)
        | ExecutionError::ProcessKillFailed(_) => {
            ("execution_failed", "Uitvoering kon niet worden voltooid")
        }
    };

    let output = serde_json::json!({
        "code": code,
        "message": message
    });

    let mut response = rmcp::model::CallToolResult::error(vec![rmcp::model::ContentBlock::text(
        output.to_string(),
    )]);

    response.structured_content = Some(output);

    Ok(response)
}

pub fn service(
    state: AppState,
) -> rmcp::transport::streamable_http_server::tower::StreamableHttpService<
    McpServer,
    rmcp::transport::streamable_http_server::session::local::LocalSessionManager,
> {
    use std::sync::Arc;

    use rmcp::transport::streamable_http_server::{
        session::local::LocalSessionManager,
        tower::StreamableHttpService,
    };

    StreamableHttpService::new(
        move || Ok(McpServer::new(state.clone())),
        Arc::new(LocalSessionManager::default()),
        transport_config(),
    )
}

fn transport_config()
-> rmcp::transport::streamable_http_server::StreamableHttpServerConfig {
    use rmcp::transport::streamable_http_server::StreamableHttpServerConfig;

    let mut config = StreamableHttpServerConfig::default()
        .with_allowed_hosts(["127.0.0.1:3000", "localhost:3000"]);

    config.allowed_origins = vec![
        "http://127.0.0.1:3000".to_owned(),
        "http://localhost:3000".to_owned(),
    ];

    config.max_request_body_bytes = 16 * 1024;
    config.legacy_session_mode = true;

    config
}

#[cfg(test)]
#[path = "tests/mcp_tests.rs"]
mod tests;