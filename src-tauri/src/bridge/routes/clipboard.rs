use axum::{routing::{get, post}, Router};
use super::super::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/clipboard/read", get(super::super::clipboard_read))
        .route("/api/clipboard/write", post(super::super::clipboard_write))
}
