use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize};
use serde_json;

use super::super::AppState;
use crate::engine::EnginePool;
use crate::native_engine::NativeEngine;
use crate::permissions::PermissionMode;

#[derive(Deserialize)]
struct CreateConversationBody {
    title: Option<String>,
    model: Option<String>,
    research_mode: Option<bool>,
}

async fn conversations_list(State(state): State<AppState>) -> Json<serde_json::Value> {
    let db = state.db_manager.clone();
    let result = tokio::task::spawn_blocking(move || {
        db.with_conn(|conn| crate::db::conversation_repo::list_conversations(conn))
    }).await;
    match result {
        Ok(Ok(Ok(convs))) => Json(serde_json::json!({ "conversations": convs })),
        _ => Json(serde_json::json!({ "conversations": [] })),
    }
}

async fn conversations_create(State(state): State<AppState>, Json(body): Json<CreateConversationBody>) -> Json<serde_json::Value> {
    let id = uuid::Uuid::new_v4().to_string();
    let db = state.db_manager.clone();
    let id_clone = id.clone();
    let title_clone = body.title.clone();
    let model_clone = body.model.clone();
    let result = tokio::task::spawn_blocking(move || {
        db.with_conn(|conn| {
            let now = chrono::Utc::now().to_rfc3339();
            crate::db::conversation_repo::insert_conversation(
                conn, &id_clone,
                title_clone.as_deref(),
                model_clone.as_deref(),
                None, None, None,
                false, false, false,
                &now, &now, 0
            )
        })
    }).await;
    let now = chrono::Utc::now().to_rfc3339();
    let success = result.is_ok();
    Json(serde_json::json!({
        "id": id,
        "title": body.title,
        "model": body.model.unwrap_or_default(),
        "workspace_path": "",
        "created_at": now.clone(),
        "updated_at": now,
        "success": success,
    }))
}

async fn conversation_get(Path(id): Path<String>, State(state): State<AppState>) -> Json<serde_json::Value> {
    let db = state.db_manager.clone();
    let id_clone = id.clone();
    let result = tokio::task::spawn_blocking(move || {
        db.with_conn(|conn| {
            let conv = crate::db::conversation_repo::get_conversation(conn, &id_clone).ok().flatten();
            let messages = crate::db::message_repo::get_messages_by_conversation(conn, &id_clone).unwrap_or_default();

            // Enrich each message with its tool_calls from the separate table
            let enriched_messages: Vec<serde_json::Value> = messages.iter().map(|msg| {
                let tool_calls = crate::db::message_repo::get_tool_calls_for_message(conn, &msg.id)
                    .unwrap_or_default();
                let mut msg_json = serde_json::to_value(msg).unwrap_or(serde_json::json!({}));
                if !tool_calls.is_empty() {
                    // Format as the frontend expects: array of { name, input (parsed), result, status }
                    let formatted_tool_calls: Vec<serde_json::Value> = tool_calls.iter().map(|tc| {
                        let input_parsed: serde_json::Value = tc.input.as_ref()
                            .and_then(|s| serde_json::from_str(s).ok())
                            .unwrap_or(serde_json::json!({}));
                        serde_json::json!({
                            "id": tc.id,
                            "name": tc.name,
                            "input": input_parsed,
                            "result": tc.output,
                            "status": if tc.is_error { "error" } else { "done" },
                        })
                    }).collect();
                    msg_json["tool_calls"] = serde_json::json!(formatted_tool_calls);
                }
                msg_json
            }).collect();

            Ok::<_, anyhow::Error>((conv, enriched_messages))
        })
    }).await;
    match result {
        Ok(Ok(Ok((Some(conv), messages)))) => Json(serde_json::json!({
            "id": conv.id,
            "title": conv.title,
            "model": conv.model,
            "workspace_path": conv.workspace_path,
            "project_id": conv.project_id,
            "research_mode": conv.research_mode,
            "created_at": conv.created_at,
            "updated_at": conv.updated_at,
            "message_count": conv.message_count,
            "messages": messages,
        })),
        Ok(Ok(Ok((None, messages)))) => Json(serde_json::json!({ "id": id, "messages": messages })),
        _ => Json(serde_json::json!({ "id": id, "messages": [] })),
    }
}

