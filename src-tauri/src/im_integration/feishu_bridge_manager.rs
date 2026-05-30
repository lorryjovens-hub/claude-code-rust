use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::{Mutex, RwLock};

use crate::im_integration::adapters::feishu_adapter::FeishuBridgeSession;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    pub enabled: bool,
    pub app_id: Option<String>,
    pub app_secret: Option<String>,
    pub verification_token: Option<String>,
    pub encrypt_key: Option<String>,
    pub enable_streaming_cards: bool,
    pub enable_group_sessions: bool,
    pub enable_document_export: bool,
    pub max_session_age_hours: u32,
    pub auto_reconnect: bool,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            app_id: None,
            app_secret: None,
            verification_token: None,
            encrypt_key: None,
            enable_streaming_cards: true,
            enable_group_sessions: true,
            enable_document_export: false,
            max_session_age_hours: 24,
            auto_reconnect: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeStatus {
    pub running: bool,
    pub process_pid: Option<u32>,
    pub active_sessions: usize,
    pub connected_platforms: Vec<String>,
    pub uptime_seconds: Option<u64>,
    pub version: Option<String>,
    pub last_error: Option<String>,
}

impl Default for BridgeStatus {
    fn default() -> Self {
        Self {
            running: false,
            process_pid: None,
            active_sessions: 0,
            connected_platforms: vec![],
            uptime_seconds: None,
            version: None,
            last_error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeSessionInfo {
    pub session_id: String,
    pub chat_id: String,
    pub title: String,
    pub created_at: String,
    pub last_active_at: String,
    pub message_count: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeCommandResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// 飞书 Bridge 管理器
/// 负责管理 lark-channel-bridge 进程生命周期，
/// 会话-群组映射、以及增强的飞书集成功能
pub struct FeishuBridgeManager {
    config: RwLock<BridgeConfig>,
    status: RwLock<BridgeStatus>,
    sessions: RwLock<HashMap<String, BridgeSessionInfo>>,
    bridge_installed: RwLock<bool>,
    started_at: Mutex<Option<chrono::DateTime<chrono::Utc>>>,
    last_error: RwLock<Option<String>>,
}

impl FeishuBridgeManager {
    pub fn new() -> Self {
        Self {
            config: RwLock::new(BridgeConfig::default()),
            status: RwLock::new(BridgeStatus::default()),
            sessions: RwLock::new(HashMap::new()),
            bridge_installed: RwLock::new(false),
            started_at: Mutex::new(None),
            last_error: RwLock::new(None),
        }
    }

    pub async fn get_config(&self) -> BridgeConfig {
        self.config.read().await.clone()
    }

    pub async fn update_config(&self, config: BridgeConfig) {
        *self.config.write().await = config;
    }

    pub async fn get_status(&self) -> BridgeStatus {
        self.status.read().await.clone()
    }

    /// 检测 lark-channel-bridge 是否已安装
    pub async fn check_bridge_installed(&self) -> bool {
        let result = Command::new("npx")
            .args(["-y", "lark-channel-bridge@latest", "--version"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await;

        match result {
            Ok(output) => {
                let installed = output.status.success();
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if installed {
                    let mut status = self.status.write().await;
                    status.version = Some(version);
                }
                *self.bridge_installed.write().await = installed;
                installed
            }
            Err(_) => {
                *self.bridge_installed.write().await = false;
                false
            }
        }
    }

    /// 安装 lark-channel-bridge
    pub async fn install_bridge(&self) -> Result<()> {
        let output = Command::new("npx")
            .args(["-y", "lark-channel-bridge@latest", "--version"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| anyhow!("Failed to run npx: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Bridge install check failed: {}", stderr));
        }

        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        *self.bridge_installed.write().await = true;
        {
            let mut status = self.status.write().await;
            status.version = Some(version);
        }
        Ok(())
    }

    /// 启动 bridge 进程
    pub async fn start_bridge(&self, config: Option<BridgeConfig>) -> Result<BridgeCommandResult> {
        if let Some(cfg) = config {
            self.update_config(cfg).await;
        }

        let current_config = self.get_config().await;
        if !current_config.enabled {
            return Ok(BridgeCommandResult {
                success: false,
                message: "Bridge is not enabled. Please configure and enable it first.".to_string(),
                data: None,
            });
        }

        let installed = self.check_bridge_installed().await;
        if !installed {
            return Ok(BridgeCommandResult {
                success: false,
                message: "lark-channel-bridge not installed. Run install first.".to_string(),
                data: None,
            });
        }

        let cmd_args = vec!["-y", "lark-channel-bridge@latest", "start"];
        let mut env_vars: Vec<(String, String)> = Vec::new();

        if let Some(ref app_id) = current_config.app_id {
            env_vars.push(("FEISHU_APP_ID".to_string(), app_id.clone()));
        }
        if let Some(ref app_secret) = current_config.app_secret {
            env_vars.push(("FEISHU_APP_SECRET".to_string(), app_secret.clone()));
        }
        if let Some(ref token) = current_config.verification_token {
            env_vars.push(("FEISHU_VERIFICATION_TOKEN".to_string(), token.clone()));
        }
        if let Some(ref key) = current_config.encrypt_key {
            env_vars.push(("FEISHU_ENCRYPT_KEY".to_string(), key.clone()));
        }

        let mut child = Command::new("npx")
            .args(&cmd_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(env_vars)
            .spawn()
            .map_err(|e| anyhow!("Failed to start bridge process: {}", e))?;

        let pid = child.id();

        tokio::spawn(async move {
            let status_result = child.wait().await;
            match status_result {
                Ok(status) => {
                    tracing::info!(module = "FeishuBridge", "Bridge process exited with status: {:?}", status);
                }
                Err(e) => {
                    tracing::error!(module = "FeishuBridge", "Bridge process error: {}", e);
                }
            }
        });

        {
            let mut status = self.status.write().await;
            status.running = true;
            status.process_pid = pid;
            status.last_error = None;
        }

        *self.started_at.lock().await = Some(chrono::Utc::now());

        Ok(BridgeCommandResult {
            success: true,
            message: format!("Bridge started successfully (PID: {})", pid.unwrap_or(0)),
            data: Some(serde_json::json!({ "pid": pid })),
        })
    }

    /// 停止 bridge 进程
    pub async fn stop_bridge(&self) -> Result<BridgeCommandResult> {
        let pid = {
            let status = self.status.read().await;
            status.process_pid
        };

        if let Some(pid) = pid {
            let _ = Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F", "/T"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .await;
        }

        {
            let mut status = self.status.write().await;
            status.running = false;
            status.process_pid = None;
        }

        Ok(BridgeCommandResult {
            success: true,
            message: "Bridge stopped".to_string(),
            data: None,
        })
    }

    /// 重启 bridge 进程
    pub async fn restart_bridge(&self) -> Result<BridgeCommandResult> {
        self.stop_bridge().await?;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        self.start_bridge(None).await
    }

    /// 注册飞书 Bridge 会话
    pub async fn register_session(
        &self,
        chat_id: &str,
        title: Option<&str>,
    ) -> BridgeSessionInfo {
        let session_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let info = BridgeSessionInfo {
            session_id: session_id.clone(),
            chat_id: chat_id.to_string(),
            title: title.unwrap_or("New Session").to_string(),
            created_at: now.clone(),
            last_active_at: now,
            message_count: 0,
            status: "active".to_string(),
        };
        self.sessions.write().await.insert(chat_id.to_string(), info.clone());
        {
            let mut status = self.status.write().await;
            status.active_sessions = self.sessions.read().await.len();
        }
        info
    }

    /// 获取会话信息
    pub async fn get_session(&self, chat_id: &str) -> Option<BridgeSessionInfo> {
        self.sessions.read().await.get(chat_id).cloned()
    }

    /// 列出所有会话
    pub async fn list_sessions(&self) -> Vec<BridgeSessionInfo> {
        self.sessions.read().await.values().cloned().collect()
    }

    /// 更新会话活跃时间
    pub async fn touch_session(&self, chat_id: &str) {
        if let Some(session) = self.sessions.write().await.get_mut(chat_id) {
            session.last_active_at = chrono::Utc::now().to_rfc3339();
            session.message_count += 1;
        }
    }

    /// 清理过期会话
    pub async fn cleanup_expired_sessions(&self) -> usize {
        let max_age = self.config.read().await.max_session_age_hours;
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(max_age as i64);
        let mut sessions = self.sessions.write().await;

        let expired: Vec<String> = sessions
            .iter()
            .filter(|(_, s)| {
                chrono::DateTime::parse_from_rfc3339(&s.last_active_at)
                    .map(|dt| dt < cutoff)
                    .unwrap_or(true)
            })
            .map(|(k, _)| k.clone())
            .collect();

        let count = expired.len();
        for key in &expired {
            sessions.remove(key);
        }

        {
            let mut status = self.status.write().await;
            status.active_sessions = sessions.len();
        }

        count
    }

    /// 处理来自 FeishuAdapter 的斜杠命令
    pub async fn handle_bridge_command(
        &self,
        cmd: &str,
        _args: &[String],
        _chat_id: &str,
    ) -> BridgeCommandResult {
        match cmd {
            "start" => match self.start_bridge(None).await {
                Ok(result) => result,
                Err(e) => BridgeCommandResult {
                    success: false,
                    message: format!("Failed to start bridge: {}", e),
                    data: None,
                },
            },
            "stop" => match self.stop_bridge().await {
                Ok(result) => result,
                Err(e) => BridgeCommandResult {
                    success: false,
                    message: format!("Failed to stop bridge: {}", e),
                    data: None,
                },
            },
            "restart" => match self.restart_bridge().await {
                Ok(result) => result,
                Err(e) => BridgeCommandResult {
                    success: false,
                    message: format!("Failed to restart bridge: {}", e),
                    data: None,
                },
            },
            "status" => {
                let status = self.get_status().await;
                BridgeCommandResult {
                    success: true,
                    message: format!(
                        "Bridge {} | Sessions: {} | PID: {}",
                        if status.running { "RUNNING" } else { "STOPPED" },
                        status.active_sessions,
                        status.process_pid.map_or("N/A".to_string(), |p| p.to_string())
                    ),
                    data: Some(serde_json::to_value(&status).unwrap_or_default()),
                }
            }
            "install" => match self.install_bridge().await {
                Ok(()) => BridgeCommandResult {
                    success: true,
                    message: "Bridge installed successfully".to_string(),
                    data: None,
                },
                Err(e) => BridgeCommandResult {
                    success: false,
                    message: format!("Install failed: {}", e),
                    data: None,
                },
            }
            _ => BridgeCommandResult {
                success: false,
                message: format!("Unknown bridge command: {}", cmd),
                data: None,
            },
        }
    }

    /// 获取 FeishuBridgeSession 格式的会话信息（兼容 FeishuAdapter 内定义的格式）
    pub async fn get_bridge_sessions(&self) -> Vec<FeishuBridgeSession> {
        self.sessions
            .read()
            .await
            .values()
            .map(|s| FeishuBridgeSession {
                chat_id: s.chat_id.clone(),
                conversation_id: s.session_id.clone(),
                title: s.title.clone(),
                created_at: s.created_at.clone(),
                last_active_at: s.last_active_at.clone(),
                status: s.status.clone(),
            })
            .collect()
    }

    pub fn get_bridge_manager() -> Arc<FeishuBridgeManager> {
        Arc::new(FeishuBridgeManager::new())
    }
}