use axum::{routing::{delete, get, post}, Router};
use super::super::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/engines", get(super::super::engine_status_handler))
        .route("/api/engines/spawn", post(super::super::engine_spawn_handler))
        .route("/api/engines/{conv_id}", delete(super::super::engine_kill_handler))
        .route("/api/streams/{conv_id}", get(super::super::stream_events_handler))
}
