use axum::{
    routing::{delete, get, post},
    Router,
};
use super::super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/upload", post(super::super::upload_handler))
        .route("/api/uploads/{id}/raw", get(super::super::upload_get_handler))
        .route("/api/uploads/{id}", delete(super::super::upload_delete_handler))
}
