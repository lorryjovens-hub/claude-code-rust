use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
    Json, Router,
};
use axum::response::IntoResponse;
use serde::{Deserialize};
use serde_json::{self, json};
use std::convert::Infallible;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::time::Duration;
use tracing;

use super::super::{AppState, set_sse_content_type};

static FEISHU_BRIDGE: LazyLock<Arc<crate::im_integration::FeishuBridgeManager>> =
    LazyLock::new(|| Arc::new(crate::im_integration::FeishuBridgeManager::new()));

#[derive(Deserialize)]
struct ImSendRequest {
    platform: String,
    chat_id: String,
    message: String,
}

async fn im_webhook_handler(
    Path(platform): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    tracing::error!(module = "IM_Webhook", "Received webhook for platform: {}", platform);

    let db_manager = state.db_manager.clone();
    let im_manager = Arc::new(crate::im_integration::ImIntegrationManager::new(db_manager));

    match im_manager.receive_message(&platform, payload).await {
        Ok(msg) => {
            tracing::error!(module = "IM_Webhook", "Parsed message from {}: chat_id={}, content_len={}", msg.platform, msg.chat_id, msg.content.len());
            (StatusCode::OK, Json(json!({"status": "ok", "message": "received"})))
        }
        Err(e) => {
            tracing::error!(module = "IM_Webhook", "Failed to parse message: {}", e);
            (StatusCode::BAD_REQUEST, Json(json!({"status": "error", "message": e.to_string()})))
        }
    }
}

async fn im_connections_list(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let db_manager = state.db_manager.clone();
    let im_manager = Arc::new(crate::im_integration::ImIntegrationManager::new(db_manager));

    let connections: Vec<crate::im_integration::ImConnectionInfo> = im_manager.list_connections().await;
    let result: Vec<serde_json::Value> = connections.iter().map(|c| {
        json!({
            "id": c.id,
            "platform": c.platform,
            "status": c.status,
            "config": {
                "webhook_url": c.config.webhook_url,
                "has_token": !c.config.token.is_empty(),
            },
            "created_at": c.created_at,
            "updated_at": c.updated_at,
        })
    }).collect();

    Json(json!({"connections": result}))
}

