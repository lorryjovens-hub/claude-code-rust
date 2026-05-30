use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::{interval, sleep};
use tokio_tungstenite::{
    connect_async,
    tungstenite::protocol::Message as WsMessage,
    MaybeTlsStream, WebSocketStream,
};

use crate::im_integration::message_router::{
    ConnectionStatus, MessageHandler, MessageType, PlatformAdapter, UnifiedMessage,
};
use crate::im_integration::ImPlatformConfig;

/// 斜杠命令定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuSlashCommand {
    pub command: String,
    pub args: Vec<String>,
    pub raw: String,
}

impl FeishuSlashCommand {
    pub fn parse(text: &str) -> Option<Self> {
        let trimmed = text.trim();
        if !trimmed.starts_with('/') {
            return None;
        }
        let parts: Vec<&str> = trimmed[1..].split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }
        let command = parts[0].to_lowercase();
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
        Some(Self {
            command,
            args,
            raw: trimmed.to_string(),
        })
    }

    pub fn is_slash_command(text: &str) -> bool {
        text.trim().starts_with('/')
    }
}

/// 飞书 Bridge 会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeishuBridgeSession {
    pub chat_id: String,
    pub conversation_id: String,
    pub title: String,
    pub created_at: String,
    pub last_active_at: String,
    pub status: String,
}

/// 流式卡片构建器
pub struct StreamingCardBuilder {
    pub message_id: Option<String>,
}

impl StreamingCardBuilder {
    pub fn new() -> Self {
        Self { message_id: None }
    }

    pub fn build_start_card(content: &str, show_progress: bool) -> String {
        let mut elements = vec![
            json!({ "tag": "markdown", "content": content }),
        ];
        if show_progress {
            elements.push(json!({
                "tag": "note",
                "elements": [{ "tag": "plain_text", "content": "⚡ 流式输出中..." }]
            }));
        }
        serde_json::to_string(&json!({
            "config": { "wide_screen_mode": true, "enable_forward": true },
            "header": {
                "title": { "tag": "plain_text", "content": "🤖 Claude" },
                "template": "blue"
            },
            "elements": elements
        })).unwrap_or_default()
    }

    pub fn build_update_card(content: &str, pending_tools: &[String]) -> String {
        let mut elements = vec![
            json!({ "tag": "markdown", "content": content }),
        ];
        if !pending_tools.is_empty() {
            let tool_list = pending_tools
                .iter()
                .map(|t| format!("• {}", t))
                .collect::<Vec<_>>()
                .join("\n");
            elements.push(json!({
                "tag": "hr"
            }));
            elements.push(json!({
                "tag": "note",
                "elements": [{ "tag": "plain_text", "content": format!("🔧 工具调用中:\n{}", tool_list) }]
            }));
        }
        elements.push(json!({
            "tag": "note",
            "elements": [{ "tag": "plain_text", "content": "⚡ 流式输出中..." }]
        }));
        serde_json::to_string(&json!({
            "config": { "wide_screen_mode": true, "enable_forward": true },
            "header": {
                "title": { "tag": "plain_text", "content": "🤖 Claude" },
                "template": "blue"
            },
            "elements": elements
        })).unwrap_or_default()
    }

    pub fn build_done_card(content: &str, model_info: Option<&str>) -> String {
        let mut elements = vec![
            json!({ "tag": "markdown", "content": content }),
        ];
        elements.push(json!({ "tag": "hr" }));
        let note_content = if let Some(model) = model_info {
            format!("✅ 完成 • Model: {}", model)
        } else {
            "✅ 完成".to_string()
        };
        elements.push(json!({
            "tag": "note",
            "elements": [{ "tag": "plain_text", "content": note_content }]
        }));
        serde_json::to_string(&json!({
            "config": { "wide_screen_mode": true, "enable_forward": true },
            "header": {
                "title": { "tag": "plain_text", "content": "🤖 Claude" },
                "template": "blue"
            },
            "elements": elements
        })).unwrap_or_default()
    }

    pub fn build_error_card(error: &str) -> String {
        serde_json::to_string(&json!({
            "config": { "wide_screen_mode": true },
            "header": {
                "title": { "tag": "plain_text", "content": "❌ 错误" },
                "template": "red"
            },
            "elements": [
                { "tag": "markdown", "content": error },
                {
                    "tag": "note",
                    "elements": [{ "tag": "plain_text", "content": "请重试或使用 /help 查看帮助" }]
                }
            ]
        })).unwrap_or_default()
    }

