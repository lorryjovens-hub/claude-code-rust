use axum::{routing::{get, post}, Router};
use super::super::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/costs/dashboard", get(super::super::costs_dashboard_handler))
        .route("/api/costs/budget", get(super::super::costs_budget_get_handler))
        .route("/api/costs/budget", post(super::super::costs_budget_set_handler))
        .route("/api/costs/usage", get(super::super::costs_usage_handler))
}
