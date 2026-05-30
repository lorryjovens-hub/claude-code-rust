use axum::{
    routing::{get, post},
    Router,
};
use super::super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/tools", post(super::super::tools_handler))
        .route("/api/tools/list", get(super::super::tools_list_handler))
        .route("/api/tools/execute", post(super::super::tool_execute_handler))
}
