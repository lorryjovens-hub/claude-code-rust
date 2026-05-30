use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
    Json, Router,
};
use axum::response::IntoResponse;
use serde_json;
use std::collections::HashMap;
use std::convert::Infallible;
use tokio::sync::broadcast;
use tokio::time::Duration;

use super::super::{AppState, ResearchTask, set_sse_content_type};
use crate::config::ConfigManager;
use crate::native_engine::engine_core::ChatRequest;
use crate::native_engine::NativeEngine;
use crate::research::{ResearchEvent, ResearchOrchestrator, ResearchRequest};
use crate::streaming::{SSE_IDLE_TIMEOUT_SECS, SSE_MAX_DURATION_SECS};

async fn research_start_handler(State(state): State<AppState>, Json(req): Json<ChatRequest>) -> Json<serde_json::Value> {
    let research_id = uuid::Uuid::new_v4().to_string();
    let native_engine = state.native_engine.clone();
    let config_manager = state.config_manager.clone();
    let active_research = state.active_research.clone();

    let model = if req.model.is_empty() { "claude-sonnet-4-6".to_string() } else { req.model.clone() };
    let query = req.messages.last()
        .and_then(|m| m.get("content").and_then(|c: &serde_json::Value| c.as_str()).map(String::from))
        .unwrap_or_default();

    let providers_to_sync = {
        let cm_guard: tokio::sync::MutexGuard<'_, Option<ConfigManager>> = config_manager.lock().await;
        if let Some(cm) = cm_guard.as_ref() {
            cm.get_config().providers.iter().map(|p| {
                crate::native_engine::provider_manager::Provider {
                    id: p.id.clone(),
                    name: p.name.clone(),
                    base_url: p.base_url.clone(),
                    api_key: p.api_key.clone().unwrap_or_default(),
                    api_format: {
                        let is_deepseek = p.base_url.contains("deepseek");
                        if p.provider_type == "anthropic" && !is_deepseek {
                            crate::native_engine::provider_manager::ApiFormat::Anthropic
                        } else {
                            crate::native_engine::provider_manager::ApiFormat::OpenAI
                        }
                    },
                    models: p.models.iter().map(|m| crate::native_engine::provider_manager::ModelConfig {
                        id: m.id.clone(),
                        name: m.name.clone(),
                        enabled: m.enabled,
                        max_tokens: m.max_tokens,
                        supports_vision: m.supports_vision,
                        supports_web_search: false,
                    }).collect(),
                    enabled: p.enabled,
                    web_search_strategy: p.web_search_strategy.clone(),
                }
            }).collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    };

    let resolved = {
        let mut engine_guard: tokio::sync::MutexGuard<'_, Option<NativeEngine>> = native_engine.lock().await;
        if let Some(engine) = engine_guard.as_mut() {
            engine.sync_providers(providers_to_sync).await;
            engine.resolve_provider(&model).await
        } else {
            None
        }
    };

    let resolved = match resolved {
        Some(r) => r,
        None => return Json(serde_json::json!({ "ok": false, "error": format!("No provider found for model: {}", model) })),
    };

    let api_key = resolved.provider.api_key.clone();
    let base_url = resolved.provider.base_url.clone();

    let (bcast_tx, _) = broadcast::channel::<ResearchEvent>(256);
    let (mpsc_tx, mut mpsc_rx) = tokio::sync::mpsc::unbounded_channel::<ResearchEvent>();

    let bcast_tx_clone = bcast_tx.clone();
    let research_request = ResearchRequest {
        query: query.clone(),
        api_key,
        base_url,
        model,
    };

    let handle = tokio::spawn(async move {
        let bcast = bcast_tx_clone.clone();
        let forward_handle = tokio::spawn(async move {
            while let Some(event) = mpsc_rx.recv().await {
                let _ = bcast.send(event);
            }
        });

        let orchestrator = ResearchOrchestrator::new(reqwest::Client::new());
        if let Err(e) = orchestrator.run_pipeline(research_request, mpsc_tx).await {
            tracing::error!(module = "Research", "Pipeline error: {}", e);
        }

        let _ = forward_handle.await;
    });

    {
        let mut research: tokio::sync::MutexGuard<'_, HashMap<String, ResearchTask>> = active_research.lock().await;
        research.insert(research_id.clone(), ResearchTask {
            handle,
            event_tx: bcast_tx,
        });
    }

    Json(serde_json::json!({ "ok": true, "research_id": research_id }))
}

async fn research_stop_handler(Path(id): Path<String>, State(state): State<AppState>) -> Json<serde_json::Value> {
    let active_research = state.active_research.clone();
    let mut research: tokio::sync::MutexGuard<'_, HashMap<String, ResearchTask>> = active_research.lock().await;
    if let Some(task) = research.remove(&id) {
        task.handle.abort();
        Json(serde_json::json!({ "ok": true }))
    } else {
        Json(serde_json::json!({ "ok": false, "error": "Research task not found" }))
    }
}

async fn research_status_handler(Path(id): Path<String>, State(state): State<AppState>) -> Json<serde_json::Value> {
    let active_research = state.active_research.clone();
    let research: tokio::sync::MutexGuard<'_, HashMap<String, ResearchTask>> = active_research.lock().await;
    if let Some(task) = research.get(&id) {
        if task.handle.is_finished() {
            Json(serde_json::json!({ "status": "Completed" }))
        } else {
            Json(serde_json::json!({ "status": "Running" }))
        }
    } else {
        Json(serde_json::json!({ "status": "NotFound" }))
    }
}

async fn research_events_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let active_research = state.active_research.clone();
    let research: tokio::sync::MutexGuard<'_, HashMap<String, ResearchTask>> = active_research.lock().await;
    let task = research.get(&id).ok_or(StatusCode::NOT_FOUND)?;
    let mut rx = task.event_tx.subscribe();
    drop(research);

    let idle_timeout = Duration::from_secs(SSE_IDLE_TIMEOUT_SECS);
    let max_duration = Duration::from_secs(SSE_MAX_DURATION_SECS);
    let mut last_activity = std::time::Instant::now();

    let stream = async_stream::stream! {
        let max_sleep = tokio::time::sleep(max_duration);
        tokio::pin!(max_sleep);

        loop {
            let idle_sleep = tokio::time::sleep(idle_timeout);
            tokio::pin!(idle_sleep);

            tokio::select! {
                _ = &mut max_sleep => {
                    tracing::error!(module = "SSE_research_events", "Max duration reached, closing stream");
                    break;
                }
                _ = &mut idle_sleep => {
                    if last_activity.elapsed() >= idle_timeout {
                        tracing::error!(module = "SSE_research_events", "Idle timeout, closing stream");
                        break;
                    }
                }
                result = rx.recv() => {
                    match result {
                        Ok(event) => {
                            last_activity = std::time::Instant::now();
                            let data = serde_json::to_string(&event).unwrap_or_default();
                            yield Ok::<Event, Infallible>(Event::default()
                                .event("research")
                                .data(data));
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            tracing::error!(module = "SSE_research_events", "Receiver lagged, dropped {} events", n);
                            last_activity = std::time::Instant::now();
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
            }
        }

        tracing::info!(module = "SSE_research_events", "Stream ended for research_id={}", id);
    };

    let mut response = Sse::new(stream).keep_alive(KeepAlive::default()).into_response();
    set_sse_content_type(&mut response);
    Ok(response)
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/research/start", post(research_start_handler))
        .route("/api/research/{id}/stop", post(research_stop_handler))
        .route("/api/research/status/{id}", get(research_status_handler))
        .route("/api/research/{id}/events", get(research_events_handler))
}
