use axum::{routing::{get, post}, Router};
use super::super::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/mcp/servers", get(super::super::mcp_servers_list))
        .route("/api/mcp/servers", post(super::super::mcp_servers_update))
        .route("/api/mcp/servers/{name}/tools", get(super::super::mcp_tools_list))
        .route("/api/mcp/servers/{name}/resources", get(super::super::mcp_resources_list))
        .route("/api/mcp/servers/{name}/resources/{uri}", get(super::super::mcp_resource_read))
        .route("/api/mcp/servers/{name}/resources/{uri}/monitor", post(super::super::mcp_resource_monitor))
        .route("/api/mcp/servers/{name}/connect", post(super::super::mcp_connect_handler))
        .route("/api/mcp/servers/{name}/disconnect", post(super::super::mcp_disconnect_handler))
}
