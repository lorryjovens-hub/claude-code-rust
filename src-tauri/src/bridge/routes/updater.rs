use axum::{routing::{get, post}, Router};
use super::super::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/update/check", get(super::super::update_check))
        .route("/api/update/download", post(super::super::update_download))
}