async fn im_send_handler(
    State(state): State<AppState>,
    Json(req): Json<ImSendRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    tracing::error!(module = "IM_Send", "platform={}, chat_id={}", req.platform, req.chat_id);

    let db_manager = state.db_manager.clone();
    let im_manager = Arc::new(crate::im_integration::ImIntegrationManager::new(db_manager));

    match im_manager.send_message(&req.platform, &req.chat_id, &req.message).await {
        Ok(()) => {
            (StatusCode::OK, Json(json!({"status": "ok"})))
        }
        Err(e) => {
            tracing::error!(module = "IM_Send", "Failed: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"status": "error", "message": e.to_string()})))
        }
    }
}

#[derive(Deserialize)]
struct ImStatusQuery {
    platform: Option<String>,
}

async fn im_status_handler(
    State(state): State<AppState>,
    Query(query): Query<ImStatusQuery>,
) -> Json<serde_json::Value> {
    let db_manager = state.db_manager.clone();
    let im_manager = Arc::new(crate::im_integration::ImIntegrationManager::new(db_manager));

    if let Some(platform) = query.platform {
        match im_manager.get_connection_status(&platform).await {
            Ok(status) => Json(json!({
                "status": "ok",
                "platform": status.platform,
                "connected": status.connected,
                "connection_status": status.status,
                "last_connected_at": status.last_connected_at,
            })),
            Err(e) => Json(json!({
                "status": "error",
                "message": e.to_string(),
            })),
        }
    } else {
        match im_manager.get_all_connection_status().await {
            Ok(statuses) => {
                let results: Vec<serde_json::Value> = statuses.iter().map(|s| {
                    json!({
                        "platform": s.platform,
                        "connected": s.connected,
                        "connection_status": s.status,
                        "last_connected_at": s.last_connected_at,
                    })
                }).collect();
                Json(json!({
                    "status": "ok",
                    "connections": results,
                }))
            }
            Err(e) => Json(json!({
                "status": "error",
                "message": e.to_string(),
            })),
        }
    }
}

#[derive(Deserialize)]
struct ImStatsQuery {
    platform: Option<String>,
}

async fn im_stats_handler(
    State(state): State<AppState>,
    Query(query): Query<ImStatsQuery>,
) -> Json<serde_json::Value> {
    let db_manager = state.db_manager.clone();
    let im_manager = Arc::new(crate::im_integration::ImIntegrationManager::new(db_manager));

    match im_manager.get_message_stats(query.platform.as_deref()).await {
        Ok(stats) => Json(json!({
            "status": "ok",
            "platform": stats.platform,
            "total_messages": stats.total_messages,
            "total_sessions": stats.total_sessions,
            "active_today": stats.active_today,
            "avg_response_time_ms": stats.avg_response_time_ms,
        })),
        Err(e) => Json(json!({
            "status": "error",
            "message": e.to_string(),
        })),
    }
}

#[derive(Deserialize)]
struct ImPermissionsQuery {
    platform: String,
}

async fn im_permissions_handler(
    State(state): State<AppState>,
    Query(query): Query<ImPermissionsQuery>,
) -> Json<serde_json::Value> {
    let db_manager = state.db_manager.clone();
    let im_manager = Arc::new(crate::im_integration::ImIntegrationManager::new(db_manager));

    let mode = im_manager.get_permission_mode(&query.platform).await;

    match im_manager.get_permissions(&query.platform).await {
        Ok(permissions) => {
            let perms: Vec<serde_json::Value> = permissions.iter().map(|p| {
                json!({
                    "id": p.id,
                    "platform": p.platform,
                    "user_id": p.user_id,
                    "permission_mode": p.permission_mode.as_str(),
                    "is_allowed": p.is_allowed,
                    "paired_code": p.paired_code,
                    "created_at": p.created_at.to_rfc3339(),
                    "updated_at": p.updated_at.to_rfc3339(),
                })
            }).collect();
            Json(json!({
                "status": "ok",
                "platform": query.platform,
                "permission_mode": mode.as_str(),
                "permissions": perms,
            }))
        }
        Err(e) => Json(json!({
            "status": "error",
            "message": e.to_string(),
        })),
    }
}

#[derive(Deserialize)]
struct ImLogsQuery {
    platform: Option<String>,
}

async fn im_logs_handler(
    State(state): State<AppState>,
    Query(query): Query<ImLogsQuery>,
) -> Json<serde_json::Value> {
    let db_manager = state.db_manager.clone();
    let im_manager = Arc::new(crate::im_integration::ImIntegrationManager::new(db_manager));

    match im_manager.get_error_logs(query.platform.as_deref()).await {
        Ok(logs) => {
            let log_entries: Vec<serde_json::Value> = logs.iter().map(|l| {
                json!({
                    "id": l.id,
                    "platform": l.platform,
                    "error_type": l.error_type,
                    "error_message": l.error_message,
                    "stack_trace": l.stack_trace,
                    "created_at": l.created_at,
                })
            }).collect();
            Json(json!({
                "status": "ok",
                "logs": log_entries,
            }))
        }
        Err(e) => Json(json!({
            "status": "error",
            "message": e.to_string(),
        })),
    }
}

async fn im_status_stream_handler(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let db_manager = state.db_manager.clone();
    let im_manager = Arc::new(crate::im_integration::ImIntegrationManager::new(db_manager));

    let stream = async_stream::stream! {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;

            match im_manager.get_all_connection_status().await {
                Ok(statuses) => {
                    let data = serde_json::json!({
                        "type": "connection_status",
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                        "connections": statuses.iter().map(|s| {
                            json!({
                                "platform": s.platform,
                                "connected": s.connected,
                                "status": s.status,
                                "last_connected_at": s.last_connected_at,
                            })
                        }).collect::<Vec<_>>(),
                    });
                    yield Ok::<Event, Infallible>(Event::default().data(data.to_string()));
                }
                Err(e) => {
                    let data = serde_json::json!({
                        "type": "error",
                        "message": e.to_string(),
                    });
                    yield Ok::<Event, Infallible>(Event::default().data(data.to_string()));
                }
            }
        }
    };

    let mut response = Sse::new(stream).keep_alive(KeepAlive::default()).into_response();
    set_sse_content_type(&mut response);
    Ok(response)
}

