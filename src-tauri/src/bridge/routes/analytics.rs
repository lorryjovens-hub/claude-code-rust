use axum::{routing::{get, post}, Router};
use super::super::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/analytics/dashboard", get(super::super::analytics_dashboard))
        .route("/api/analytics/track", post(super::super::analytics_track_event))
        .route("/api/analytics/daily/{date}", get(super::super::analytics_daily))
        .route("/api/analytics/range", get(super::super::analytics_range))
        .route("/api/analytics/summary", get(super::super::analytics_summary))
        .route("/api/analytics/event-counts", get(super::super::analytics_event_counts))
        .route("/api/analytics/recent-events", get(super::super::analytics_recent_events))
}
