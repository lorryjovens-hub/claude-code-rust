use axum::{routing::{get, post}, Router};
use super::super::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/computer-use/screen-info", get(super::super::computer_use_screen_info))
        .route("/api/computer-use/execute", post(super::super::computer_use_execute))
        .route("/api/computer-use/screenshot", get(super::super::computer_use_screenshot))
}
