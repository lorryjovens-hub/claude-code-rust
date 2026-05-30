//! Health check endpoints — no auth required.

use crate::api::error::ApiError;
use crate::api::state::AppState;
use axum::{Router, routing::get, extract::State, response::IntoResponse, Json};
use serde_json::json;

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let db_healthy = state.db_manager.with_conn(|conn| {
        conn.execute_batch("SELECT 1")
    }).is_ok();

    let active_sse = crate::metrics::ACTIVE_SSE_CONNECTIONS.get();

    let status = if db_healthy { "healthy" } else { "unhealthy" };
    let http_status = if db_healthy { axum::http::StatusCode::OK } else { axum::http::StatusCode::SERVICE_UNAVAILABLE };

    (http_status, Json(json!({
        "status": status,
        "database": db_healthy,
        "active_sse_connections": active_sse,
    })))
}

async fn metrics() -> Result<String, ApiError> {
    let metrics = crate::metrics::gather_metrics();
    Ok(metrics)
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics))
}
