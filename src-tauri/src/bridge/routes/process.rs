use axum::{routing::{delete, get, post}, Router};
use super::super::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/process/spawn", post(super::super::process_spawn))
        .route("/api/process/{pid}", delete(super::super::process_kill))
        .route("/api/process/list", get(super::super::process_list))
}