    pub fn build_help_card() -> String {
        serde_json::to_string(&json!({
            "config": { "wide_screen_mode": true },
            "header": {
                "title": { "tag": "plain_text", "content": "📖 飞书 Bridge 命令帮助" },
                "template": "wathet"
            },
            "elements": [
                {
                    "tag": "markdown",
                    "content": "**可用命令：**\n\n`/help` — 显示此帮助信息\n`/new [标题]` — 创建新的会话（一事一群）\n`/reconnect` — 重新连接 WebSocket\n`/status` — 查看当前连接状态\n`/doc [标题]` — 将对话内容导出为飞书文档\n`/code` — 查看当前代码库结构\n\n**直接发送消息即可与 Claude 对话**\n\n支持文本、图片、语音输入"
                },
                { "tag": "hr" },
                {
                    "tag": "action",
                    "actions": [
                        {
                            "tag": "button",
                            "text": { "tag": "plain_text", "content": "新建会话" },
                            "type": "primary",
                            "value": { "cmd": "/new" }
                        },
                        {
                            "tag": "button",
                            "text": { "tag": "plain_text", "content": "查看状态" },
                            "type": "default",
                            "value": { "cmd": "/status" }
                        },
                        {
                            "tag": "button",
                            "text": { "tag": "plain_text", "content": "重连" },
                            "type": "danger",
                            "value": { "cmd": "/reconnect" }
                        }
                    ]
                }
            ]
        })).unwrap_or_default()
    }

    pub fn build_status_card(
        connected: bool,
        session_count: usize,
        uptime: Option<&str>,
        model: Option<&str>,
    ) -> String {
        let status_icon = if connected { "🟢" } else { "🔴" };
        let status_text = if connected { "已连接" } else { "未连接" };
        let model_text = model.unwrap_or("未配置");
        let uptime_text = uptime.unwrap_or("N/A");

        serde_json::to_string(&json!({
            "config": { "wide_screen_mode": true },
            "header": {
                "title": { "tag": "plain_text", "content": format!("{} 飞书 Bridge 状态", status_icon) },
                "template": if connected { "green" } else { "red" }
            },
            "elements": [
                {
                    "tag": "markdown",
                    "content": format!(
                        "**连接状态**: {}\n**活跃会话**: {} 个\n**运行时间**: {}\n**AI 模型**: {}",
                        status_text, session_count, uptime_text, model_text
                    )
                }
            ]
        })).unwrap_or_default()
    }
}

impl Default for StreamingCardBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 飞书 tenant_access_token 响应
#[derive(Debug, Deserialize)]
struct TenantAccessTokenResponse {
    code: i32,
    msg: String,
    #[serde(default)]
    tenant_access_token: Option<String>,
    #[serde(default)]
    expire: Option<i64>,
}

/// 飞书通用 API 响应
#[derive(Debug, Deserialize)]
struct FeishuApiResponse<T> {
    code: i32,
    msg: String,
    #[serde(default)]
    data: Option<T>,
}

/// 飞书 WebSocket 事件信封
#[derive(Debug, Deserialize)]
struct WsEventEnvelope {
    #[serde(default)]
    sid: Option<String>,
    #[serde(default)]
    #[serde(rename = "type")]
    event_type: Option<String>,
    #[serde(default)]
    schema: Option<String>,
    #[serde(default)]
    header: Option<serde_json::Value>,
    #[serde(default)]
    event: Option<serde_json::Value>,
    #[serde(default)]
    challenge: Option<String>,
}

/// 飞书事件 Header
#[derive(Debug, Deserialize)]
struct EventHeader {
    #[serde(default)]
    event_id: Option<String>,
    #[serde(default)]
    event_type: Option<String>,
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    create_time: Option<String>,
}

/// 飞书消息事件
#[derive(Debug, Deserialize)]
struct MessageEvent {
    #[serde(default)]
    sender: Option<serde_json::Value>,
    #[serde(default)]
    message: Option<serde_json::Value>,
}

/// 飞书卡片回调事件
#[derive(Debug, Deserialize)]
struct CardCallbackEvent {
    #[serde(default)]
    operator: Option<serde_json::Value>,
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    action: Option<serde_json::Value>,
    #[serde(default)]
    host: Option<String>,
    #[serde(default)]
    context: Option<serde_json::Value>,
}

/// 发送消息请求体
#[derive(Debug, Serialize)]
struct SendMessageRequest<'a> {
    receive_id: &'a str,
    #[serde(rename = "msg_type")]
    msg_type: &'a str,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    uuid: Option<String>,
}

/// 发送消息响应
#[derive(Debug, Deserialize, Default)]
struct SendMessageData {
    #[serde(default)]
    message_id: Option<String>,
}

/// 飞书平台适配器（WebSocket 长连接 + Bridge 增强）
pub struct FeishuAdapter {
    config: ImPlatformConfig,
    client: reqwest::Client,
    running: AtomicBool,
    status: RwLock<ConnectionStatus>,
    message_tx: mpsc::Sender<UnifiedMessage>,
    handler: Mutex<Option<MessageHandler>>,
    token: Mutex<Option<String>>,
    token_expiry: Mutex<Option<chrono::DateTime<Utc>>>,
    ws_stream: Mutex<Option<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
    started_at: Mutex<Option<chrono::DateTime<Utc>>>,
    session_count: AtomicBool,
}