async fn conversation_update(Path(id): Path<String>, State(state): State<AppState>, Json(body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    let db = state.db_manager.clone();
    let id_for_error = id.clone();
    let id_clone = id.clone();
    let result = tokio::task::spawn_blocking(move || {
        db.with_conn(|conn: &rusqlite::Connection| {
            if let Some(title) = body.get("title").and_then(|v| v.as_str()) {
                let _ = conn.execute(
                    "UPDATE conversations SET title = ?1, updated_at = ?2 WHERE id = ?3",
                    rusqlite::params![title, chrono::Utc::now().to_rfc3339(), &id_clone],
                );
            }
            if let Some(model) = body.get("model").and_then(|v| v.as_str()) {
                let _ = conn.execute(
                    "UPDATE conversations SET model = ?1, updated_at = ?2 WHERE id = ?3",
                    rusqlite::params![model, chrono::Utc::now().to_rfc3339(), &id_clone],
                );
            }
            if let Some(workspace) = body.get("workspace_path").and_then(|v| v.as_str()) {
                let _ = conn.execute(
                    "UPDATE conversations SET workspace_path = ?1, updated_at = ?2 WHERE id = ?3",
                    rusqlite::params![workspace, chrono::Utc::now().to_rfc3339(), &id_clone],
                );
            }
            if let Some(messages) = body.get("messages").and_then(|v| v.as_array()) {
                let tx = conn.unchecked_transaction()?;
                crate::db::message_repo::delete_messages_from(&tx, &id_clone, 0)?;
                for (idx, msg) in messages.iter().enumerate() {
                    // Safe: unwrap_or generates a fresh UUID when msg has no id field
                    let msg_id = msg.get("id").and_then(|v| v.as_str()).unwrap_or(&uuid::Uuid::new_v4().to_string()).to_string();
                    // Safe: unwrap_or defaults to "user" when role field is missing
                    let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("user");
                    let content = match msg.get("content") {
                        Some(v) if v.is_string() => v.as_str().unwrap_or("").to_string(),
                        Some(v) => serde_json::to_string(v).unwrap_or_default(),
                        None => String::new(),
                    };
                    let now = chrono::Utc::now().to_rfc3339();
                    crate::db::message_repo::insert_message(&tx, &msg_id, &id_clone, role, &content, None, &now, false, idx as i64)?;
                }
                crate::db::conversation_repo::increment_message_count(&tx, &id_clone)?;
                tx.commit()?;
            }
            Ok::<(), anyhow::Error>(())
        })
    }).await;
    match result {
        Ok(Ok(Ok(()))) => Json(serde_json::json!({ "ok": true })),
        Err(e) => {
            tracing::error!(module = "Conversations", "Failed to update conversation {}: {}", id_for_error, e);
            Json(serde_json::json!({ "ok": false, "error": format!("{}", e) }))
        }
        Ok(Ok(Err(e))) => {
            tracing::error!(module = "Conversations", "Failed to update conversation {}: {}", id_for_error, e);
            Json(serde_json::json!({ "ok": false, "error": format!("{}", e) }))
        }
        Ok(Err(e)) => {
            tracing::error!(module = "Conversations", "DB error on update {}: {}", id_for_error, e);
            Json(serde_json::json!({ "ok": false, "error": format!("{}", e) }))
        }
    }
}

async fn conversation_delete(Path(id): Path<String>, State(state): State<AppState>) -> Json<serde_json::Value> {
    let db = state.db_manager.clone();
    let id_for_error = id.clone();
    let result = tokio::task::spawn_blocking(move || {
        db.with_conn(|conn| {
            crate::db::message_repo::delete_messages_from(conn, &id, 0).ok();
            crate::db::conversation_repo::delete_conversation(conn, &id)
        })
    }).await;
    match result {
        Ok(Ok(Ok(()))) => Json(serde_json::json!({ "ok": true })),
        Err(e) => {
            tracing::error!(module = "Conversations", "Failed to delete conversation {}: {}", id_for_error, e);
            Json(serde_json::json!({ "ok": false, "error": format!("{}", e) }))
        }
        Ok(Ok(Err(e))) => {
            tracing::error!(module = "Conversations", "Failed to delete conversation {}: {}", id_for_error, e);
            Json(serde_json::json!({ "ok": false, "error": format!("{}", e) }))
        }
        Ok(Err(e)) => {
            tracing::error!(module = "Conversations", "DB error on delete {}: {}", id_for_error, e);
            Json(serde_json::json!({ "ok": false, "error": format!("{}", e) }))
        }
    }
}

async fn conversation_messages(Path(id): Path<String>, State(state): State<AppState>) -> Json<serde_json::Value> {
    let db = state.db_manager.clone();
    let result = tokio::task::spawn_blocking(move || {
        db.with_conn(|conn| crate::db::message_repo::get_messages_by_conversation(conn, &id))
    }).await;
    match result {
        Ok(Ok(Ok(messages))) => Json(serde_json::json!({ "messages": messages })),
        _ => Json(serde_json::json!({ "messages": [] })),
    }
}

async fn conversation_message_delete(
    Path((id, mid)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let db = state.db_manager.clone();
    let result = tokio::task::spawn_blocking(move || {
        db.with_conn(|conn| {
            let msg = crate::db::message_repo::get_message(conn, &mid)?;
            if let Some(m) = msg {
                crate::db::message_repo::delete_messages_from(conn, &id, m.sort_order)?;
            }
            crate::db::message_repo::get_messages_by_conversation(conn, &id)
        })
    }).await;
    match result {
        Ok(Ok(Ok(messages))) => Ok(Json(serde_json::json!({ "success": true, "messages": messages }))),
        Ok(Ok(Err(e))) => { tracing::error!(module = "MessageDelete", "Failed: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
        Ok(Err(e)) => { tracing::error!(module = "MessageDelete", "DB lock error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn conversation_messages_tail_delete(
    Path((id, count)): Path<(String, i64)>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let db = state.db_manager.clone();
    let result = tokio::task::spawn_blocking(move || {
        db.with_conn(|conn| {
            crate::db::message_repo::delete_messages_tail(conn, &id, count)?;
            crate::db::message_repo::get_messages_by_conversation(conn, &id)
        })
    }).await;
    match result {
        Ok(Ok(Ok(messages))) => Ok(Json(serde_json::json!({ "success": true, "messages": messages }))),
        Ok(Ok(Err(e))) => { tracing::error!(module = "MessagesTailDelete", "Failed: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
        Ok(Err(e)) => { tracing::error!(module = "MessagesTailDelete", "DB lock error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
struct BranchRequest {
    from_message_id: Option<String>,
}

async fn conversation_branch_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<BranchRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let db = state.db_manager.clone();
    let result = tokio::task::spawn_blocking(move || {
        db.with_conn(|conn| {
            let new_id = uuid::Uuid::new_v4().to_string();
            let now = chrono::Utc::now().to_rfc3339();
            let source = crate::db::conversation_repo::get_conversation(conn, &id)?;
            let title = source.as_ref().and_then(|c| c.title.as_deref()).unwrap_or("Branched conversation");
            let model = source.as_ref().and_then(|c| c.model.as_deref());
            crate::db::conversation_repo::insert_conversation(
                conn, &new_id, Some(&format!("{} (branch)", title)), model, None, None, None, false, false, false, &now, &now, 0,
            )?;
            let mut messages = crate::db::message_repo::get_messages_by_conversation(conn, &id)?;
            if let Some(mid) = req.from_message_id.as_deref() {
                if let Some(m) = crate::db::message_repo::get_message(conn, mid)? {
                    messages.retain(|msg| msg.sort_order < m.sort_order);
                }
            }
            for msg in &messages {
                let msg_id = uuid::Uuid::new_v4().to_string();
                crate::db::message_repo::insert_message(
                    conn, &msg_id, &new_id, &msg.role, &msg.content, msg.thinking.as_deref(), &msg.created_at, msg.is_compact_boundary, msg.sort_order,
                )?;
            }
            Ok::<String, anyhow::Error>(new_id)
        })
    }).await;
    match result {
        Ok(Ok(Ok(new_id))) => Ok(Json(serde_json::json!({ "success": true, "new_conversation_id": new_id }))),
        Ok(Ok(Err(e))) => { tracing::error!(module = "Branch", "Failed: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
        Ok(Err(e)) => { tracing::error!(module = "Branch", "DB lock error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
struct AnswerRequest {
    request_id: String,
    tool_use_id: Option<String>,
    answers: Option<serde_json::Value>,
}

async fn conversation_answer_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<AnswerRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let engine_pool = state.engine_pool.clone();
    let mut pool: tokio::sync::MutexGuard<'_, EnginePool> = engine_pool.lock().await;

    let original_input = pool.get_ask_user_pending(&id).unwrap_or(serde_json::json!({}));

    let answers = req.answers.unwrap_or(serde_json::json!({}));

    let mut updated_input = original_input;
    if let Some(obj) = updated_input.as_object_mut() {
        obj.insert("answers".to_string(), answers.clone());
    } else {
        updated_input = serde_json::json!({ "answers": answers.clone() });
    }

    let tool_use_id = req.tool_use_id.unwrap_or_default();

    match pool.send_control_response(&id, &req.request_id, &tool_use_id, updated_input).await {
        Ok(()) => Ok(Json(serde_json::json!({ "ok": true }))),
        Err(_) => {
            drop(pool);
            let native_engine = state.native_engine.clone();
            let engine_guard: tokio::sync::MutexGuard<'_, Option<NativeEngine>> = native_engine.lock().await;
            if let Some(engine) = engine_guard.as_ref() {
                let answer_str = serde_json::to_string(&answers).unwrap_or_default();
                match engine.resume_with_answer(&id, answer_str).await {
                    Ok(()) => Ok(Json(serde_json::json!({ "ok": true }))),
                    Err(e) => {
                        tracing::error!(module = "AskUser", "Native engine answer failed: {}", e);
                        Err(StatusCode::NOT_FOUND)
                    }
                }
            } else {
                tracing::error!(module = "AskUser", "No engine available for conversation {}", id);
                Err(StatusCode::NOT_FOUND)
            }
        }
    }
}

#[derive(Deserialize)]
struct WarmRequest {
    permission_mode: Option<String>,
}

async fn conversation_warm_handler(
    Path(_id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<WarmRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Only log permission mode changes, don't spam for every request
    if let Some(ref pm_str) = req.permission_mode {
        let perm_mode = match pm_str.as_str() {
            "ask_permissions" => PermissionMode::AskPermissions,
            "accept_edits" => PermissionMode::AcceptEdits,
            "plan_mode" => PermissionMode::PlanMode,
            "bypass_permissions" => PermissionMode::BypassPermissions,
            _ => PermissionMode::AskPermissions,
        };
        if let Some(engine) = state.native_engine.lock().await.as_ref() {
            engine.set_permission_mode(perm_mode).await;
            // Only log once per mode change, not for every conversation
            // tracing::error!(module = "Bridge", "Warm: permission_mode set to {:?} for conversation {}", perm_mode, id);
        }
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(Deserialize)]
struct PermissionRequest {
    request_id: String,
    tool_use_id: Option<String>,
    behavior: Option<String>,
}

async fn conversation_permission_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<PermissionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let engine_pool = state.engine_pool.clone();
    let mut pool: tokio::sync::MutexGuard<'_, EnginePool> = engine_pool.lock().await;

    let pending = pool.get_tool_permission_pending(&id);
    let tool_use_id = req.tool_use_id
        .or_else(|| pending.as_ref().and_then(|p| p.get("tool_use_id").and_then(|t| t.as_str()).map(String::from)))
        .unwrap_or_default();

    let behavior = req.behavior.unwrap_or_else(|| "allow".to_string());

    let updated_input = pending.and_then(|p| p.get("input").cloned());

    // Try EnginePool first (legacy external process mode)
    let pool_result = pool.send_permission_response(&id, &req.request_id, &tool_use_id, &behavior, updated_input).await;
    if pool_result.is_ok() {
        pool.remove_tool_permission_pending(&id);
        return Ok(Json(serde_json::json!({ "ok": true })));
    }

    // EnginePool failed, try NativeEngine (new Rust-native mode)
    drop(pool);
    let native_engine = state.native_engine.clone();
    let engine_guard: tokio::sync::MutexGuard<'_, Option<NativeEngine>> = native_engine.lock().await;
    if let Some(engine) = engine_guard.as_ref() {
        let answer = if behavior == "allow" { "allow".to_string() } else { "deny".to_string() };
        match engine.resume_with_answer(&id, answer).await {
            Ok(()) => Ok(Json(serde_json::json!({ "ok": true }))),
            Err(e) => {
                tracing::error!(module = "Permission", "Native engine answer failed: {}", e);
                Err(StatusCode::NOT_FOUND)
            }
        }
    } else {
        tracing::error!(module = "Permission", "No pool engine and no native engine for conversation {}", id);
        Err(StatusCode::NOT_FOUND)
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/conversations", get(conversations_list))
        .route("/api/conversations", post(conversations_create))
        .route("/api/conversations/{id}", get(conversation_get))
        .route("/api/conversations/{id}", post(conversation_update))
        .route("/api/conversations/{id}", delete(conversation_delete))
        .route("/api/conversations/{id}/messages", get(conversation_messages))
        .route("/api/conversations/{id}/messages/{mid}", delete(conversation_message_delete))
        .route("/api/conversations/{id}/messages-tail/{count}", delete(conversation_messages_tail_delete))
        .route("/api/conversations/{id}/branch", post(conversation_branch_handler))
        .route("/api/conversations/{id}/answer", post(conversation_answer_handler))
        .route("/api/conversations/{id}/permission", post(conversation_permission_handler))
        .route("/api/conversations/{id}/warm", post(conversation_warm_handler))
}
