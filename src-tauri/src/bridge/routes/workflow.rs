use axum::{routing::{get, post}, Router};
use super::super::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/workflow/execute", post(super::super::workflow_execute))
        .route("/api/workflow/execute-stream", post(super::super::workflow_execute_stream))
        .route("/api/workflow/stats", get(super::super::workflow_stats))
        .route("/api/workflow/config", get(super::super::workflow_config_get))
        .route("/api/workflow/config", post(super::super::workflow_config_set))
}
