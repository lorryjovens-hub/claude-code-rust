use axum::{routing::{delete, get, post}, Router};
use super::super::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/ide/status", get(super::super::ide_status))
        .route("/api/ide/start", post(super::super::ide_start))
        .route("/api/ide/stop", post(super::super::ide_stop))
        .route("/api/ide/connections", get(super::super::ide_connections))
        .route("/api/ide/connections/{id}", delete(super::super::ide_disconnect))
}
