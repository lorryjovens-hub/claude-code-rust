use axum::{
    extract::{Path, State},
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::{Deserialize};
use serde_json;
use std::collections::HashMap;

use super::super::AppState;
use crate::config::ConfigManager;

async fn providers_list(State(state): State<AppState>) -> Json<serde_json::Value> {
    let config_manager = state.config_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, Option<ConfigManager>> = config_manager.lock().await;
    if let Some(m) = manager.as_ref() {
        let config = m.get_config();
        let providers: Vec<serde_json::Value> = config.providers.iter().map(|p| {
            serde_json::json!({
                "id": p.id,
                "name": p.name,
                "apiKey": p.api_key,
                "baseUrl": p.base_url,
                "format": p.provider_type,
                "models": p.models.iter().map(|m| serde_json::json!({
                    "id": m.id,
                    "name": m.name,
                    "enabled": m.enabled,
                })).collect::<Vec<_>>(),
                "enabled": p.enabled,
                "supportsWebSearch": p.supports_web_search,
                "webSearchStrategy": p.web_search_strategy,
                "webSearchTestedAt": p.web_search_tested_at,
                "webSearchTestReason": p.web_search_test_reason,
            })
        }).collect();
        return Json(serde_json::json!({ "providers": providers }));
    }
    Json(serde_json::json!({ "providers": [] }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateProviderRequest {
    name: String,
    base_url: Option<String>,
    api_key: Option<String>,
    format: Option<String>,
    models: Option<Vec<serde_json::Value>>,
    enabled: Option<bool>,
    supports_web_search: Option<bool>,
}

async fn providers_create(State(state): State<AppState>, Json(req): Json<CreateProviderRequest>) -> Json<serde_json::Value> {
    let config_manager = state.config_manager.clone();
    let mut manager: tokio::sync::MutexGuard<'_, Option<ConfigManager>> = config_manager.lock().await;
    if let Some(m) = manager.as_mut() {
        let id = uuid::Uuid::new_v4().to_string();
        let provider_type = req.format.unwrap_or_else(|| "openai".to_string());
        let models: Vec<crate::config::ModelConfig> = req.models.unwrap_or_default().iter().map(|m| {
            crate::config::ModelConfig {
                id: m.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                name: m.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                enabled: m.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
                max_tokens: None,
                supports_vision: false,
                supports_tools: true,
                supports_streaming: true,
                context_window: None,
                cost_per_1k_input: None,
                cost_per_1k_output: None,
            }
        }).collect();
        let new_provider = crate::config::ProviderConfig {
            id: id.clone(),
            name: req.name.clone(),
            provider_type,
            api_key: if req.api_key.as_ref().map_or(false, |k| k.is_empty()) { None } else { req.api_key.clone() },
            base_url: req.base_url.clone().unwrap_or_default(),
            models,
            enabled: req.enabled.unwrap_or(true),
            is_default: false,
            settings: std::collections::HashMap::new(),
            supports_web_search: req.supports_web_search.unwrap_or(false),
            web_search_strategy: None,
            web_search_tested_at: None,
            web_search_test_reason: None,
        };
        match m.add_provider(new_provider) {
            Ok(()) => {
                let created_id = id.clone();
                drop(manager);
                let state_clone = state.clone();
                sync_provider_manager_owned(state_clone).await;
                let config_manager2 = state.config_manager.clone();
                let manager2: tokio::sync::MutexGuard<'_, Option<ConfigManager>> = config_manager2.lock().await;
                if let Some(m2) = manager2.as_ref() {
                    if let Some(created) = m2.get_provider(&created_id) {
                        return Json(serde_json::json!({
                            "id": created.id,
                            "name": created.name,
                            "apiKey": created.api_key,
                            "baseUrl": created.base_url,
                            "format": created.provider_type,
                            "models": created.models.iter().map(|m| serde_json::json!({"id": m.id, "name": m.name, "enabled": m.enabled})).collect::<Vec<_>>(),
                            "enabled": created.enabled,
                            "supportsWebSearch": created.supports_web_search,
                            "webSearchStrategy": created.web_search_strategy,
                        }));
                    }
                }
                Json(serde_json::json!({ "error": "Provider created but not found" }))
            }
            Err(e) => Json(serde_json::json!({ "error": format!("{}", e) }))
        }
    } else {
        Json(serde_json::json!({ "error": "Config manager not initialized" }))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateProviderRequest {
    name: Option<String>,
    base_url: Option<String>,
    api_key: Option<String>,
    format: Option<String>,
    models: Option<Vec<serde_json::Value>>,
    enabled: Option<bool>,
    supports_web_search: Option<bool>,
    web_search_strategy: Option<Option<String>>,
    web_search_tested_at: Option<Option<u64>>,
    web_search_test_reason: Option<Option<String>>,
}

async fn providers_patch(Path(id): Path<String>, State(state): State<AppState>, Json(updates): Json<HashMap<String, serde_json::Value>>) -> Json<serde_json::Value> {
    let config_manager = state.config_manager.clone();
    let mut manager: tokio::sync::MutexGuard<'_, Option<ConfigManager>> = config_manager.lock().await;
    if let Some(m) = manager.as_mut() {
        let config = m.get_config();
        let idx = config.providers.iter().position(|p| p.id == id);
        if let Some(idx) = idx {
            m.update_config(|c| {
                if let Some(name) = updates.get("name").and_then(|v| v.as_str()) {
                    c.providers[idx].name = name.to_string();
                }
                if let Some(base_url) = updates.get("baseUrl").and_then(|v| v.as_str()) {
                    c.providers[idx].base_url = base_url.to_string();
                }
                if let Some(api_key) = updates.get("apiKey").and_then(|v| v.as_str()) {
                    c.providers[idx].api_key = Some(api_key.to_string());
                }
                if let Some(format) = updates.get("format").and_then(|v| v.as_str()) {
                    c.providers[idx].provider_type = format.to_string();
                }
                if let Some(enabled) = updates.get("enabled").and_then(|v| v.as_bool()) {
                    c.providers[idx].enabled = enabled;
                }
                if let Some(sws) = updates.get("supportsWebSearch") {
                    c.providers[idx].supports_web_search = sws.as_bool().unwrap_or(false);
                }
                if let Some(strategy) = updates.get("webSearchStrategy") {
                    c.providers[idx].web_search_strategy = if strategy.is_null() { None } else { strategy.as_str().map(|s| s.to_string()) };
                }
                if let Some(tested_at) = updates.get("webSearchTestedAt") {
                    c.providers[idx].web_search_tested_at = if tested_at.is_null() { None } else { tested_at.as_u64() };
                }
                if let Some(reason) = updates.get("webSearchTestReason") {
                    c.providers[idx].web_search_test_reason = if reason.is_null() { None } else { reason.as_str().map(|s| s.to_string()) };
                }
                if let Some(models_val) = updates.get("models").and_then(|v| v.as_array()) {
                    c.providers[idx].models = models_val.iter().map(|m| {
                        crate::config::ModelConfig {
                            id: m.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            name: m.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            enabled: m.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
                            max_tokens: None,
                            supports_vision: false,
                            supports_tools: true,
                            supports_streaming: true,
                            context_window: None,
                            cost_per_1k_input: None,
                            cost_per_1k_output: None,
                        }
                    }).collect();
                }
            }).ok();

            drop(manager);
            sync_provider_manager_owned(state.clone()).await;

            let config_manager2 = state.config_manager.clone();
            let manager2: tokio::sync::MutexGuard<'_, Option<ConfigManager>> = config_manager2.lock().await;
            if let Some(m2) = manager2.as_ref() {
                if let Some(p) = m2.get_provider(&id) {
                    return Json(serde_json::json!({
                        "id": p.id,
                        "name": p.name,
                        "apiKey": p.api_key,
                        "baseUrl": p.base_url,
                        "format": p.provider_type,
                        "models": p.models.iter().map(|m| serde_json::json!({"id": m.id, "name": m.name, "enabled": m.enabled})).collect::<Vec<_>>(),
                        "enabled": p.enabled,
                        "supportsWebSearch": p.supports_web_search,
                        "webSearchStrategy": p.web_search_strategy,
                        "webSearchTestedAt": p.web_search_tested_at,
                        "webSearchTestReason": p.web_search_test_reason,
                    }));
                }
            }
            Json(serde_json::json!({ "error": "Provider not found after update" }))
        } else {
            Json(serde_json::json!({ "error": format!("Provider '{}' not found", id) }))
        }
    } else {
        Json(serde_json::json!({ "error": "Config manager not initialized" }))
    }
}

async fn providers_delete(Path(id): Path<String>, State(state): State<AppState>) -> Json<serde_json::Value> {
    let config_manager = state.config_manager.clone();
    let mut manager: tokio::sync::MutexGuard<'_, Option<ConfigManager>> = config_manager.lock().await;
    if let Some(m) = manager.as_mut() {
        match m.remove_provider(&id) {
            Ok(()) => {
                drop(manager);
                sync_provider_manager_owned(state.clone()).await;
                Json(serde_json::json!({ "ok": true }))
            }
            Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
        }
    } else {
        Json(serde_json::json!({ "error": "Config manager not initialized" }))
    }
}

async fn providers_models_list(State(state): State<AppState>) -> Json<serde_json::Value> {
    let config_manager = state.config_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, Option<ConfigManager>> = config_manager.lock().await;
    if let Some(m) = manager.as_ref() {
        let config = m.get_config();
        let models: Vec<serde_json::Value> = config.providers.iter()
            .filter(|p| p.enabled)
            .flat_map(|p| {
                p.models.iter()
                    .filter(|m| m.enabled)
                    .map(|m| serde_json::json!({
                        "id": m.id,
                        "name": m.name,
                        "providerId": p.id,
                        "providerName": p.name,
                    }))
            })
            .collect();
        return Json(serde_json::json!({ "models": models }));
    }
    Json(serde_json::json!({ "models": [] }))
}

async fn providers_test_websearch(Path(id): Path<String>, State(state): State<AppState>) -> Json<serde_json::Value> {
    let config_manager = state.config_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, Option<ConfigManager>> = config_manager.lock().await;
    if let Some(m) = manager.as_ref() {
        if let Some(provider) = m.get_provider(&id) {
            let api_key = provider.api_key.clone().unwrap_or_default();
            let base_url = provider.base_url.clone();
            let provider_type = provider.provider_type.clone();
            drop(manager);

            let result = test_web_search_capability(&id, &api_key, &base_url, &provider_type).await;

            let config_manager = state.config_manager.clone();
            let mut manager: tokio::sync::MutexGuard<'_, Option<ConfigManager>> = config_manager.lock().await;
            if let Some(m) = manager.as_mut() {
                if let Some(provider) = m.get_provider_mut(&id) {
                    provider.supports_web_search = result.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
                    provider.web_search_strategy = result.get("strategy").and_then(|v| v.as_str()).map(String::from);
                    provider.web_search_tested_at = Some(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs());
                    provider.web_search_test_reason = result.get("reason").and_then(|v| v.as_str()).map(String::from);
                    let _ = m.save();
                }
            }
            return Json(result);
        }
    }
    Json(serde_json::json!({ "ok": false, "reason": "Provider not found" }))
}

async fn sync_provider_manager(state: &AppState) {
    let config_manager = state.config_manager.clone();
    let native_engine = state.native_engine.clone();

    let providers_to_sync = {
        let cm_guard = config_manager.lock().await;
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
                        supports_web_search: p.supports_web_search,
                    }).collect(),
                    enabled: p.enabled,
                    web_search_strategy: p.web_search_strategy.clone(),
                }
            }).collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    };

    let mut engine_guard = native_engine.lock().await;
    if let Some(engine) = engine_guard.as_mut() {
        engine.sync_providers(providers_to_sync).await;
        tracing::info!(module = "Bridge", "ProviderManager synced with ConfigManager providers");
    }
}

async fn sync_provider_manager_owned(state: AppState) {
    let config_manager = state.config_manager.clone();
    let native_engine = state.native_engine.clone();

    let providers_to_sync = {
        let cm_guard = config_manager.lock().await;
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
                        supports_web_search: p.supports_web_search,
                    }).collect(),
                    enabled: p.enabled,
                    web_search_strategy: p.web_search_strategy.clone(),
                }
            }).collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    };

    let mut engine_guard = native_engine.lock().await;
    if let Some(engine) = engine_guard.as_mut() {
        engine.sync_providers(providers_to_sync).await;
        tracing::info!(module = "Bridge", "ProviderManager synced with ConfigManager providers");
    }
}

async fn test_web_search_capability(_id: &str, _api_key: &str, _base_url: &str, _provider_type: &str) -> serde_json::Value {
    serde_json::json!({ "ok": false, "reason": "Web search test not yet implemented" })
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/providers", get(providers_list))
        .route("/api/providers", post(providers_create))
        .route("/api/providers/{id}", patch(providers_patch))
        .route("/api/providers/{id}", delete(providers_delete))
        .route("/api/providers/models", get(providers_models_list))
        .route("/api/providers/{id}/test-websearch", post(providers_test_websearch))
}
