use axum::{routing::{get, post, put}, Router};
use super::super::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/memory/search", post(super::super::memory_search_handler))
        .route("/api/memory/ingest", post(super::super::memory_ingest_handler))
        .route("/api/memory/stats", get(super::super::memory_stats_handler))
        .route("/api/memory/config", get(super::super::memory_get_config_handler))
        .route("/api/memory/config", put(super::super::memory_update_config_handler))
        .route("/api/memory/clear", post(super::super::memory_clear_handler))
}
