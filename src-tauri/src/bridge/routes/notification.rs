use axum::{routing::post, Router};
use super::super::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/notification/show", post(super::super::notification_show))
}
