use axum::{routing::{get, post}, Router};
use super::super::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/caveman/add", post(super::super::caveman_add_memory))
        .route("/api/caveman/query", post(super::super::caveman_query_memory))
        .route("/api/caveman/rlm/feedback", post(super::super::caveman_rlm_feedback))
        .route("/api/caveman/rlm/iterate", post(super::super::caveman_rlm_iterate))
        .route("/api/caveman/stats", get(super::super::caveman_stats))
}
