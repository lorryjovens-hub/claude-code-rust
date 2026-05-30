use axum::{routing::{get, post}, Router};
use super::super::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/logs", get(super::super::logs_read))
        .route("/api/logs/clear", post(super::super::logs_clear))
}