impl FeishuAdapter {
    pub fn new(config: ImPlatformConfig, message_tx: mpsc::Sender<UnifiedMessage>) -> Self {
        Self {
            client: reqwest::Client::new(),
            running: AtomicBool::new(false),
            status: RwLock::new(ConnectionStatus::Disconnected),
            message_tx,
            handler: Mutex::new(None),
            token: Mutex::new(None),
            token_expiry: Mutex::new(None),
            ws_stream: Mutex::new(None),
            started_at: Mutex::new(None),
            session_count: AtomicBool::new(false),
            config,
        }
    }

    fn clone_for_spawn(&self) -> Arc<Self> {
        Arc::new(Self {
            client: self.client.clone(),
            running: AtomicBool::new(self.running.load(Ordering::Relaxed)),
            status: RwLock::new(ConnectionStatus::Disconnected),
            message_tx: self.message_tx.clone(),
            handler: Mutex::new(None),
            token: Mutex::new(None),
            token_expiry: Mutex::new(None),
            ws_stream: Mutex::new(None),
            started_at: Mutex::new(None),
            session_count: AtomicBool::new(false),
            config: self.config.clone(),
        })
    }

    /// 从配置中获取 App ID
    fn get_app_id(&self) -> Result<String> {
        self.config
            .extra
            .as_ref()
            .and_then(|e| e.get("app_id"))
            .cloned()
            .ok_or_else(|| anyhow!("Feishu app_id not configured in extra"))
    }

    /// 从配置中获取 App Secret
    fn get_app_secret(&self) -> Result<String> {
        self.config
            .extra
            .as_ref()
            .and_then(|e| e.get("app_secret"))
            .cloned()
            .ok_or_else(|| anyhow!("Feishu app_secret not configured in extra"))
    }

