use axum::{routing::{get, post}, Router};
use super::super::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/tasks", post(super::super::task_execute))
        .route("/api/tasks/{id}/status", get(super::super::task_status))
        .route("/api/tasks/{id}/cancel", post(super::super::task_cancel))
}
