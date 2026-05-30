use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
    Json, Router,
};
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::time::Duration;

use super::super::AppState;
use crate::db::DbManager;

async fn validate_h5_token_from_db(
    db_manager: &Arc<DbManager>,
    token: &str,
) -> Result<crate::db::h5_repo::H5TokenRow, StatusCode> {
    let token_str = token.to_string();
    let db = db_manager.clone();
    let inner_result = tokio::task::spawn_blocking(move || {
        db.with_conn(|conn| {
            crate::db::h5_repo::validate_h5_token(conn, &token_str)
                .map_err(|e| {
                    tracing::error!(module = "H5_API", "DB error: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })
        })
    })
    .await
    .map_err(|e| {
        tracing::error!(module = "H5_API", "spawn_blocking error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let inner_result = inner_result.map_err(|e| {
        tracing::error!(module = "H5_API", "with_conn error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })??;

    inner_result.ok_or(StatusCode::UNAUTHORIZED)
}

async fn h5_access_handler(
    Path(token): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let db_manager = state.db_manager.clone();

    let token_row = match validate_h5_token_from_db(&db_manager, &token).await {
        Ok(row) => row,
        Err(status) => {
            return (status, Json(serde_json::json!({
                "error": "Invalid or expired token"
            }))).into_response();
        }
    };

    let conv_id = token_row.conversation_id.clone();
    let db = db_manager.clone();
    let cid = conv_id.clone();

    let result = tokio::task::spawn_blocking(move || {
        db.with_conn(|conn| -> anyhow::Result<serde_json::Value> {
            let conv = crate::db::conversation_repo::get_conversation(conn, &cid)?;
            let msgs = crate::db::message_repo::get_messages_by_conversation(conn, &cid)?;
            Ok(serde_json::json!({
                "success": true,
                "conversation": conv,
                "messages": msgs,
            }))
        })
    })
    .await;

    match result {
        Ok(Ok(Ok(json))) => (StatusCode::OK, Json(json)).into_response(),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "Failed to fetch conversation"
        }))).into_response(),
    }
}

#[derive(Deserialize)]
struct H5ChatRequest {
    message: String,
    model: Option<String>,
}

async fn h5_chat_handler(
    Path(token): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<H5ChatRequest>,
) -> impl IntoResponse {
    let db_manager = state.db_manager.clone();

    let token_row = match validate_h5_token_from_db(&db_manager, &token).await {
        Ok(row) => row,
        Err(status) => {
            return (status, Json(serde_json::json!({
                "error": "Invalid or expired token"
            }))).into_response();
        }
    };

    let conv_id = token_row.conversation_id.clone();
    let message = req.message.clone();
    let model = req.model.clone().unwrap_or_else(|| "claude-sonnet-4-6".to_string());

    tracing::info!(module = "H5_Chat", "token={}, conv_id={}, model={}", token, conv_id, model);

    let native_engine = state.native_engine.clone();
    let rx_opt = {
        let mut engine_guard = native_engine.lock().await;
        if let Some(engine) = engine_guard.as_mut() {
            let chat_req = crate::native_engine::engine_core::ChatRequest {
                conversation_id: conv_id.clone(),
                messages: vec![serde_json::json!({
                    "role": "user",
                    "content": message
                })],
                model: model.clone(),
                system_prompt: None,
                max_tokens: None,
                workspace_path: None,
                temperature: None,
                top_p: None,
                reasoning_mode: None,
            };
            match engine.send_message(chat_req).await {
                Ok(rx) => Some(rx),
                Err(e) => {
                    tracing::error!(module = "H5_Chat", "Engine error: {}", e);
                    None
                }
            }
        } else {
            tracing::error!(module = "H5_Chat", "Engine not initialized");
            None
        }
    };

    let db_for_stream = state.db_manager.clone();
    let stream = async_stream::stream! {
        let mut rx = match rx_opt {
            Some(rx) => rx,
            None => {
                yield Ok::<Event, Infallible>(Event::default().data(
                    serde_json::json!({"type": "error", "error": "Engine not available"}).to_string()
                ));
                return;
            }
        };

        let mut full_text = String::new();
        while let Some(event) = rx.recv().await {
            let event_data = match event {
                crate::native_engine::tool_loop::EngineEvent::MessageStart { model } => {
                    Some(serde_json::json!({
                        "type": "message_start",
                        "model": model,
                    }))
                }
                crate::native_engine::tool_loop::EngineEvent::Text(text) => {
                    full_text.push_str(&text);
                    Some(serde_json::json!({
                        "type": "content_block_delta",
                        "delta": {"type": "text_delta", "text": text},
                    }))
                }
                crate::native_engine::tool_loop::EngineEvent::Thinking(thinking) => {
                    Some(serde_json::json!({
                        "type": "thinking",
                        "thinking": thinking,
                    }))
                }
                crate::native_engine::tool_loop::EngineEvent::ToolUseStart { tool_use_id, tool_name, tool_input, .. } => {
                    Some(serde_json::json!({
                        "type": "tool_use_start",
                        "tool_use_id": tool_use_id,
                        "tool_name": tool_name,
                        "tool_input": tool_input,
                    }))
                }
                crate::native_engine::tool_loop::EngineEvent::ToolUseDone { tool_use_id, tool_name, tool_input, output, is_error } => {
                    Some(serde_json::json!({
                        "type": "tool_use_done",
                        "tool_use_id": tool_use_id,
                        "tool_name": tool_name,
                        "tool_input": tool_input,
                        "output": output,
                        "is_error": is_error,
                    }))
                }
                crate::native_engine::tool_loop::EngineEvent::MessageStop { full_text: _ft, stop_reason } => {
                    Some(serde_json::json!({
                        "type": "message_stop",
                        "stop_reason": stop_reason,
                    }))
                }
                crate::native_engine::tool_loop::EngineEvent::Error(error) => {
                    tracing::error!(module = "H5_Chat", "Engine error event: {}", error);
                    Some(serde_json::json!({
                        "type": "error",
                        "error": error,
                    }))
                }
                crate::native_engine::tool_loop::EngineEvent::Usage(usage) => {
                    Some(serde_json::json!({
                        "type": "usage",
                        "usage": usage,
                    }))
                }
                crate::native_engine::tool_loop::EngineEvent::ToolPermission { request_id, tool_use_id, tool_name, input } => {
                    Some(serde_json::json!({
                        "type": "tool_permission",
                        "request_id": request_id,
                        "tool_use_id": tool_use_id,
                        "tool_name": tool_name,
                        "input": input,
                    }))
                }
                crate::native_engine::tool_loop::EngineEvent::AskUser { .. } |
                crate::native_engine::tool_loop::EngineEvent::MessageDelta { .. } |
                crate::native_engine::tool_loop::EngineEvent::ToolArgDelta { .. } |
                crate::native_engine::tool_loop::EngineEvent::PipelineToolResult { .. } => None,
                crate::native_engine::tool_loop::EngineEvent::BudgetWarning { message, usage, limit } => {
                    Some(serde_json::json!({
                        "type": "budget_warning",
                        "message": message,
                        "usage": usage,
                        "limit": limit,
                    }))
                }
            };

            if let Some(data) = event_data {
                yield Ok::<Event, Infallible>(Event::default().data(data.to_string()));
            }
        }

        let db = db_for_stream.clone();
        let conv_id_save = conv_id.clone();
        let full_text_save = full_text.clone();
        let model_save = model.clone();
        let _ = tokio::task::spawn_blocking(move || {
            db.with_conn(|conn| {
                let now = chrono::Utc::now().to_rfc3339();
                let sort_order = crate::db::message_repo::get_messages_by_conversation(conn, &conv_id_save)
                    .unwrap_or_default()
                    .len() as i64;
                let _ = crate::db::message_repo::insert_message(
                    conn,
                    &uuid::Uuid::new_v4().to_string(),
                    &conv_id_save,
                    "assistant",
                    &full_text_save,
                    None,
                    &now,
                    false,
                    sort_order,
                );
                let count = crate::db::message_repo::get_messages_by_conversation(conn, &conv_id_save)
                    .unwrap_or_default()
                    .len() as i64;
                let _ = conn.execute(
                    "UPDATE conversations SET message_count = ?1, updated_at = ?2 WHERE id = ?3",
                    rusqlite::params![count, &now, &conv_id_save],
                );
                let _ = conn.execute(
                    "UPDATE conversations SET model = ?1 WHERE id = ?2 AND model IS NULL",
                    rusqlite::params![model_save, conv_id_save],
                );
                tracing::error!(module = "H5_Chat", "Assistant message saved: conv={}, len={}", conv_id_save, full_text_save.len());
                Ok::<(), anyhow::Error>(())
            })
        }).await;
    };

    Sse::new(stream)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text("keep-alive-text"),
        )
        .into_response()
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/h5/access/{token}", get(h5_access_handler))
        .route("/api/h5/access/{token}/chat", post(h5_chat_handler))
}
