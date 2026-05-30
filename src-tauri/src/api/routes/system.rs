//! System status and workspace config endpoints — no auth required.

use crate::api::state::AppState;
use crate::commands::SystemStatus;
use axum::{Router, routing::get, response::IntoResponse, Json};
use serde::Serialize;
use serde_json::json;

#[derive(Serialize)]
struct WorkspaceConfig {
    default_dir: String,
}

async fn system_status() -> Json<SystemStatus> {
    let platform = std::env::consts::OS.to_string();
    Json(SystemStatus {
        platform,
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn workspace_config_get() -> Json<WorkspaceConfig> {
    let default_dir = dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());
    Json(WorkspaceConfig { default_dir })
}

async fn workspace_config_set(Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    tracing::warn!(module = "WorkspaceConfig", "Set called with: {:?}", body);
    Json(json!({"ok": true}))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/system-status", get(system_status))
        .route("/api/workspace-config", get(workspace_config_get).put(workspace_config_set))
}