async fn feishu_bridge_config_get() -> Json<serde_json::Value> {
    let config = FEISHU_BRIDGE.get_config().await;
    Json(json!({
        "status": "ok",
        "config": {
            "enabled": config.enabled,
            "app_id": config.app_id,
            "has_app_secret": config.app_secret.is_some(),
            "has_verification_token": config.verification_token.is_some(),
            "has_encrypt_key": config.encrypt_key.is_some(),
            "enable_streaming_cards": config.enable_streaming_cards,
            "enable_group_sessions": config.enable_group_sessions,
            "enable_document_export": config.enable_document_export,
            "max_session_age_hours": config.max_session_age_hours,
            "auto_reconnect": config.auto_reconnect,
        }
    }))
}

#[derive(Deserialize)]
struct BridgeConfigUpdateRequest {
    enabled: Option<bool>,
    app_id: Option<String>,
    app_secret: Option<String>,
    verification_token: Option<String>,
    encrypt_key: Option<String>,
    enable_streaming_cards: Option<bool>,
    enable_group_sessions: Option<bool>,
    enable_document_export: Option<bool>,
    max_session_age_hours: Option<u32>,
    auto_reconnect: Option<bool>,
}

async fn feishu_bridge_config_update(Json(req): Json<BridgeConfigUpdateRequest>) -> Json<serde_json::Value> {
    let mut config = FEISHU_BRIDGE.get_config().await;
    if let Some(v) = req.enabled { config.enabled = v; }
    if let Some(v) = req.app_id { config.app_id = Some(v); }
    if let Some(v) = req.app_secret { config.app_secret = Some(v); }
    if let Some(v) = req.verification_token { config.verification_token = Some(v); }
    if let Some(v) = req.encrypt_key { config.encrypt_key = Some(v); }
    if let Some(v) = req.enable_streaming_cards { config.enable_streaming_cards = v; }
    if let Some(v) = req.enable_group_sessions { config.enable_group_sessions = v; }
    if let Some(v) = req.enable_document_export { config.enable_document_export = v; }
    if let Some(v) = req.max_session_age_hours { config.max_session_age_hours = v; }
    if let Some(v) = req.auto_reconnect { config.auto_reconnect = v; }
    FEISHU_BRIDGE.update_config(config).await;
    Json(json!({ "status": "ok", "message": "Bridge config updated" }))
}

async fn feishu_bridge_status() -> Json<serde_json::Value> {
    let status = FEISHU_BRIDGE.get_status().await;
    Json(json!({
        "status": "ok",
        "bridge": {
            "running": status.running,
            "process_pid": status.process_pid,
            "active_sessions": status.active_sessions,
            "uptime_seconds": status.uptime_seconds,
            "version": status.version,
            "last_error": status.last_error,
        }
    }))
}

async fn feishu_bridge_start() -> Json<serde_json::Value> {
    match FEISHU_BRIDGE.start_bridge(None).await {
        Ok(result) => Json(json!({
            "status": if result.success { "ok" } else { "error" },
            "message": result.message,
            "data": result.data,
        })),
        Err(e) => Json(json!({
            "status": "error",
            "message": e.to_string(),
        })),
    }
}

async fn feishu_bridge_stop() -> Json<serde_json::Value> {
    match FEISHU_BRIDGE.stop_bridge().await {
        Ok(result) => Json(json!({
            "status": if result.success { "ok" } else { "error" },
            "message": result.message,
        })),
        Err(e) => Json(json!({
            "status": "error",
            "message": e.to_string(),
        })),
    }
}

async fn feishu_bridge_restart() -> Json<serde_json::Value> {
    match FEISHU_BRIDGE.restart_bridge().await {
        Ok(result) => Json(json!({
            "status": if result.success { "ok" } else { "error" },
            "message": result.message,
            "data": result.data,
        })),
        Err(e) => Json(json!({
            "status": "error",
            "message": e.to_string(),
        })),
    }
}

async fn feishu_bridge_install() -> Json<serde_json::Value> {
    match FEISHU_BRIDGE.install_bridge().await {
        Ok(()) => Json(json!({
            "status": "ok",
            "message": "Bridge package cached successfully (npx -y lark-channel-bridge@latest)"
        })),
        Err(e) => Json(json!({
            "status": "error",
            "message": e.to_string(),
        })),
    }
}

async fn feishu_bridge_check() -> Json<serde_json::Value> {
    let installed = FEISHU_BRIDGE.check_bridge_installed().await;
    let status = FEISHU_BRIDGE.get_status().await;
    Json(json!({
        "status": "ok",
        "installed": installed,
        "version": status.version,
        "running": status.running,
        "pid": status.process_pid,
    }))
}