    /// 获取 tenant_access_token
    async fn fetch_tenant_access_token(&self) -> Result<String> {
        let url = "https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal";
        let app_id = self.get_app_id()?;
        let app_secret = self.get_app_secret()?;

        let response = self
            .client
            .post(url)
            .json(&json!({
                "app_id": app_id,
                "app_secret": app_secret,
            }))
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch tenant_access_token: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "tenant_access_token HTTP error: {}",
                response.status()
            ));
        }

        let body: TenantAccessTokenResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse tenant_access_token response: {}", e))?;

        if body.code != 0 {
            return Err(anyhow!(
                "Feishu auth error ({}): {}",
                body.code,
                body.msg
            ));
        }

        let token = body
            .tenant_access_token
            .ok_or_else(|| anyhow!("No tenant_access_token in response"))?;

        let expiry = body.expire.unwrap_or(7200);
        let expiry_time = Utc::now() + chrono::Duration::seconds(expiry - 300);

        *self.token.lock().await = Some(token.clone());
        *self.token_expiry.lock().await = Some(expiry_time);

        Ok(token)
    }

    /// 获取有效的 tenant_access_token（带缓存）
    async fn get_valid_token(&self) -> Result<String> {
        let expiry = self.token_expiry.lock().await;
        if let Some(exp) = *expiry {
            if Utc::now() < exp {
                if let Some(token) = self.token.lock().await.as_ref() {
                    return Ok(token.clone());
                }
            }
        }
        drop(expiry);
        self.fetch_tenant_access_token().await
    }

    /// 生成飞书应用安装二维码 URL（简化版扫码授权）
    pub fn generate_install_qr_url(&self) -> Result<String> {
        let app_id = self.get_app_id()?;
        let redirect_uri = self
            .config
            .extra
            .as_ref()
            .and_then(|e| e.get("redirect_uri"))
            .cloned()
            .unwrap_or_else(|| "https://open.feishu.cn/app/cli_a00000000000000c/baseinfo".to_string());

        let url = format!(
            "https://open.feishu.cn/open-apis/authen/v1/index?app_id={}&redirect_uri={}&state=install",
            urlencoding::encode(&app_id),
            urlencoding::encode(&redirect_uri)
        );
        Ok(url)
    }

    /// 生成飞书应用配置页 URL（手动输入 App ID / App Secret）
    pub fn get_manual_config_url(&self) -> String {
        "https://open.feishu.cn/app".to_string()
    }

    /// WebSocket 主循环（含自动重连、心跳）
    async fn ws_loop(self: Arc<Self>) {
        let mut retry_delay = Duration::from_secs(1);
        const MAX_RETRY_DELAY: Duration = Duration::from_secs(60);
        const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

        while self.running.load(Ordering::Relaxed) {
            *self.status.write().await = ConnectionStatus::Connecting;

            match self.connect_ws().await {
                Ok((ws_stream, _)) => {
                    retry_delay = Duration::from_secs(1);
                    *self.status.write().await = ConnectionStatus::Connected;
                    tracing::info!("Feishu WebSocket connected");

                    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
                    let mut heartbeat = interval(HEARTBEAT_INTERVAL);
                    let mut pong_received = true;

                    loop {
                        tokio::select! {
                            _ = heartbeat.tick() => {
                                if !pong_received {
                                    tracing::warn!("Feishu heartbeat timeout, reconnecting...");
                                    break;
                                }
                                pong_received = false;
                                if let Err(e) = ws_sender.send(WsMessage::Ping(vec![])).await {
                                    tracing::error!("Feishu heartbeat send failed: {}", e);
                                    break;
                                }
                            }
                            Some(msg_result) = ws_receiver.next() => {
                                match msg_result {
                                    Ok(WsMessage::Text(text)) => {
                                        if let Err(e) = self.handle_ws_text(&text).await {
                                            tracing::error!("Feishu handle text error: {}", e);
                                        }
                                    }
                                    Ok(WsMessage::Binary(bin)) => {
                                        if let Err(e) = self.handle_ws_binary(&bin).await {
                                            tracing::error!("Feishu handle binary error: {}", e);
                                        }
                                    }
                                    Ok(WsMessage::Pong(_)) => {
                                        pong_received = true;
                                    }
                                    Ok(WsMessage::Close(_)) => {
                                        tracing::info!("Feishu WebSocket closed by server");
                                        break;
                                    }
                                    Ok(WsMessage::Ping(data)) => {
                                        if let Err(e) = ws_sender.send(WsMessage::Pong(data)).await {
                                            tracing::error!("Feishu pong failed: {}", e);
                                            break;
                                        }
                                    }
                                    Ok(WsMessage::Frame(_)) => {}
                                    Err(e) => {
                                        tracing::error!("Feishu WebSocket error: {}", e);
                                        break;
                                    }
                                }
                            }
                            else => {
                                tracing::warn!("Feishu WebSocket stream ended");
                                break;
                            }
                        }

                        if !self.running.load(Ordering::Relaxed) {
                            break;
                        }
                    }

                    let _ = ws_sender.close().await;
                }
                Err(e) => {
                    tracing::error!(
                        "Feishu WebSocket connect failed: {}, retry in {:?}",
                        e,
                        retry_delay
                    );
                    *self.status.write().await = ConnectionStatus::Error(e.to_string());
                    sleep(retry_delay).await;
                    retry_delay = std::cmp::min(retry_delay * 2, MAX_RETRY_DELAY);
                }
            }

            if !self.running.load(Ordering::Relaxed) {
                break;
            }
        }

        *self.status.write().await = ConnectionStatus::Disconnected;
        tracing::info!("Feishu WebSocket loop stopped");
    }

    /// 建立 WebSocket 连接
    async fn connect_ws(
        &self,
    ) -> Result<(
        WebSocketStream<MaybeTlsStream<TcpStream>>,
        tokio_tungstenite::tungstenite::handshake::client::Response,
    )> {
        let token = self.get_valid_token().await?;
        let ws_url = format!("wss://open.feishu.cn/open-apis/ws/v1/?tenant_access_token={}", token);

        let (ws_stream, response) = connect_async(&ws_url)
            .await
            .map_err(|e| anyhow!("WebSocket connect failed: {}", e))?;

        Ok((ws_stream, response))
    }

    /// 处理 WebSocket 文本消息
    async fn handle_ws_text(&self, text: &str) -> Result<()> {
        let envelope: WsEventEnvelope = serde_json::from_str(text)
            .map_err(|e| anyhow!("Failed to parse WS envelope: {}", e))?;

        let event_type = envelope
            .event_type
            .as_deref()
            .or_else(|| envelope.header.as_ref()?.get("event_type")?.as_str())
            .unwrap_or("");

        match event_type {
            "im.message.receive_v1" => {
                if let Some(event) = envelope.event {
                    if let Some(msg) = self.convert_message_event(event).await {
                        let is_cmd = self.try_handle_slash_command_if_any(
                            &msg.content, &msg.chat_id, &msg.user_id
                        ).await;
                        if !is_cmd {
                            self.dispatch_message(msg).await;
                        }
                    }
                }
            }
            "card.callback" => {
                if let Some(event) = envelope.event {
                    if let Some(msg) = self.convert_card_callback(event).await {
                        self.dispatch_message(msg).await;
                    }
                }
            }
            "url_verification" => {
                tracing::debug!("Feishu URL verification event received");
            }
            "" => {
                if let Some(challenge) = envelope.challenge {
                    tracing::debug!("Feishu challenge: {}", challenge);
                }
            }
            other => {
                tracing::debug!("Feishu unhandled event type: {}", other);
            }
        }

        Ok(())
    }

    /// 处理 WebSocket 二进制消息（飞书通常使用文本，此处做兼容）
    async fn handle_ws_binary(&self, bin: &[u8]) -> Result<()> {
        if let Ok(text) = String::from_utf8(bin.to_vec()) {
            self.handle_ws_text(&text).await?;
        }
        Ok(())
    }

    /// 将 im.message.receive_v1 事件转换为 UnifiedMessage
    async fn convert_message_event(&self, event: serde_json::Value) -> Option<UnifiedMessage> {
        let msg = event.get("message")?;
        let chat_id = msg
            .get("chat_id")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();

        let user_id = event
            .get("sender")
            .and_then(|s| s.get("sender_id"))
            .and_then(|id| id.get("user_id"))
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .to_string();

        let msg_type_str = msg
            .get("message_type")
            .and_then(|t| t.as_str())
            .unwrap_or("text");

        let msg_type = MessageType::from_str(msg_type_str).unwrap_or(MessageType::Text);

        let content = match msg_type {
            MessageType::Text => {
                let content_raw = msg
                    .get("content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("{}");
                let content_obj: serde_json::Value = serde_json::from_str(content_raw).unwrap_or_default();
                content_obj
                    .get("text")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string()
            }
            MessageType::Image => {
                let content_raw = msg
                    .get("content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("{}");
                let content_obj: serde_json::Value = serde_json::from_str(content_raw).unwrap_or_default();
                content_obj
                    .get("image_key")
                    .and_then(|t| t.as_str())
                    .unwrap_or("[image]")
                    .to_string()
            }
            MessageType::File => {
                let content_raw = msg
                    .get("content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("{}");
                let content_obj: serde_json::Value = serde_json::from_str(content_raw).unwrap_or_default();
                content_obj
                    .get("file_name")
                    .and_then(|f| f.as_str())
                    .unwrap_or("[file]")
                    .to_string()
            }
            MessageType::Card => {
                let content_raw = msg
                    .get("content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("{}");
                let content_obj: serde_json::Value = serde_json::from_str(content_raw).unwrap_or_default();
                serde_json::to_string(&content_obj).unwrap_or_else(|_| "[card]".to_string())
            }
            MessageType::Voice => "[voice]".to_string(),
        };

        let thread_id = msg
            .get("thread_id")
            .and_then(|t| t.as_str())
            .map(|s| s.to_string());

        Some(UnifiedMessage {
            platform: "feishu".to_string(),
            user_id,
            chat_id,
            message_type: msg_type,
            content,
            timestamp: Utc::now(),
            raw_data: event,
            thread_id,
        })
    }

    /// 将 card.callback 事件转换为 UnifiedMessage
    async fn convert_card_callback(&self, event: serde_json::Value) -> Option<UnifiedMessage> {
        let operator = event.get("operator")?;
        let user_id = operator
            .get("operator_id")
            .and_then(|id| id.get("user_id"))
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .to_string();

        let action = event.get("action")?;
        let action_value = action
            .get("value")
            .cloned()
            .unwrap_or_default();
        let content = serde_json::to_string(&action_value).unwrap_or_default();

        let chat_id = event
            .get("context")
            .and_then(|c| c.get("open_chat_id"))
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();

        Some(UnifiedMessage {
            platform: "feishu".to_string(),
            user_id,
            chat_id,
            message_type: MessageType::Card,
            content,
            timestamp: Utc::now(),
            raw_data: event,
            thread_id: None,
        })
    }

    /// 分发消息到 handler 和 channel
    async fn dispatch_message(&self, msg: UnifiedMessage) {
        if let Err(e) = self.message_tx.send(msg.clone()).await {
            tracing::error!("Failed to send message to channel: {}", e);
        }

        let handler_guard = self.handler.lock().await;
        if let Some(handler) = handler_guard.as_ref() {
            if let Err(e) = handler(msg) {
                tracing::error!("Feishu message handler error: {}", e);
            }
        }
    }

    /// 检查并处理斜杠命令，返回 true 表示已处理
    pub async fn try_handle_slash_command_if_any(&self, content: &str, chat_id: &str, user_id: &str) -> bool {
        if FeishuSlashCommand::is_slash_command(content) {
            if let Some(cmd) = FeishuSlashCommand::parse(content) {
                tracing::info!("Feishu slash command: /{} from user {}", cmd.command, user_id);
                if let Err(e) = self.handle_slash_command(&cmd, chat_id, user_id).await {
                    tracing::error!("Slash command error: {}", e);
                }
                return true;
            }
        }
        false
    }

    /// 发送消息到飞书，返回 message_id
    async fn send_message_api(
        &self,
        chat_id: &str,
        msg_type: &str,
        content_json: &str,
    ) -> Result<String> {
        let token = self.get_valid_token().await?;
        let url = "https://open.feishu.cn/open-apis/im/v1/messages";

        let request_body = SendMessageRequest {
            receive_id: chat_id,
            msg_type,
            content: content_json.to_string(),
            uuid: Some(uuid::Uuid::new_v4().to_string()),
        };

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", token))
            .query(&[("receive_id_type", "chat_id")])
            .json(&request_body)
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| anyhow!("Feishu send message request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Feishu send message HTTP error: {} - {}", status, body));
        }

        let resp: FeishuApiResponse<SendMessageData> = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse send message response: {}", e))?;

        if resp.code != 0 {
            return Err(anyhow!(
                "Feishu send message API error ({}): {}",
                resp.code,
                resp.msg
            ));
        }

        Ok(resp.data.and_then(|d| d.message_id).unwrap_or_default())
    }

    /// 更新已发送的消息（用于流式卡片逐步展示）
    async fn update_message_api(
        &self,
        message_id: &str,
        content_json: &str,
    ) -> Result<()> {
        let token = self.get_valid_token().await?;
        let url = format!("https://open.feishu.cn/open-apis/im/v1/messages/{}", message_id);

        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({
                "content": content_json,
            }))
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| anyhow!("Feishu update message request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Feishu update message HTTP error: {} - {}", status, body));
        }

        let resp: FeishuApiResponse<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse update message response: {}", e))?;

        if resp.code != 0 {
            return Err(anyhow!(
                "Feishu update message API error ({}): {}",
                resp.code,
                resp.msg
            ));
        }

        Ok(())
    }

    /// --- 流式互动卡片 ---

    /// 构建一个可更新的 Markdown 卡片 JSON
    fn build_card_json(content: &str) -> String {
        StreamingCardBuilder::build_start_card(content, true)
    }

    /// 发送初始流式卡片，返回 message_id
    pub async fn send_streaming_start(&self, chat_id: &str) -> Result<String> {
        let card_json = Self::build_card_json("🤔 *正在思考...*");
        self.send_message_api(chat_id, "interactive", &card_json).await
    }

    /// 更新流式卡片内容
    pub async fn send_streaming_update(&self, message_id: &str, text: &str) -> Result<()> {
        let card_json = Self::build_card_json(text);
        self.update_message_api(message_id, &card_json).await
    }

    /// 更新流式卡片内容（含工具调用信息）
    pub async fn send_streaming_update_with_tools(&self, message_id: &str, text: &str, tools: &[String]) -> Result<()> {
        let card_json = StreamingCardBuilder::build_update_card(text, tools);
        self.update_message_api(message_id, &card_json).await
    }

    /// 完成流式卡片（移除"流式输出中"提示）
    pub async fn send_streaming_done(&self, message_id: &str, final_text: &str) -> Result<()> {
        let card_json = StreamingCardBuilder::build_done_card(final_text, None);
        self.update_message_api(message_id, &card_json).await
    }

    pub async fn send_streaming_done_with_model(&self, message_id: &str, final_text: &str, model: &str) -> Result<()> {
        let card_json = StreamingCardBuilder::build_done_card(final_text, Some(model));
        self.update_message_api(message_id, &card_json).await
    }

    pub async fn send_streaming_error(&self, message_id: &str, error_text: &str) -> Result<()> {
        let card_json = StreamingCardBuilder::build_error_card(error_text);
        self.update_message_api(message_id, &card_json).await
    }

    pub async fn send_help_card(&self, chat_id: &str) -> Result<String> {
        let card_json = StreamingCardBuilder::build_help_card();
        self.send_message_api(chat_id, "interactive", &card_json).await
    }

    pub async fn send_status_card(&self, chat_id: &str, session_count: usize, model: Option<&str>) -> Result<String> {
        let connected = self.running.load(Ordering::Relaxed);
        let uptime = self.get_uptime_string();
        let card_json = StreamingCardBuilder::build_status_card(connected, session_count, uptime.as_deref(), model);
        self.send_message_api(chat_id, "interactive", &card_json).await
    }

    /// 处理斜杠命令
    pub async fn handle_slash_command(
        &self,
        cmd: &FeishuSlashCommand,
        chat_id: &str,
        _user_id: &str,
    ) -> Result<()> {
        match cmd.command.as_str() {
            "help" | "h" => {
                let _ = self.send_help_card(chat_id).await;
            }
            "new" | "newchat" => {
                let title = if cmd.args.is_empty() {
                    "新会话"
                } else {
                    &cmd.args.join(" ")
                };
                let _ = self.send_message_api(
                    chat_id,
                    "interactive",
                    &serde_json::to_string(&json!({
                        "config": { "wide_screen_mode": true },
                        "header": {
                            "title": { "tag": "plain_text", "content": "🆕 创建新会话" },
                            "template": "green"
                        },
                        "elements": [
                            {
                                "tag": "markdown",
                                "content": format!("正在创建新会话：**{}**\n\n请在飞书中新建一个群聊，将 Claude Bot 加入群中即可开始独立会话。", title)
                            },
                            {
                                "tag": "note",
                                "elements": [{ "tag": "plain_text", "content": "一事一群：每个群 = 一个独立的 AI 会话" }]
                            }
                        ]
                    })).unwrap_or_default(),
                ).await;
                self.session_count.store(true, Ordering::Relaxed);
            }
            "reconnect" | "r" => {
                let _ = self.send_message_api(
                    chat_id,
                    "text",
                    &serde_json::to_string(&json!({ "text": "🔌 正在重新连接 WebSocket..." })).unwrap_or_default(),
                ).await;
                self.running.store(false, Ordering::Relaxed);
                sleep(Duration::from_secs(1)).await;
                self.running.store(true, Ordering::Relaxed);
                if let Err(e) = self.connect().await {
                    let _ = self.send_message_api(
                        chat_id,
                        "text",
                        &serde_json::to_string(&json!({ "text": format!("❌ 重连失败: {}", e) })).unwrap_or_default(),
                    ).await;
                } else {
                    let _ = self.send_message_api(
                        chat_id,
                        "text",
                        &serde_json::to_string(&json!({ "text": "✅ WebSocket 重新连接成功" })).unwrap_or_default(),
                    ).await;
                }
            }
            "status" | "s" => {
                let session_count = if self.session_count.load(Ordering::Relaxed) { 1 } else { 0 };
                let _ = self.send_status_card(chat_id, session_count, None).await;
            }
            "doc" | "export" => {
                let title = if cmd.args.is_empty() {
                    "飞书对话记录"
                } else {
                    &cmd.args.join(" ")
                };
                let _ = self.send_message_api(
                    chat_id,
                    "interactive",
                    &serde_json::to_string(&json!({
                        "config": { "wide_screen_mode": true },
                        "header": {
                            "title": { "tag": "plain_text", "content": "📄 文档导出" },
                            "template": "purple"
                        },
                        "elements": [
                            {
                                "tag": "markdown",
                                "content": format!("正在准备导出对话内容为飞书文档...\n\n标题: **{}**\n\n此功能需要飞书应用的 doc:document 权限。", title)
                            }
                        ]
                    })).unwrap_or_default(),
                ).await;
            }
            "code" | "repo" => {
                let _ = self.send_message_api(
                    chat_id,
                    "interactive",
                    &serde_json::to_string(&json!({
                        "config": { "wide_screen_mode": true },
                        "header": {
                            "title": { "tag": "plain_text", "content": "💻 代码库" },
                            "template": "blue"
                        },
                        "elements": [
                            {
                                "tag": "markdown",
                                "content": "**当前代码库**\n\nTauri 桌面应用\n- 前端: React + TypeScript + Vite\n- 后端: Rust + Tauri v2\n- 功能: AI 助手、IM 集成、代码编辑\n\n使用 Claude 可以搜索、读取、编辑代码库中的文件。\n\n**提示**: 直接将代码问题发送给我，我会帮您分析。"
                            }
                        ]
                    })).unwrap_or_default(),
                ).await;
            }
            other => {
                let _ = self.send_message_api(
                    chat_id,
                    "text",
                    &serde_json::to_string(&json!({
                        "text": format!("❓ 未知命令: /{}\n使用 /help 查看可用命令", other)
                    })).unwrap_or_default(),
                ).await;
            }
        }
        Ok(())
    }

    /// 创建飞书群聊（用于一事一群功能）
    pub async fn create_feishu_group(
        &self,
        name: &str,
        description: Option<&str>,
    ) -> Result<String> {
        let token = self.get_valid_token().await?;
        let url = "https://open.feishu.cn/open-apis/im/v1/chats";

        let mut body = json!({
            "name": name,
            "chat_type": "private",
        });
        if let Some(desc) = description {
            body["description"] = json!(desc);
        }

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to create Feishu group: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Create group HTTP error: {} - {}", status, body_text));
        }

        let resp: FeishuApiResponse<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse create group response: {}", e))?;

        if resp.code != 0 {
            return Err(anyhow!("Create group API error ({}): {}", resp.code, resp.msg));
        }

        let chat_id = resp
            .data
            .and_then(|d| d.get("chat_id").and_then(|c| c.as_str()).map(|s| s.to_string()))
            .ok_or_else(|| anyhow!("No chat_id in create group response"))?;

        Ok(chat_id)
    }

    /// 将 Bot 添加到群聊
    pub async fn add_bot_to_chat(&self, chat_id: &str) -> Result<()> {
        let token = self.get_valid_token().await?;
        let url = format!("https://open.feishu.cn/open-apis/im/v1/chats/{}/members", chat_id);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .query(&[("member_id_type", "app_id")])
            .json(&json!({
                "id_list": [self.get_app_id()?]
            }))
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to add bot to chat: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Add bot to chat HTTP error: {} - {}", status, body_text));
        }

        Ok(())
    }

    /// 获取运行时间
    pub fn get_uptime_string(&self) -> Option<String> {
        let rt = tokio::runtime::Handle::try_current().ok()?;
        let _guard = rt.enter();
        let started = futures::executor::block_on(async { *self.started_at.lock().await });
        if let Some(start) = started {
            let duration = Utc::now() - start;
            let hours = duration.num_hours();
            let minutes = duration.num_minutes() % 60;
            Some(format!("{}h {}m", hours, minutes))
        } else {
            None
        }
    }

    pub async fn set_started_now(&self) {
        *self.started_at.lock().await = Some(Utc::now());
    }
}

