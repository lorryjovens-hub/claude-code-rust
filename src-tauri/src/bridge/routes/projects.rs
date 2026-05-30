use axum::{
    routing::{get, post},
    Router,
};
use super::super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/projects", get(super::super::projects_list))
        .route("/api/projects", post(super::super::projects_create))
}
