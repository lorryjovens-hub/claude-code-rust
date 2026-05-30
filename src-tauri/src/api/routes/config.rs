//! Configuration endpoints.

use crate::api::state::AppState;
use crate::config::AppConfig;
use axum::{Router, routing::get, extract::State, Json};
use serde_json::json;

async fn config_get(State(state): State<AppState>) -> Json<serde_json::Value> {
    let manager = state.config_manager.lock().await;
    if let Some(m) = manager.as_ref() {
        return Json(serde_json::to_value(m.get_config()).unwrap_or_default());
    }
    Json(json!({}))
}

async fn config_update(State(state): State<AppState>, Json(config): Json<AppConfig>) -> Json<serde_json::Value> {
    let mut manager = state.config_manager.lock().await;
    if let Some(m) = manager.as_mut() {
        let _ = m.update_config(|c| *c = config);
    }
    Json(json!({"ok": true}))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/config", get(config_get).put(config_update))
}