#[async_trait]
impl PlatformAdapter for FeishuAdapter {
    async fn connect(&self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        self.fetch_tenant_access_token().await?;
        self.running.store(true, Ordering::Relaxed);
        self.set_started_now().await;

        let adapter = self.clone_for_spawn();
        tokio::spawn(async move {
            adapter.ws_loop().await;
        });

        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        self.running.store(false, Ordering::Relaxed);
        *self.status.write().await = ConnectionStatus::Disconnected;
        Ok(())
    }

    async fn send_message(&self, chat_id: &str, content: &str, msg_type: MessageType) -> Result<()> {
        let (msg_type_str, content_json) = match msg_type {
            MessageType::Text => {
                let json = serde_json::to_string(&json!({ "text": content }))
                    .unwrap_or_else(|_| format!("{{\"text\":\"{}\"}}", content));
                ("text", json)
            }
            MessageType::Image => {
                let json = serde_json::to_string(&json!({ "image_key": content }))
                    .unwrap_or_else(|_| format!("{{\"image_key\":\"{}\"}}", content));
                ("image", json)
            }
            MessageType::File => {
                let json = serde_json::to_string(&json!({ "file_key": content }))
                    .unwrap_or_else(|_| format!("{{\"file_key\":\"{}\"}}", content));
                ("file", json)
            }
            MessageType::Card => {
                let card_json = if content.trim().starts_with('{') {
                    content.to_string()
                } else {
                    serde_json::to_string(&json!({
                        "config": { "wide_screen_mode": true },
                        "elements": [
                            {
                                "tag": "div",
                                "text": {
                                    "tag": "lark_md",
                                    "content": content
                                }
                            }
                        ]
                    }))
                    .unwrap_or_else(|_| content.to_string())
                };
                ("interactive", card_json)
            }
            MessageType::Voice => {
                let json = serde_json::to_string(&json!({ "file_key": content }))
                    .unwrap_or_else(|_| format!("{{\"file_key\":\"{}\"}}", content));
                ("audio", json)
            }
        };

        self.send_message_api(chat_id, msg_type_str, &content_json).await?;
        Ok(())
    }

