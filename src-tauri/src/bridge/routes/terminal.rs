use axum::{routing::{get, post}, Router};
use super::super::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/terminal/create", post(super::super::terminal_create))
        .route("/api/terminal/write", post(super::super::terminal_write))
        .route("/api/terminal/resize", post(super::super::terminal_resize))
        .route("/api/terminal/close", post(super::super::terminal_close))
        .route("/api/terminal/list", get(super::super::terminal_list))
        .route("/api/terminal/{session_id}/stream", get(super::super::terminal_output_stream))
}
