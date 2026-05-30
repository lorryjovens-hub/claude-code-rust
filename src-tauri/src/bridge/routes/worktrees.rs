use axum::{routing::{delete, get, post}, Router};
use super::super::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/worktrees", get(super::super::worktree_list))
        .route("/api/worktrees", post(super::super::worktree_create))
        .route("/api/worktrees/sync", post(super::super::worktree_sync))
        .route("/api/worktrees/{id}", get(super::super::worktree_get))
        .route("/api/worktrees/{id}", delete(super::super::worktree_remove))
        .route("/api/worktrees/merge", post(super::super::worktree_merge))
}
