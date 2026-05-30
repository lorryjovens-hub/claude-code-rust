//! Tool execution endpoints.

use crate::api::error::ApiError;
use crate::api::state::AppState;
use crate::bridge::ToolRequest;
use crate::tools::{execute_tool, get_tool_definitions, ToolDefinition};
use axum::{Router, routing::get, routing::post, extract::State, Json};

async fn tools_list() -> Json<Vec<ToolDefinition>> {
    Json(get_tool_definitions())
}

async fn tool_execute(
    State(_state): State<AppState>,
    Json(req): Json<ToolRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    crate::metrics::TOOL_CALLS_TOTAL.inc();
    let _tool_timer = crate::metrics::TOOL_CALL_DURATION.start_timer();

    let cwd = req.cwd.clone().unwrap_or_else(|| ".".to_string());
    let name = req.name.clone();
    let name_log = name.clone();
    let input_log = serde_json::to_string(&req.input).unwrap_or_default();

    let start_time = std::time::Instant::now();
    let result = tokio::task::spawn_blocking(move || {
        execute_tool(&name, req.input, &cwd)
    }).await;

    let duration_ms = start_time.elapsed().as_millis() as u64;

    match result {
        Ok(Ok(ref result_value)) => {
            tracing::info!(
                module = "Audit",
                "Tool: {} | Success: true | Duration: {}ms | Input: {}",
                name_log, duration_ms, input_log
            );
            Ok(Json(result_value.clone()))
        }
        _ => {
            tracing::error!(
                module = "Audit",
                "Tool: {} | Success: false | Duration: {}ms | Input: {}",
                name_log, duration_ms, input_log
            );
            Err(ApiError::internal("tool execution failed"))
        }
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/tools", get(tools_list))
        .route("/api/tool/execute", post(tool_execute))
}
