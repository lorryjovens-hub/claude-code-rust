use axum::{
    routing::{get, post},
    Router,
};
use super::super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(super::super::health_handler))
        .route("/metrics", get(super::super::metrics_handler))
        .route("/api/system-status", get(super::super::system_status))
        .route("/api/workspace-config", get(super::super::workspace_config_get))
        .route("/api/workspace-config", post(super::super::workspace_config_set))
        .route("/api/performance/overview", get(super::super::performance_overview_handler))
        .route("/api/csrf-token", get(super::super::csrf_token_handler))
        .route("/api/interceptor/stats", get(super::super::interceptor_stats_handler))
        .route("/api/interceptor/blocked-paths", get(super::super::interceptor_blocked_paths_handler).post(super::super::interceptor_block_path_handler).delete(super::super::interceptor_unblock_path_handler))
}
