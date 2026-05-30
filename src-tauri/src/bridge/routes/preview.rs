use axum::{routing::{delete, get, post}, Router};
use super::super::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/preview", get(super::super::preview_list_handler))
        .route("/api/preview/{id}", get(super::super::preview_get_handler))
        .route("/api/preview/{id}", post(super::super::preview_set_handler))
        .route("/api/preview/{id}", delete(super::super::preview_delete_handler))
        .route("/api/preview/{id}/events", get(super::super::preview_events_handler))
}
