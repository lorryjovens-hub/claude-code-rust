pub mod adapters;
pub mod feishu_bridge_manager;
pub mod message_router;
pub mod session_manager;
pub mod permission_manager;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

use crate::db::DbManager;

pub use adapters::FeishuAdapter;
pub use feishu_bridge_manager::{
    BridgeCommandResult, BridgeConfig, BridgeSessionInfo, BridgeStatus, FeishuBridgeManager,
};
pub use message_router::{
    ConnectionStatus, MessageRouter, PlatformAdapter,
    UnifiedMessage,
};
pub use permission_manager::{PermissionManager, PermissionMode, UserPermission};
pub use session_manager::SessionManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImPlatform {
    Telegram,
    Feishu,
    WeChat,
    DingTalk,
}

impl ImPlatform {
    pub fn as_str(&self) -> &str {
        match self {
            ImPlatform::Telegram => "telegram",
            ImPlatform::Feishu => "feishu",
            ImPlatform::WeChat => "wechat",
            ImPlatform::DingTalk => "dingtalk",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "telegram" => Some(ImPlatform::Telegram),
            "feishu" => Some(ImPlatform::Feishu),
            "wechat" => Some(ImPlatform::WeChat),
            "dingtalk" => Some(ImPlatform::DingTalk),
            _ => None,
        }
    }

