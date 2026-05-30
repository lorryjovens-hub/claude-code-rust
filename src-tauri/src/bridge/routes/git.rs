use axum::{routing::{get, post}, Router};
use super::super::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/git/status", get(super::super::git_status_handler))
        .route("/api/git/log", get(super::super::git_log_handler))
        .route("/api/git/diff", get(super::super::git_diff_handler))
        .route("/api/git/commit", post(super::super::git_commit_handler))
        .route("/api/git/push", post(super::super::git_push_handler))
        .route("/api/git/pull", post(super::super::git_pull_handler))
}
