use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;

use super::super::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SyncProviderPayload {
    id: String,
    name: String,
    #[serde(default)]
    api_key: String,
    #[serde(default)]
    base_url: String,
    #[serde(default)]
    format: String,
    #[serde(default)]
    models: Vec<serde_json::Value>,
    #[serde(default)]
    enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SyncConversationPayload {
    id: String,
    title: Option<String>,
    model: Option<String>,
    provider: Option<String>,
    workspace_path: Option<String>,
    project_id: Option<String>,
    #[serde(default)]
    research_mode: bool,
    #[serde(default)]
    pinned: bool,
    #[serde(default)]
    archived: bool,
    created_at: String,
    updated_at: String,
    message_count: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SyncMessagePayload {
    id: String,
    conversation_id: String,
    role: String,
    content: String,
    thinking: Option<String>,
    created_at: String,
    #[serde(default)]
    is_compact_boundary: bool,
    sort_order: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SyncPushRequest {
    #[serde(default)]
    device_id: String,
    #[serde(default)]
    providers: Vec<SyncProviderPayload>,
    #[serde(default)]
    conversations: Vec<SyncConversationPayload>,
    #[serde(default)]
    messages_per_conversation: HashMap<String, Vec<SyncMessagePayload>>,
    #[serde(default)]
    deleted_provider_ids: Vec<String>,
    #[serde(default)]
    deleted_conversation_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SyncPullRequest {
    #[serde(default)]
    device_id: String,
    #[serde(default)]
    last_pull_at: Option<String>,
}

async fn sync_push(
    State(state): State<AppState>,
    Json(req): Json<SyncPushRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    tracing::info!(
        "Sync Push: {} providers, {} conversations, {} message buckets, {} deleted providers, {} deleted conversations",
        req.providers.len(),
        req.conversations.len(),
        req.messages_per_conversation.len(),
        req.deleted_provider_ids.len(),
        req.deleted_conversation_ids.len()
    );

    if !req.providers.is_empty() {
        let config_manager = state.config_manager.clone();
        let mut manager = config_manager.lock().await;
        if let Some(m) = manager.as_mut() {
            let _ = m.update_config(|config| {
                for sync_p in &req.providers {
                    let provider_type = if sync_p.format == "anthropic" {
                        "anthropic".to_string()
                    } else {
                        "openai".to_string()
                    };
                    let models: Vec<crate::config::ModelConfig> = sync_p
                        .models
                        .iter()
                        .map(|m| {
                            crate::config::ModelConfig {
                                id: m
                                    .get("id")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                name: m
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                enabled: m
                                    .get("enabled")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(true),
                                max_tokens: m
                                    .get("maxTokens")
                                    .and_then(|v| v.as_u64())
                                    .map(|v| v as u32),
                                supports_vision: false,
                                supports_tools: true,
                                supports_streaming: true,
                                context_window: None,
                                cost_per_1k_input: None,
                                cost_per_1k_output: None,
                            }
                        })
                        .collect();

                    let existing = config.providers.iter().position(|p| p.id == sync_p.id);
                    let merged = crate::config::ProviderConfig {
                        id: sync_p.id.clone(),
                        name: sync_p.name.clone(),
                        base_url: if sync_p.base_url.is_empty() {
                            String::from("https://api.openai.com/v1")
                        } else {
                            sync_p.base_url.clone()
                        },
                        api_key: Some(sync_p.api_key.clone()),
                        provider_type,
                        models,
                        enabled: sync_p.enabled,
                        is_default: false,
                        settings: Default::default(),
                        supports_web_search: false,
                        web_search_strategy: None,
                        web_search_tested_at: None,
                        web_search_test_reason: None,
                    };
                    if let Some(idx) = existing {
                        config.providers[idx] = merged;
                    } else {
                        config.providers.push(merged);
                    }
                    tracing::info!("Sync Upserted provider: {} ({})", sync_p.name, sync_p.id);
                }
            });
        }
    }

    for deleted_id in &req.deleted_provider_ids {
        let config_manager = state.config_manager.clone();
        let mut manager = config_manager.lock().await;
        if let Some(m) = manager.as_mut() {
            let _ = m.update_config(|config| {
                config.providers.retain(|p| p.id != *deleted_id);
            });
            tracing::info!("Sync Deleted provider: {}", deleted_id);
        }
    }

    let db = state.db_manager.clone();
    for conv in &req.conversations {
        let id = conv.id.clone();
        let id_for_err = id.clone();
        let title = conv.title.clone();
        let model = conv.model.clone();
        let provider = conv.provider.clone();
        let workspace_path = conv.workspace_path.clone();
        let project_id = conv.project_id.clone();
        let research_mode = conv.research_mode;
        let pinned = conv.pinned;
        let archived = conv.archived;
        let created_at = conv.created_at.clone();
        let updated_at = conv.updated_at.clone();
        let message_count = conv.message_count;
        let db_clone = db.clone();

        let conv_result = tokio::task::spawn_blocking(move || {
            db_clone.with_conn(|conn| {
                let existing = crate::db::conversation_repo::get_conversation(conn, &id)?;
                if existing.is_none() {
                    crate::db::conversation_repo::insert_conversation(
                        conn,
                        &id,
                        title.as_deref(),
                        model.as_deref(),
                        provider.as_deref(),
                        workspace_path.as_deref(),
                        project_id.as_deref(),
                        research_mode,
                        pinned,
                        archived,
                        &created_at,
                        &updated_at,
                        message_count,
                    )
                } else if let Some(ex) = existing {
                    if updated_at > ex.updated_at {
                        conn.execute(
                            "UPDATE conversations SET title=?1, model=?2, provider=?3, workspace_path=?4, project_id=?5, research_mode=?6, pinned=?7, archived=?8, created_at=?9, updated_at=?10, message_count=?11 WHERE id=?12",
                            rusqlite::params![
                                title.as_deref(),
                                model.as_deref(),
                                provider.as_deref(),
                                workspace_path.as_deref(),
                                project_id.as_deref(),
                                research_mode as i64, pinned as i64, archived as i64,
                                &created_at, &updated_at, message_count,
                                &id,
                            ],
                        )?;
                    }
                    Ok(())
                } else {
                    Ok(())
                }
            })
        }).await;

        match conv_result {
            Ok(Ok(Ok(()))) => {}
            Ok(Ok(Err(db_err))) => {
                tracing::error!("Sync DB error for conversation {}: {:?}", id_for_err, db_err);
            }
            Ok(Err(conn_err)) => {
                tracing::error!("Sync Conn error for conversation {}: {:?}", id_for_err, conn_err);
            }
            Err(join_err) => {
                tracing::error!("Sync Join error for conversation {}: {:?}", id_for_err, join_err);
            }
        }
    }

    for (conv_id, messages) in &req.messages_per_conversation {
        for msg in messages {
            let msg_id = msg.id.clone();
            let msg_id_for_err = msg_id.clone();
            let cid = conv_id.clone();
            let role = msg.role.clone();
            let content = msg.content.clone();
            let thinking = msg.thinking.clone();
            let created_at = msg.created_at.clone();
            let is_compact_boundary = msg.is_compact_boundary;
            let sort_order = msg.sort_order;
            let db_clone = db.clone();

            let msg_result = tokio::task::spawn_blocking(move || {
                db_clone.with_conn(|conn| {
                    conn.execute(
                        "INSERT OR REPLACE INTO messages (id, conversation_id, role, content, thinking, created_at, is_compact_boundary, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                        rusqlite::params![
                            &msg_id, &cid, &role, &content,
                            thinking.as_deref(),
                            &created_at,
                            is_compact_boundary as i64,
                            sort_order,
                        ],
                    )?;
                    Ok::<_, anyhow::Error>(())
                })
            }).await;

            match msg_result {
                Ok(Ok(Ok(()))) => {}
                Ok(Ok(Err(db_err))) => {
                    tracing::error!("Sync DB error for message {}: {:?}", msg_id_for_err, db_err);
                }
                Ok(Err(conn_err)) => {
                    tracing::error!("Sync Conn error for message {}: {:?}", msg_id_for_err, conn_err);
                }
                Err(join_err) => {
                    tracing::error!("Sync Join error for message {}: {:?}", msg_id_for_err, join_err);
                }
            }
        }
    }

    for deleted_id in &req.deleted_conversation_ids {
        let id = deleted_id.clone();
        let db_clone = db.clone();
        let result = tokio::task::spawn_blocking(move || {
            db_clone.with_conn(|conn| {
                crate::db::conversation_repo::delete_conversation(conn, &id)
            })
        }).await;

        match result {
            Ok(Ok(Ok(()))) => {
                tracing::info!("Sync Deleted conversation: {}", deleted_id);
            }
            Ok(Ok(Err(db_err))) => {
                tracing::error!("Sync DB error for delete conversation {}: {:?}", deleted_id, db_err);
            }
            Ok(Err(conn_err)) => {
                tracing::error!("Sync Conn error for delete conversation {}: {:?}", deleted_id, conn_err);
            }
            Err(join_err) => {
                tracing::error!("Sync Join error for delete conversation {}: {:?}", deleted_id, join_err);
            }
        }
    }

    let server_timestamp = chrono::Utc::now().to_rfc3339();
    tracing::info!("Sync Push complete at {}", server_timestamp);

    Ok(Json(serde_json::json!({
        "ok": true,
        "serverTimestamp": server_timestamp,
        "deletedProviderIds": [],
        "deletedConversationIds": [],
    })))
}

async fn sync_pull(
    State(state): State<AppState>,
    Json(req): Json<SyncPullRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    tracing::info!(
        "Sync Pull: device={}, lastPullAt={:?}",
        req.device_id,
        req.last_pull_at
    );

    let config_manager = state.config_manager.clone();
    let manager = config_manager.lock().await;
    let providers: Vec<serde_json::Value> = if let Some(m) = manager.as_ref() {
        let config = m.get_config();
        config
            .providers
            .iter()
            .map(|p| {
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
                })
            })
            .collect()
    } else {
        vec![]
    };
    drop(manager);

    let db = state.db_manager.clone();
    let conversations_result = tokio::task::spawn_blocking({
        let db = db.clone();
        move || db.with_conn(|conn| crate::db::conversation_repo::list_conversations(conn))
    })
    .await;

    let conversations = match conversations_result {
        Ok(Ok(Ok(convs))) => convs,
        _ => {
            tracing::error!("Sync Failed to list conversations");
            vec![]
        }
    };

    let mut messages_per_conversation: HashMap<String, Vec<serde_json::Value>> =
        HashMap::new();

    for conv in &conversations {
        let conv_id = conv.id.clone();
        let conv_id_for_closure = conv_id.clone();
        let db_clone = db.clone();
        let msgs_result = tokio::task::spawn_blocking(move || {
            db_clone.with_conn(|conn| {
                crate::db::message_repo::get_messages_by_conversation(conn, &conv_id_for_closure)
            })
        })
        .await;

        if let Ok(Ok(Ok(msgs))) = msgs_result {
            let msg_jsons: Vec<serde_json::Value> = msgs
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "id": m.id,
                        "conversation_id": m.conversation_id,
                        "role": m.role,
                        "content": m.content,
                        "thinking": m.thinking,
                        "created_at": m.created_at,
                        "is_compact_boundary": m.is_compact_boundary,
                        "sort_order": m.sort_order,
                    })
                })
                .collect();
            if !msg_jsons.is_empty() {
                messages_per_conversation.insert(conv_id, msg_jsons);
            }
        }
    }

    let conv_jsons: Vec<serde_json::Value> = conversations
        .iter()
        .map(|c| {
            serde_json::json!({
                "id": c.id,
                "title": c.title,
                "model": c.model,
                "provider": c.provider,
                "workspace_path": c.workspace_path,
                "project_id": c.project_id,
                "research_mode": c.research_mode,
                "pinned": c.pinned,
                "archived": c.archived,
                "created_at": c.created_at,
                "updated_at": c.updated_at,
                "message_count": c.message_count,
            })
        })
        .collect();

    let total_messages: usize = messages_per_conversation.values().map(|v| v.len()).sum();
    let total_convs = conversations.len();
    let server_timestamp = chrono::Utc::now().to_rfc3339();

    tracing::info!(
        "Sync Pull: {} providers, {} conversations, {} messages at {}",
        providers.len(),
        total_convs,
        total_messages,
        server_timestamp
    );

    Ok(Json(serde_json::json!({
        "providers": providers,
        "conversations": conv_jsons,
        "messagesPerConversation": messages_per_conversation,
        "deletedProviderIds": [],
        "deletedConversationIds": [],
        "serverTimestamp": server_timestamp,
    })))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/data/sync/push", post(sync_push))
        .route("/data/sync/pull", post(sync_pull))
}