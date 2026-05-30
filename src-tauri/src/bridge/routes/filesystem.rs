use axum::{routing::{delete, get, post}, Router};
use super::super::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/filesystem/tree", get(super::super::fs_tree))
        .route("/api/filesystem/read", get(super::super::fs_read))
        .route("/api/filesystem/read-encoding", get(super::super::fs_read_encoding))
        .route("/api/filesystem/download", get(super::super::fs_download))
        .route("/api/filesystem/write", post(super::super::fs_write))
        .route("/api/filesystem/create", post(super::super::fs_create))
        .route("/api/filesystem/delete", delete(super::super::fs_delete))
}