    async fn on_message(&self, handler: MessageHandler) -> Result<()> {
        *self.handler.lock().await = Some(handler);
        Ok(())
    }

    fn get_status(&self) -> ConnectionStatus {
        let rt = match tokio::runtime::Handle::try_current() {
            Ok(_handle) => {
                let status = self.status.try_read();
                match status {
                    Ok(s) => s.clone(),
                    Err(_) => ConnectionStatus::Connecting,
                }
            }
            Err(_) => ConnectionStatus::Disconnected,
        };
        rt
    }

    fn get_platform(&self) -> &str {
        "feishu"
    }
}

/// 兼容旧版 trait（telegram_adapter.rs 中定义的 PlatformAdapter）
/// 提供 start/stop/send_text/send_photo/send_inline_keyboard 方法
#[async_trait]
impl crate::im_integration::adapters::telegram_adapter::PlatformAdapter for FeishuAdapter {
    async fn start(&self) -> Result<()> {
        <Self as PlatformAdapter>::connect(self).await
    }

    async fn stop(&self) -> Result<()> {
        <Self as PlatformAdapter>::disconnect(self).await
    }

    async fn send_text(&self, chat_id: &str, text: &str) -> Result<()> {
        <Self as PlatformAdapter>::send_message(self, chat_id, text, MessageType::Text).await
    }

