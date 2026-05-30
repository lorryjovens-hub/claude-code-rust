use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MessageType {
    Text,
    Image,
    File,
    Card,
    Voice,
}

impl MessageType {
    pub fn as_str(&self) -> &str {
        match self {
            MessageType::Text => "text",
            MessageType::Image => "image",
            MessageType::File => "file",
            MessageType::Card => "card",
            MessageType::Voice => "voice",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "text" => Some(MessageType::Text),
            "image" => Some(MessageType::Image),
            "file" => Some(MessageType::File),
            "card" => Some(MessageType::Card),
            "voice" => Some(MessageType::Voice),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMessage {
    pub platform: String,
    pub user_id: String,
    pub chat_id: String,
    pub message_type: MessageType,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub raw_data: serde_json::Value,
    pub thread_id: Option<String>,
}

impl UnifiedMessage {
    pub fn session_key(&self) -> String {
        format!("{}:{}:{}", self.platform, self.user_id, self.chat_id)
    }

    pub fn thread_key(&self) -> String {
        match &self.thread_id {
            Some(thread_id) => format!("{}:{}:{}:{}", self.platform, self.user_id, self.chat_id, thread_id),
            None => self.session_key(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

impl ConnectionStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ConnectionStatus::Disconnected => "disconnected",
            ConnectionStatus::Connecting => "connecting",
            ConnectionStatus::Connected => "connected",
            ConnectionStatus::Error(_) => "error",
        }
    }
}

pub type MessageHandler = Arc<dyn Fn(UnifiedMessage) -> Result<()> + Send + Sync>;

#[async_trait]
pub trait PlatformAdapter: Send + Sync {
    async fn connect(&self) -> Result<()>;
    async fn disconnect(&self) -> Result<()>;
    async fn send_message(&self, chat_id: &str, content: &str, msg_type: MessageType) -> Result<()>;
    async fn on_message(&self, handler: MessageHandler) -> Result<()>;
    fn get_status(&self) -> ConnectionStatus;
    fn get_platform(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyPayload {
    pub platform: String,
    pub chat_id: String,
    pub content: String,
    pub message_type: MessageType,
    pub thread_id: Option<String>,
    pub extra: Option<serde_json::Value>,
}

pub struct MessageRouter {
    adapters: RwLock<HashMap<String, Arc<dyn PlatformAdapter>>>,
    inbound_tx: mpsc::Sender<UnifiedMessage>,
    pub inbound_rx: Mutex<mpsc::Receiver<UnifiedMessage>>,
    handlers: Mutex<Vec<MessageHandler>>,
}

impl MessageRouter {
    pub fn new() -> Self {
        let (inbound_tx, inbound_rx) = mpsc::channel::<UnifiedMessage>(1024);
        Self {
            adapters: RwLock::new(HashMap::new()),
            inbound_tx,
            inbound_rx: Mutex::new(inbound_rx),
            handlers: Mutex::new(Vec::new()),
        }
    }

    pub async fn register_adapter(&self, adapter: Arc<dyn PlatformAdapter>) -> Result<()> {
        let platform = adapter.get_platform().to_string();
        let mut adapters = self.adapters.write().await;
        adapters.insert(platform, adapter);
        Ok(())
    }

    pub async fn unregister_adapter(&self, platform: &str) -> Result<()> {
        let mut adapters = self.adapters.write().await;
        adapters.remove(platform);
        Ok(())
    }

    pub async fn get_adapter(&self, platform: &str) -> Option<Arc<dyn PlatformAdapter>> {
        let adapters = self.adapters.read().await;
        adapters.get(platform).cloned()
    }

    pub async fn list_platforms(&self) -> Vec<String> {
        let adapters = self.adapters.read().await;
        adapters.keys().cloned().collect()
    }

    pub async fn connect_platform(&self, platform: &str) -> Result<()> {
        let adapters = self.adapters.read().await;
        let adapter = adapters
            .get(platform)
            .ok_or_else(|| anyhow::anyhow!("Platform {} not registered", platform))?
            .clone();
        drop(adapters);

        let tx = self.inbound_tx.clone();

        let handler: MessageHandler = Arc::new(move |msg: UnifiedMessage| -> Result<()> {
            let tx = tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(msg).await;
            });
            Ok(())
        });

        adapter.on_message(handler).await?;
        Ok(())
    }

    pub async fn disconnect_platform(&self, platform: &str) -> Result<()> {
        let adapters = self.adapters.read().await;
        let adapter = adapters
            .get(platform)
            .ok_or_else(|| anyhow::anyhow!("Platform {} not registered", platform))?
            .clone();
        drop(adapters);
        adapter.disconnect().await
    }

    pub async fn send_reply(&self, payload: ReplyPayload) -> Result<()> {
        let adapters = self.adapters.read().await;
        let adapter = adapters
            .get(&payload.platform)
            .ok_or_else(|| anyhow::anyhow!("Platform {} not registered", payload.platform))?;

        let formatted = self.format_outbound(&payload);
        adapter
            .send_message(&payload.chat_id, &formatted, payload.message_type.clone())
            .await
    }

    /// Send a streaming Feishu card reply (start → update(s) → done).
    /// Returns a message_id for subsequent updates.
    pub async fn feishu_stream_start(&self, chat_id: &str) -> Result<String> {
        let adapters = self.adapters.read().await;
        let adapter = adapters.get("feishu")
            .ok_or_else(|| anyhow::anyhow!("Feishu adapter not registered"))?
            .clone();
        drop(adapters);

        let card = serde_json::json!({
            "config": { "wide_screen_mode": true },
            "header": { "title": { "tag": "plain_text", "content": "Claude" }, "template": "blue" },
            "elements": [
                { "tag": "markdown", "content": "🤔 *正在思考...*" },
                { "tag": "note", "elements": [{ "tag": "plain_text", "content": "流式输出中..." }] }
            ]
        });
        let card_str = serde_json::to_string(&card).unwrap_or_default();
        let _ = adapter.send_message(chat_id, &card_str, MessageType::Card).await?;
        Ok(String::new())
    }

    pub async fn add_handler(&self, handler: MessageHandler) {
        let mut handlers = self.handlers.lock().await;
        handlers.push(handler);
    }

    pub async fn start_dispatch(&self) -> Result<()> {
        let mut rx = self.inbound_rx.lock().await;
        while let Some(msg) = rx.recv().await {
            let handlers = self.handlers.lock().await.clone();
            for handler in handlers {
                if let Err(e) = handler(msg.clone()) {
                    tracing::error!("Message handler error: {}", e);
                }
            }
        }
        Ok(())
    }

    pub fn parse_inbound(&self, platform: &str, raw: serde_json::Value) -> Result<UnifiedMessage> {
        match platform {
            "telegram" => self.parse_telegram(raw),
            "feishu" => self.parse_feishu(raw),
            "wechat" => self.parse_wechat(raw),
            "dingtalk" => self.parse_dingtalk(raw),
            _ => Err(anyhow::anyhow!("Unsupported platform: {}", platform)),
        }
    }

    pub fn format_outbound(&self, payload: &ReplyPayload) -> String {
        match payload.platform.as_str() {
            "telegram" => self.format_telegram_reply(payload),
            "feishu" => self.format_feishu_reply(payload),
            "wechat" => self.format_wechat_reply(payload),
            "dingtalk" => self.format_dingtalk_reply(payload),
            _ => payload.content.clone(),
        }
    }

    fn parse_telegram(&self, raw: serde_json::Value) -> Result<UnifiedMessage> {
        let message = raw
            .get("message")
            .or_else(|| raw.get("edited_message"))
            .ok_or_else(|| anyhow::anyhow!("Missing message field"))?;

        let chat_id = message
            .get("chat")
            .and_then(|c| c.get("id"))
            .and_then(|id| id.as_i64())
            .map(|id| id.to_string())
            .unwrap_or_default();

        let user_id = message
            .get("from")
            .and_then(|f| f.get("id"))
            .and_then(|id| id.as_i64())
            .map(|id| id.to_string())
            .unwrap_or_default();

        let (content, msg_type) = if let Some(text) = message.get("text").and_then(|t| t.as_str()) {
            (text.to_string(), MessageType::Text)
        } else if message.get("photo").is_some() {
            let caption = message
                .get("caption")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string();
            (caption, MessageType::Image)
        } else if message.get("voice").is_some() {
            ("[voice]".to_string(), MessageType::Voice)
        } else if message.get("document").is_some() {
            let file_name = message
                .get("document")
                .and_then(|d| d.get("file_name"))
                .and_then(|f| f.as_str())
                .unwrap_or("[file]")
                .to_string();
            (file_name, MessageType::File)
        } else {
            ("[unsupported]".to_string(), MessageType::Text)
        };

        let thread_id = message
            .get("message_thread_id")
            .and_then(|t| t.as_i64())
            .map(|id| id.to_string());

        Ok(UnifiedMessage {
            platform: "telegram".to_string(),
            user_id,
            chat_id,
            message_type: msg_type,
            content,
            timestamp: Utc::now(),
            raw_data: raw,
            thread_id,
        })
    }

    fn parse_feishu(&self, raw: serde_json::Value) -> Result<UnifiedMessage> {
        let event = raw.get("event").unwrap_or(&raw).clone();
        let msg = event
            .get("message")
            .ok_or_else(|| anyhow::anyhow!("Missing message field"))?;

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
            MessageType::Image => "[image]".to_string(),
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
            _ => "[unsupported]".to_string(),
        };

        let thread_id = msg
            .get("thread_id")
            .and_then(|t| t.as_str())
            .map(|s| s.to_string());

        Ok(UnifiedMessage {
            platform: "feishu".to_string(),
            user_id,
            chat_id,
            message_type: msg_type,
            content,
            timestamp: Utc::now(),
            raw_data: raw,
            thread_id,
        })
    }

    fn parse_wechat(&self, raw: serde_json::Value) -> Result<UnifiedMessage> {
        let msg_type_str = raw
            .get("MsgType")
            .and_then(|m| m.as_str())
            .unwrap_or("text");

        let msg_type = match msg_type_str {
            "text" => MessageType::Text,
            "image" => MessageType::Image,
            "voice" => MessageType::Voice,
            "file" => MessageType::File,
            _ => MessageType::Text,
        };

        let chat_id = raw
            .get("FromUserName")
            .and_then(|f| f.as_str())
            .unwrap_or("")
            .to_string();

        let content = if msg_type == MessageType::Text {
            raw.get("Content")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string()
        } else {
            format!("[{}]", msg_type_str)
        };

        Ok(UnifiedMessage {
            platform: "wechat".to_string(),
            user_id: chat_id.clone(),
            chat_id,
            message_type: msg_type,
            content,
            timestamp: Utc::now(),
            raw_data: raw,
            thread_id: None,
        })
    }

    fn parse_dingtalk(&self, raw: serde_json::Value) -> Result<UnifiedMessage> {
        let chat_id = raw
            .get("conversationId")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();

        let user_id = raw
            .get("senderStaffId")
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();

        let msg_type_str = raw
            .get("msgtype")
            .and_then(|m| m.as_str())
            .unwrap_or("text");

        let msg_type = match msg_type_str {
            "text" => MessageType::Text,
            "image" => MessageType::Image,
            "file" => MessageType::File,
            "voice" => MessageType::Voice,
            _ => MessageType::Text,
        };

        let content = if msg_type == MessageType::Text {
            raw.get("text")
                .and_then(|t| t.get("content"))
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string()
        } else {
            format!("[{}]", msg_type_str)
        };

        Ok(UnifiedMessage {
            platform: "dingtalk".to_string(),
            user_id,
            chat_id,
            message_type: msg_type,
            content,
            timestamp: Utc::now(),
            raw_data: raw,
            thread_id: None,
        })
    }

    fn format_telegram_reply(&self, payload: &ReplyPayload) -> String {
        match payload.message_type {
            MessageType::Text => payload.content.clone(),
            MessageType::Image => payload.content.clone(),
            MessageType::File => payload.content.clone(),
            MessageType::Card => {
                serde_json::to_string(&serde_json::json!({
                    "text": payload.content,
                    "parse_mode": "HTML"
                }))
                .unwrap_or_else(|_| payload.content.clone())
            }
            MessageType::Voice => payload.content.clone(),
        }
    }

    fn format_feishu_reply(&self, payload: &ReplyPayload) -> String {
        match payload.message_type {
            MessageType::Text => {
                serde_json::to_string(&serde_json::json!({
                    "text": payload.content
                }))
                .unwrap_or_else(|_| payload.content.clone())
            }
            MessageType::Image => payload.content.clone(),
            MessageType::File => payload.content.clone(),
            MessageType::Card => {
                serde_json::to_string(&serde_json::json!({
                    "config": {
                        "wide_screen_mode": true
                    },
                    "elements": [
                        {
                            "tag": "div",
                            "text": {
                                "tag": "lark_md",
                                "content": payload.content
                            }
                        }
                    ]
                }))
                .unwrap_or_else(|_| payload.content.clone())
            }
            MessageType::Voice => payload.content.clone(),
        }
    }

    fn format_wechat_reply(&self, payload: &ReplyPayload) -> String {
        match payload.message_type {
            MessageType::Text => {
                serde_json::to_string(&serde_json::json!({
                    "msgtype": "text",
                    "text": {
                        "content": payload.content
                    }
                }))
                .unwrap_or_else(|_| payload.content.clone())
            }
            MessageType::Image => {
                serde_json::to_string(&serde_json::json!({
                    "msgtype": "image",
                    "image": {
                        "media_id": payload.content
                    }
                }))
                .unwrap_or_else(|_| payload.content.clone())
            }
            MessageType::File => {
                serde_json::to_string(&serde_json::json!({
                    "msgtype": "file",
                    "file": {
                        "media_id": payload.content
                    }
                }))
                .unwrap_or_else(|_| payload.content.clone())
            }
            MessageType::Card => {
                serde_json::to_string(&serde_json::json!({
                    "msgtype": "markdown",
                    "markdown": {
                        "content": payload.content
                    }
                }))
                .unwrap_or_else(|_| payload.content.clone())
            }
            MessageType::Voice => payload.content.clone(),
        }
    }

    fn format_dingtalk_reply(&self, payload: &ReplyPayload) -> String {
        match payload.message_type {
            MessageType::Text => {
                serde_json::to_string(&serde_json::json!({
                    "msgtype": "text",
                    "text": {
                        "content": payload.content
                    }
                }))
                .unwrap_or_else(|_| payload.content.clone())
            }
            MessageType::Image => {
                serde_json::to_string(&serde_json::json!({
                    "msgtype": "image",
                    "image": {
                        "picURL": payload.content
                    }
                }))
                .unwrap_or_else(|_| payload.content.clone())
            }
            MessageType::File => {
                serde_json::to_string(&serde_json::json!({
                    "msgtype": "file",
                    "file": {
                        "media_id": payload.content
                    }
                }))
                .unwrap_or_else(|_| payload.content.clone())
            }
            MessageType::Card => {
                serde_json::to_string(&serde_json::json!({
                    "msgtype": "markdown",
                    "markdown": {
                        "title": "消息",
                        "text": payload.content
                    }
                }))
                .unwrap_or_else(|_| payload.content.clone())
            }
            MessageType::Voice => payload.content.clone(),
        }
    }
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}