async fn feishu_bridge_sessions_list() -> Json<serde_json::Value> {
    let sessions = FEISHU_BRIDGE.list_sessions().await;
    let items: Vec<serde_json::Value> = sessions.iter().map(|s| {
        json!({
            "session_id": s.session_id,
            "chat_id": s.chat_id,
            "title": s.title,
            "created_at": s.created_at,
            "last_active_at": s.last_active_at,
            "message_count": s.message_count,
            "status": s.status,
        })
    }).collect();
    Json(json!({
        "status": "ok",
        "sessions": items,
        "total": items.len(),
    }))
}

async fn feishu_bridge_sessions_cleanup() -> Json<serde_json::Value> {
    let count = FEISHU_BRIDGE.cleanup_expired_sessions().await;
    Json(json!({
        "status": "ok",
        "cleaned": count,
        "message": format!("Cleaned up {} expired sessions", count),
    }))
}

#[derive(Deserialize)]
struct BridgeCommandRequest {
    command: String,
    args: Option<Vec<String>>,
    chat_id: Option<String>,
}

async fn feishu_bridge_command(Json(req): Json<BridgeCommandRequest>) -> Json<serde_json::Value> {
    let args = req.args.unwrap_or_default();
    let chat_id = req.chat_id.unwrap_or_else(|| "default".to_string());
    let result = FEISHU_BRIDGE.handle_bridge_command(&req.command, &args, &chat_id).await;
    Json(json!({
        "status": if result.success { "ok" } else { "error" },
        "message": result.message,
        "data": result.data,
    }))
}

async fn feishu_bridge_features() -> Json<serde_json::Value> {
    let config = FEISHU_BRIDGE.get_config().await;
    Json(json!({
        "status": "ok",
        "features": {
            "streaming_cards": {
                "enabled": config.enable_streaming_cards,
                "description": "飞书流式卡片：实时推送 AI 输出，支持工具调用进度显示"
            },
            "group_sessions": {
                "enabled": config.enable_group_sessions,
                "description": "一事一群：每个 AI 会话自动创建独立飞书群，实现对话隔离"
            },
            "document_export": {
                "enabled": config.enable_document_export,
                "description": "飞书文档导出：将 AI 对话内容导出为飞书文档"
            },
            "slash_commands": {
                "enabled": true,
                "commands": [
                    { "cmd": "/help", "desc": "显示帮助信息" },
                    { "cmd": "/new [标题]", "desc": "创建新会话" },
                    { "cmd": "/reconnect", "desc": "重新连接 WebSocket" },
                    { "cmd": "/status", "desc": "查看连接状态" },
                    { "cmd": "/doc [标题]", "desc": "导出对话为飞书文档" },
                    { "cmd": "/code", "desc": "查看代码库结构" }
                ]
            },
            "bridge_daemon": {
                "enabled": config.enabled,
                "description": "lark-channel-bridge 守护进程：管理飞书 WebSocket 连接和消息路由"
            }
        }
    }))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/im/webhook/{platform}", post(im_webhook_handler))
        .route("/api/im/connections", get(im_connections_list))
        .route("/api/im/send", post(im_send_handler))
        .route("/api/im/status", get(im_status_handler))
        .route("/api/im/stats", get(im_stats_handler))
        .route("/api/im/permissions", get(im_permissions_handler))
        .route("/api/im/logs", get(im_logs_handler))
        .route("/api/im/status/stream", get(im_status_stream_handler))
        .route("/api/feishu-bridge/config", get(feishu_bridge_config_get).post(feishu_bridge_config_update))
        .route("/api/feishu-bridge/status", get(feishu_bridge_status))
        .route("/api/feishu-bridge/start", post(feishu_bridge_start))
        .route("/api/feishu-bridge/stop", post(feishu_bridge_stop))
        .route("/api/feishu-bridge/restart", post(feishu_bridge_restart))
        .route("/api/feishu-bridge/install", post(feishu_bridge_install))
        .route("/api/feishu-bridge/check", get(feishu_bridge_check))
        .route("/api/feishu-bridge/sessions", get(feishu_bridge_sessions_list))
        .route("/api/feishu-bridge/sessions/cleanup", post(feishu_bridge_sessions_cleanup))
        .route("/api/feishu-bridge/command", post(feishu_bridge_command))
        .route("/api/feishu-bridge/features", get(feishu_bridge_features))
}