    async fn send_photo(&self, chat_id: &str, photo_url: &str, caption: Option<&str>) -> Result<()> {
        let content = if let Some(cap) = caption {
            format!("{}\n[图片: {}]", cap, photo_url)
        } else {
            format!("[图片: {}]", photo_url)
        };
        <Self as PlatformAdapter>::send_message(self, chat_id, &content, MessageType::Image).await
    }

    async fn send_inline_keyboard(
        &self,
        chat_id: &str,
        text: &str,
        buttons: Vec<Vec<crate::im_integration::adapters::telegram_adapter::InlineKeyboardButton>>,
    ) -> Result<()> {
        let card_elements: Vec<serde_json::Value> = buttons
            .into_iter()
            .map(|row| {
                let actions: Vec<serde_json::Value> = row
                    .into_iter()
                    .map(|btn| {
                        json!({
                            "tag": "button",
                            "text": {
                                "tag": "plain_text",
                                "content": btn.text
                            },
                            "type": "primary",
                            "value": {
                                "callback_data": btn.callback_data
                            }
                        })
                    })
                    .collect();
                json!({
                    "tag": "action",
                    "actions": actions
                })
            })
            .collect();

        let card_json = serde_json::to_string(&json!({
            "config": { "wide_screen_mode": true },
            "header": {
                "title": {
                    "tag": "plain_text",
                    "content": text
                }
            },
            "elements": card_elements
        }))
        .unwrap_or_else(|_| text.to_string());

        self.send_message_api(chat_id, "interactive", &card_json).await?;
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}
