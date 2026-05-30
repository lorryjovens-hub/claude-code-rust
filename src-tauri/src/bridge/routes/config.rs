use axum::{
    routing::{get, post},
    Router,
};
use super::super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/config", get(super::super::config_get))
        .route("/api/config", post(super::super::config_update))
}
