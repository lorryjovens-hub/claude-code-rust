use axum::{routing::post, Router};
use super::super::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/watcher/start", post(super::super::watcher_start))
        .route("/api/watcher/watch", post(super::super::watcher_watch))
        .route("/api/watcher/unwatch", post(super::super::watcher_unwatch))
}
