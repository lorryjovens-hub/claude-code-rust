use crate::api::state::AppState;
use axum::extract::Request;
use axum::extract::State;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Clone)]
pub struct RequestInterceptor {
    enabled: Arc<RwLock<bool>>,
    log_body: Arc<RwLock<bool>>,
    blocked_paths: Arc<RwLock<Vec<String>>>,
}

impl RequestInterceptor {
    pub fn new() -> Self {
        Self {
            enabled: Arc::new(RwLock::new(true)),
            log_body: Arc::new(RwLock::new(false)),
            blocked_paths: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn set_enabled(&self, enabled: bool) {
        let mut e = self.enabled.write().await;
        *e = enabled;
    }

    pub async fn set_log_body(&self, log_body: bool) {
        let mut lb = self.log_body.write().await;
        *lb = log_body;
    }

    pub async fn add_blocked_path(&self, path: &str) {
        let mut bp = self.blocked_paths.write().await;
        bp.push(path.to_string());
    }

    pub async fn remove_blocked_path(&self, path: &str) {
        let mut bp = self.blocked_paths.write().await;
        bp.retain(|p| p != path);
    }

    pub async fn is_path_blocked(&self, path: &str) -> bool {
        let bp = self.blocked_paths.read().await;
        bp.iter().any(|p| path.starts_with(p))
    }
}

pub async fn interceptor_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    let start = Instant::now();
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let query = req.uri().query().map(|q| format!("?{}", q)).unwrap_or_default();

    let full_path = format!("{} {}", method, path);

    let interceptor = &state.request_interceptor;
    let enabled = *interceptor.enabled.read().await;

    if enabled && interceptor.is_path_blocked(&path).await {
        warn!(module = "Interceptor", "Blocked request: {}", full_path);
        return (StatusCode::FORBIDDEN, Json(json!({
            "error": "Request blocked by interceptor",
            "path": path
        }))).into_response();
    }

    let log_body_flag = *interceptor.log_body.read().await;

    if enabled {
        info!(module = "Interceptor", "→ {} {}", method, path);
    }

    let response = next.run(req).await;

    let duration = start.elapsed();
    let status = response.status();

    if enabled {
        info!(module = "Interceptor", "← {} {} → {} ({:?})", method, path, status.as_u16(), duration);
    }

    state.metrics_interceptor.record(method.to_string(), path, status.as_u16(), duration);

    response
}

pub struct MetricsInterceptor {
    total_requests: Arc<RwLock<u64>>,
    total_errors: Arc<RwLock<u64>>,
    total_duration_ms: Arc<RwLock<u64>>,
    request_counts: Arc<RwLock<std::collections::HashMap<String, u64>>>,
}

impl MetricsInterceptor {
    pub fn new() -> Self {
        Self {
            total_requests: Arc::new(RwLock::new(0)),
            total_errors: Arc::new(RwLock::new(0)),
            total_duration_ms: Arc::new(RwLock::new(0)),
            request_counts: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub fn record(&self, method: String, path: String, status: u16, duration: std::time::Duration) {
        let total = *self.total_requests.blocking_read();
        *self.total_requests.blocking_write() = total + 1;
        if status >= 400 {
            let errors = *self.total_errors.blocking_read();
            *self.total_errors.blocking_write() = errors + 1;
        }
        let dur = *self.total_duration_ms.blocking_read();
        *self.total_duration_ms.blocking_write() = dur + duration.as_millis() as u64;
        let key = format!("{} {}", method, path);
        let mut counts = self.request_counts.blocking_write();
        *counts.entry(key).or_insert(0) += 1;
    }

    pub async fn snapshot(&self) -> serde_json::Value {
        let total = *self.total_requests.read().await;
        let errors = *self.total_errors.read().await;
        let total_dur = *self.total_duration_ms.read().await;
        let counts = self.request_counts.read().await;
        let avg_duration = if total > 0 { total_dur / total } else { 0 };
        let mut sorted: Vec<(String, u64)> = counts.iter().map(|(k, v)| (k.clone(), *v)).collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        let top_endpoints: Vec<serde_json::Value> = sorted.into_iter().take(10).map(|(k, v)| {
            json!({ "endpoint": k, "count": v })
        }).collect();
        json!({
            "total_requests": total,
            "total_errors": errors,
            "error_rate": if total > 0 { format!("{:.2}%", errors as f64 / total as f64 * 100.0) } else { "0%".to_string() },
            "avg_duration_ms": avg_duration,
            "top_endpoints": top_endpoints
        })
    }
}