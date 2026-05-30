use axum::{routing::{get, post}, Router};
use super::super::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/agents", get(super::super::agent_list))
        .route("/api/agents/{id}", get(super::super::agent_get))
        .route("/api/agents/{id}/cancel", post(super::super::agent_cancel))
}