    pub fn all() -> Vec<&'static str> {
        vec!["telegram", "feishu", "wechat", "dingtalk"]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImPlatformConfig {
    pub webhook_url: String,
    pub token: String,
    pub extra: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImConnectionInfo {
    pub id: String,
    pub platform: String,
    pub status: String,
    pub config: ImPlatformConfig,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImMessage {
    pub platform: String,
    pub chat_id: String,
    pub content: String,
    pub sender: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImConnectionStatusResult {
    pub platform: String,
    pub connected: bool,
    pub status: String,
    pub last_connected_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImMessageStatsResult {
    pub platform: Option<String>,
    pub total_messages: u64,
    pub total_sessions: u64,
    pub active_today: u64,
    pub avg_response_time_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImErrorLogInfo {
    pub id: String,
    pub platform: String,
    pub error_type: String,
    pub error_message: Option<String>,
    pub stack_trace: Option<String>,
    pub created_at: String,
}

pub struct ImIntegrationManager {
    db: Arc<DbManager>,
    connections: Mutex<HashMap<String, ImConnectionInfo>>,
    pub message_router: Arc<MessageRouter>,
    pub session_manager: Arc<SessionManager>,
    pub permission_manager: Arc<PermissionManager>,
    pub feishu_bridge: Arc<FeishuBridgeManager>,
    message_channel: Arc<Mutex<Option<mpsc::UnboundedSender<UnifiedMessage>>>>,
}

impl ImIntegrationManager {
    pub fn new(db: Arc<DbManager>) -> Self {
        let message_router = Arc::new(MessageRouter::new());
        let session_manager = Arc::new(SessionManager::new(db.clone()));
        let permission_manager = Arc::new(PermissionManager::new(db.clone()));
        let feishu_bridge = Arc::new(FeishuBridgeManager::new());
        Self {
            db,
            connections: Mutex::new(HashMap::new()),
            message_router,
            session_manager,
            permission_manager,
            feishu_bridge,
            message_channel: Arc::new(Mutex::new(None)),
        }
    }

    /// Set a channel to forward incoming IM messages to the main app
    pub async fn set_message_channel(&self, tx: mpsc::UnboundedSender<UnifiedMessage>) {
        *self.message_channel.lock().await = Some(tx);
    }

    pub async fn initialize(&self) -> Result<()> {
        self.session_manager.initialize().await?;

        let db = self.db.clone();
        let configs: Vec<crate::db::im_config_repo::ImConfigRow> = tokio::task::spawn_blocking(move || {
            db.with_conn(|conn| {
                crate::db::im_config_repo::list_im_configs(conn)
            })?
        })
        .await??;

        let mut connections = self.connections.lock().await;
        for config in configs {
            let platform_config: ImPlatformConfig = serde_json::from_str(&config.config_json)
                .unwrap_or(ImPlatformConfig {
                    webhook_url: String::new(),
                    token: String::new(),
                    extra: None,
                });
            connections.insert(config.platform.clone(), ImConnectionInfo {
                id: config.id,
                platform: config.platform,
                status: config.status,
                config: platform_config,
                created_at: config.created_at,
                updated_at: config.updated_at,
            });
        }

        // Start message dispatch loop in background
        let router = self.message_router.clone();
        let ch = self.message_channel.clone();
        tokio::spawn(async move {
            let mut rx = router.inbound_rx.lock().await;
            while let Some(msg) = rx.recv().await {
                tracing::info!(module = "IM_Dispatch", "Incoming message from {}: chat_id={}, len={}",
                    msg.platform, msg.chat_id, msg.content.len());

                // Forward to registered channel (set by main.rs)
                if let Some(tx) = ch.lock().await.as_ref() {
                    let _ = tx.send(msg);
                }
            }
        });

        Ok(())
    }

    pub async fn connect_platform(
        &self,
        platform: &str,
        config: ImPlatformConfig,
    ) -> Result<ImConnectionInfo> {
        let platform_enum = ImPlatform::from_str(platform)
            .ok_or_else(|| anyhow::anyhow!("Unknown platform: {}", platform))?;

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let config_json = serde_json::to_string(&config)?;

        let validated = self.validate_connection(&platform_enum, &config).await;

        let status = if validated { "connected" } else { "error" };

        let db = self.db.clone();
        let id_clone = id.clone();
        let platform_clone = platform.to_string();
        let config_json_clone = config_json.clone();
        let status_clone = status.to_string();
        let now_clone = now.clone();

        tokio::task::spawn_blocking(move || -> Result<String> {
            let existing: Option<crate::db::im_config_repo::ImConfigRow> = db.with_conn(|conn| {
                crate::db::im_config_repo::get_im_config_by_platform(conn, &platform_clone)
            })??;
            match existing {
                Some(existing_row) => {
                    db.with_conn(|conn| {
                        crate::db::im_config_repo::update_im_config(
                            conn,
                            &existing_row.id,
                            &config_json_clone,
                            &status_clone,
                            &now_clone,
                        )
                    })??;
                    Ok(existing_row.id)
                }
                None => {
                    db.with_conn(|conn| {
                        crate::db::im_config_repo::insert_im_config(
                            conn,
                            &id_clone,
                            &platform_clone,
                            &config_json_clone,
                            &status_clone,
                            &now_clone,
                            &now_clone,
                        )
                    })??;
                    Ok(id_clone)
                }
            }
        })
        .await??;

        let info = ImConnectionInfo {
            id,
            platform: platform.to_string(),
            status: status.to_string(),
            config,
            created_at: now.clone(),
            updated_at: now,
        };

        self.connections.lock().await.insert(platform.to_string(), info.clone());

        Ok(info)
    }

    pub async fn disconnect_platform(&self, platform: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let db = self.db.clone();
        let platform_clone = platform.to_string();
        let now_clone = now.clone();

        tokio::task::spawn_blocking(move || -> Result<()> {
            let config: Option<crate::db::im_config_repo::ImConfigRow> = db.with_conn(|conn| {
                crate::db::im_config_repo::get_im_config_by_platform(conn, &platform_clone)
            })??;
            if let Some(row) = config {
                db.with_conn(|conn| {
                    crate::db::im_config_repo::update_im_config_status(
                        conn,
                        &row.id,
                        "disconnected",
                        &now_clone,
                    )
                })??;
            }
            Ok(())
        })
        .await??;

        if let Some(conn) = self.connections.lock().await.get_mut(platform) {
            conn.status = "disconnected".to_string();
            conn.updated_at = now;
        }

        Ok(())
    }

    pub async fn list_connections(&self) -> Vec<ImConnectionInfo> {
        let connections = self.connections.lock().await;
        connections.values().cloned().collect()
    }

    pub async fn get_connection(&self, platform: &str) -> Option<ImConnectionInfo> {
        self.connections.lock().await.get(platform).cloned()
    }

    pub async fn send_message(
        &self,
        platform: &str,
        chat_id: &str,
        message: &str,
    ) -> Result<()> {
        let connections = self.connections.lock().await;
        let conn = connections.get(platform)
            .ok_or_else(|| anyhow::anyhow!("Platform {} not connected", platform))?
            .clone();
        drop(connections);

        if conn.status != "connected" {
            return Err(anyhow::anyhow!("Platform {} is not connected", platform));
        }

        let platform_enum = ImPlatform::from_str(platform)
            .ok_or_else(|| anyhow::anyhow!("Unknown platform: {}", platform))?;

        match platform_enum {
            ImPlatform::Telegram => {
                self.send_telegram_message(&conn.config, chat_id, message).await
            }
            ImPlatform::Feishu => {
                self.send_feishu_message(&conn.config, chat_id, message).await
            }
            ImPlatform::WeChat => {
                self.send_wechat_message(&conn.config, chat_id, message).await
            }
            ImPlatform::DingTalk => {
                self.send_dingtalk_message(&conn.config, chat_id, message).await
            }
        }
    }

    pub async fn receive_message(&self, platform: &str, payload: serde_json::Value) -> Result<ImMessage> {
        let platform_enum = ImPlatform::from_str(platform)
            .ok_or_else(|| anyhow::anyhow!("Unknown platform: {}", platform))?;

        match platform_enum {
            ImPlatform::Telegram => {
                let chat_id = payload.get("message")
                    .and_then(|m| m.get("chat"))
                    .and_then(|c| c.get("id"))
                    .and_then(|id| id.as_i64())
                    .map(|id| id.to_string())
                    .unwrap_or_default();
                let content = payload.get("message")
                    .and_then(|m| m.get("text"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();
                let sender = payload.get("message")
                    .and_then(|m| m.get("from"))
                    .and_then(|f| f.get("username"))
                    .and_then(|u| u.as_str())
                    .map(|s| s.to_string());
                Ok(ImMessage {
                    platform: platform.to_string(),
                    chat_id,
                    content,
                    sender,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                })
            }
            ImPlatform::Feishu => {
                let event = payload.get("event").unwrap_or(&payload);
                let chat_id = event.get("message")
                    .and_then(|m| m.get("chat_id"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                let content_raw = event.get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("{}");
                let content_obj: serde_json::Value = serde_json::from_str(content_raw).unwrap_or_default();
                let content = content_obj.get("text")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();
                let sender = event.get("sender")
                    .and_then(|s| s.get("sender_id"))
                    .and_then(|id| id.get("user_id"))
                    .and_then(|u| u.as_str())
                    .map(|s| s.to_string());
                Ok(ImMessage {
                    platform: platform.to_string(),
                    chat_id,
                    content,
                    sender,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                })
            }
            ImPlatform::WeChat => {
                let chat_id = payload.get("FromUserName")
                    .and_then(|f| f.as_str())
                    .unwrap_or("")
                    .to_string();
                let content = payload.get("Content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                Ok(ImMessage {
                    platform: platform.to_string(),
                    chat_id,
                    content,
                    sender: None,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                })
            }
            ImPlatform::DingTalk => {
                let chat_id = payload.get("conversationId")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                let content_obj = payload.get("text")
                    .cloned()
                    .unwrap_or_default();
                let content = content_obj.get("content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                let sender = payload.get("senderStaffId")
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string());
                Ok(ImMessage {
                    platform: platform.to_string(),
                    chat_id,
                    content,
                    sender,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                })
            }
        }
    }

    pub async fn generate_qr_code_url(&self, platform: &str) -> Result<String> {
        let platform_enum = ImPlatform::from_str(platform)
            .ok_or_else(|| anyhow::anyhow!("Unknown platform: {}", platform))?;

        match platform_enum {
            ImPlatform::Feishu => {
                let conn = self.connections.lock().await;
                let config = conn.get(platform)
                    .map(|c| c.config.clone())
                    .unwrap_or(ImPlatformConfig {
                        webhook_url: String::new(),
                        token: String::new(),
                        extra: None,
                    });
                drop(conn);
                let (inbound_tx, _inbound_rx) = tokio::sync::mpsc::channel::<crate::im_integration::message_router::UnifiedMessage>(1024);
                let adapter = FeishuAdapter::new(config, inbound_tx);
                adapter.generate_install_qr_url()
            }
            _ => Err(anyhow::anyhow!("QR code generation not supported for platform: {}", platform)),
        }
    }

    pub async fn check_auth_status(&self, platform: &str) -> Result<(bool, String)> {
        let conn = self.connections.lock().await;
        let info = conn.get(platform).cloned();
        drop(conn);

        match info {
            Some(i) => {
                let is_authenticated = i.status == "connected";
                Ok((is_authenticated, i.status))
            }
            None => Ok((false, "not_configured".to_string())),
        }
    }

    pub async fn get_connection_status(&self, platform: &str) -> Result<ImConnectionStatusResult> {
        let conn = self.connections.lock().await;
        let info = conn.get(platform).cloned();
        drop(conn);

        if let Some(adapter) = self.message_router.get_adapter(platform).await {
            let status = adapter.get_status();
            let connected = matches!(status, ConnectionStatus::Connected);
            let status_str = status.as_str().to_string();
            return Ok(ImConnectionStatusResult {
                platform: platform.to_string(),
                connected,
                status: status_str,
                last_connected_at: info.as_ref().map(|i| i.updated_at.clone()),
            });
        }

        match info {
            Some(i) => {
                let connected = i.status == "connected";
                Ok(ImConnectionStatusResult {
                    platform: platform.to_string(),
                    connected,
                    status: i.status.clone(),
                    last_connected_at: Some(i.updated_at),
                })
            }
            None => Ok(ImConnectionStatusResult {
                platform: platform.to_string(),
                connected: false,
                status: "not_configured".to_string(),
                last_connected_at: None,
            }),
        }
    }

    pub async fn get_message_stats(&self, platform: Option<&str>) -> Result<ImMessageStatsResult> {
        let db = self.db.clone();
        let platform_clone = platform.map(|s| s.to_string());

        let stats = tokio::task::spawn_blocking(move || -> Result<ImMessageStatsResult> {
            db.with_conn(|conn| {
                let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

                let total_messages: u64 = if let Some(ref p) = platform_clone {
                    conn.query_row(
                        "SELECT COALESCE(SUM(message_count), 0) FROM im_message_stats WHERE platform = ?1",
                        rusqlite::params![p],
                        |row| row.get(0),
                    ).unwrap_or(0)
                } else {
                    conn.query_row(
                        "SELECT COALESCE(SUM(message_count), 0) FROM im_message_stats",
                        [],
                        |row| row.get(0),
                    ).unwrap_or(0)
                };

                let total_sessions: u64 = if let Some(ref p) = platform_clone {
                    conn.query_row(
                        "SELECT COUNT(*) FROM im_sessions WHERE platform = ?1",
                        rusqlite::params![p],
                        |row| row.get(0),
                    ).unwrap_or(0)
                } else {
                    conn.query_row(
                        "SELECT COUNT(*) FROM im_sessions",
                        [],
                        |row| row.get(0),
                    ).unwrap_or(0)
                };

                let active_today: u64 = if let Some(ref p) = platform_clone {
                    conn.query_row(
                        "SELECT COALESCE(SUM(message_count), 0) FROM im_message_stats WHERE platform = ?1 AND date = ?2",
                        rusqlite::params![p, today],
                        |row| row.get(0),
                    ).unwrap_or(0)
                } else {
                    conn.query_row(
                        "SELECT COALESCE(SUM(message_count), 0) FROM im_message_stats WHERE date = ?1",
                        rusqlite::params![today],
                        |row| row.get(0),
                    ).unwrap_or(0)
                };

                let avg_response: f64 = if let Some(ref p) = platform_clone {
                    conn.query_row(
                        "SELECT COALESCE(AVG(avg_response_time), 0.0) FROM im_message_stats WHERE platform = ?1",
                        rusqlite::params![p],
                        |row| row.get(0),
                    ).unwrap_or(0.0)
                } else {
                    conn.query_row(
                        "SELECT COALESCE(AVG(avg_response_time), 0.0) FROM im_message_stats",
                        [],
                        |row| row.get(0),
                    ).unwrap_or(0.0)
                };

                Ok(ImMessageStatsResult {
                    platform: platform_clone,
                    total_messages,
                    total_sessions,
                    active_today,
                    avg_response_time_ms: avg_response as u64,
                })
            })?
        })
        .await??;

        Ok(stats)
    }

    pub async fn set_permission_mode(&self, platform: &str, mode: PermissionMode) -> Result<()> {
        self.permission_manager.set_permission_mode(platform, mode).await
    }

    pub async fn get_permission_mode(&self, platform: &str) -> PermissionMode {
        self.permission_manager.get_permission_mode(platform).await
    }

    pub async fn generate_pairing_code(&self, platform: &str, user_id: &str) -> Result<String> {
        self.permission_manager.generate_pairing_code(platform, user_id).await
    }

    pub async fn get_pending_pairing_requests(&self, platform: &str) -> Result<Vec<UserPermission>> {
        self.permission_manager.get_pending_pairing_requests(platform).await
    }

    pub async fn approve_pairing_request(&self, platform: &str, user_id: &str) -> Result<()> {
        self.permission_manager.approve_pairing_request(platform, user_id).await
    }

    pub async fn reject_pairing_request(&self, platform: &str, user_id: &str) -> Result<()> {
        self.permission_manager.reject_pairing_request(platform, user_id).await
    }

    pub async fn get_error_logs(&self, platform: Option<&str>) -> Result<Vec<ImErrorLogInfo>> {
        let db = self.db.clone();
        let platform_clone = platform.map(|s| s.to_string());

        let logs = tokio::task::spawn_blocking(move || -> Result<Vec<ImErrorLogInfo>> {
            db.with_conn(|conn| {
                let sql = if platform_clone.is_some() {
                    "SELECT id, platform, error_type, error_message, stack_trace, created_at FROM im_error_logs WHERE platform = ?1 ORDER BY created_at DESC LIMIT 100"
                } else {
                    "SELECT id, platform, error_type, error_message, stack_trace, created_at FROM im_error_logs ORDER BY created_at DESC LIMIT 100"
                };
                let mut stmt = conn.prepare_cached(sql)?;

                let params: Vec<&dyn rusqlite::ToSql> = if let Some(ref p) = platform_clone {
                    vec![p]
                } else {
                    vec![]
                };

                let rows = stmt.query_map(rusqlite::params_from_iter(params), |row| {
                    Ok(ImErrorLogInfo {
                        id: row.get(0)?,
                        platform: row.get(1)?,
                        error_type: row.get(2)?,
                        error_message: row.get(3)?,
                        stack_trace: row.get(4)?,
                        created_at: row.get(5)?,
                    })
                })?;

                let mut result = Vec::new();
                for row in rows {
                    result.push(row?);
                }
                Ok(result)
            })?
        })
        .await??;

        Ok(logs)
    }

    pub async fn connect_websocket_platform(
        &self,
        platform: &str,
        config: ImPlatformConfig,
    ) -> Result<ImConnectionInfo> {
        let platform_enum = ImPlatform::from_str(platform)
            .ok_or_else(|| anyhow::anyhow!("Unknown platform: {}", platform))?;

        let (inbound_tx, _inbound_rx) = tokio::sync::mpsc::channel::<UnifiedMessage>(1024);

        match platform_enum {
            ImPlatform::Feishu => {
                let adapter = Arc::new(FeishuAdapter::new(config.clone(), inbound_tx));
                self.message_router.register_adapter(adapter.clone()).await?;
                adapter.connect().await?;
            }
            _ => {
                return Err(anyhow::anyhow!("WebSocket connection not supported for platform: {}", platform));
            }
        }

        self.connect_platform(platform, config).await
    }

    pub async fn get_all_connection_status(&self) -> Result<Vec<ImConnectionStatusResult>> {
        let mut results = Vec::new();
        let platforms = ImPlatform::all();

        for platform in platforms {
            let status = self.get_connection_status(platform).await?;
            results.push(status);
        }

        Ok(results)
    }

    pub async fn get_permissions(&self, platform: &str) -> Result<Vec<UserPermission>> {
        self.permission_manager.list_permissions(platform).await
    }

    async fn validate_connection(
        &self,
        platform: &ImPlatform,
        config: &ImPlatformConfig,
    ) -> bool {
        let client = reqwest::Client::new();
        match platform {
            ImPlatform::Telegram => {
                let url = format!("https://api.telegram.org/bot{}/getMe", config.token);
                client.get(&url).send().await
                    .map(|r| r.status().is_success())
                    .unwrap_or(false)
            }
            ImPlatform::Feishu => {
                !config.webhook_url.is_empty()
            }
            ImPlatform::WeChat => {
                !config.webhook_url.is_empty()
            }
            ImPlatform::DingTalk => {
                !config.webhook_url.is_empty()
            }
        }
    }

    async fn send_telegram_message(
        &self,
        config: &ImPlatformConfig,
        chat_id: &str,
        message: &str,
    ) -> Result<()> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", config.token);
        let client = reqwest::Client::new();
        let response = client.post(&url)
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "text": message,
            }))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Telegram send failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Telegram API error: {} - {}", status, body));
        }

        Ok(())
    }

    async fn send_feishu_message(
        &self,
        config: &ImPlatformConfig,
        _chat_id: &str,
        message: &str,
    ) -> Result<()> {
        let url = if config.webhook_url.is_empty() {
            return Err(anyhow::anyhow!("Feishu webhook URL not configured"));
        } else {
            config.webhook_url.clone()
        };
        let client = reqwest::Client::new();
        let response = client.post(&url)
            .json(&serde_json::json!({
                "msg_type": "text",
                "content": {
                    "text": message
                }
            }))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Feishu send failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Feishu API error: {} - {}", status, body));
        }

        Ok(())
    }

    async fn send_wechat_message(
        &self,
        config: &ImPlatformConfig,
        _chat_id: &str,
        message: &str,
    ) -> Result<()> {
        let url = if config.webhook_url.is_empty() {
            return Err(anyhow::anyhow!("WeChat webhook URL not configured"));
        } else {
            config.webhook_url.clone()
        };
        let client = reqwest::Client::new();
        let response = client.post(&url)
            .json(&serde_json::json!({
                "msgtype": "text",
                "text": {
                    "content": message
                }
            }))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("WeChat send failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("WeChat API error: {} - {}", status, body));
        }

        Ok(())
    }

    async fn send_dingtalk_message(
        &self,
        config: &ImPlatformConfig,
        _chat_id: &str,
        message: &str,
    ) -> Result<()> {
        let url = if config.webhook_url.is_empty() {
            return Err(anyhow::anyhow!("DingTalk webhook URL not configured"));
        } else {
            config.webhook_url.clone()
        };
        let client = reqwest::Client::new();
        let response = client.post(&url)
            .json(&serde_json::json!({
                "msgtype": "text",
                "text": {
                    "content": message
                }
            }))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("DingTalk send failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("DingTalk API error: {} - {}", status, body));
        }

        Ok(())
    }
}
