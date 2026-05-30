use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use futures::SinkExt;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::{interval, sleep};

use crate::im_integration::message_router::{
    ConnectionStatus, MessageHandler, MessageType, PlatformAdapter, UnifiedMessage,
};

const DINGTALK_API_BASE: &str = "https://api.dingtalk.com";
const DINGTALK_WS_URL: &str = "wss://wss-open-connection.dingtalk.com";

/// 钉钉 Stream 连接响应
#[derive(Debug, Deserialize)]
struct StreamConnectionResponse {
    endpoint: String,
    ticket: String,
}

/// 钉钉 access_token 响应
#[derive(Debug, Deserialize)]
struct AccessTokenResponse {
    access_token: String,
    expire_in: Option<i64>,
}

/// 钉钉机器人消息回调数据结构
#[derive(Debug, Deserialize)]
struct DingTalkBotMessage {
    conversation_id: Option<String>,
    #[serde(rename = "conversationId")]
    conversation_id_alt: Option<String>,
    #[serde(rename = "senderStaffId")]
    sender_staff_id: Option<String>,
    #[serde(rename = "senderNick")]
    sender_nick: Option<String>,
    msgtype: Option<String>,
    text: Option<DingTalkTextContent>,
    markdown: Option<DingTalkMarkdownContent>,
    #[serde(rename = "createAt")]
    create_at: Option<String>,
    #[serde(rename = "chatbotCorpId")]
    chatbot_corp_id: Option<String>,
    #[serde(rename = "chatbotUserId")]
    chatbot_user_id: Option<String>,
    #[serde(rename = "msgId")]
    msg_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DingTalkTextContent {
    content: String,
}

#[derive(Debug, Deserialize)]
struct DingTalkMarkdownContent {
    title: Option<String>,
    text: String,
}

/// WebSocket 帧数据结构（钉钉 Stream 协议）
#[derive(Debug, Deserialize)]
struct StreamFrame {
    #[serde(rename = "type")]
    frame_type: Option<String>,
    headers: Option<serde_json::Value>,
    message: Option<serde_json::Value>,
    data: Option<serde_json::Value>,
}

/// 钉钉 Stream 适配器
pub struct DingTalkAdapter {
    client_id: String,
    client_secret: String,
    client: reqwest::Client,
    running: AtomicBool,
    status: Arc<RwLock<ConnectionStatus>>,
    message_tx: Mutex<Option<mpsc::Sender<UnifiedMessage>>>,
    access_token: Mutex<Option<String>>,
    token_expiry: Mutex<Option<chrono::DateTime<Utc>>>,
    ws_shutdown_tx: Mutex<Option<mpsc::Sender<()>>>,
}

impl DingTalkAdapter {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .unwrap_or_default(),
            running: AtomicBool::new(false),
            status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            message_tx: Mutex::new(None),
            access_token: Mutex::new(None),
            token_expiry: Mutex::new(None),
            ws_shutdown_tx: Mutex::new(None),
        }
    }

    /// 获取 access_token
    async fn get_access_token(&self) -> Result<String> {
        {
            let token = self.access_token.lock().await;
            let expiry = self.token_expiry.lock().await;
            if let (Some(t), Some(e)) = (token.as_ref(), expiry.as_ref()) {
                if *e > Utc::now() + Duration::from_secs(300) {
                    return Ok(t.clone());
                }
            }
        }

        let url = format!("{}/v1.0/oauth2/accessToken", DINGTALK_API_BASE);
        let response = self
            .client
            .post(&url)
            .json(&json!({
                "appKey": self.client_id,
                "appSecret": self.client_secret,
            }))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to get access token: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Access token API error: {} - {}", status, body));
        }

        let resp: AccessTokenResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse access token response: {}", e))?;

        let mut token = self.access_token.lock().await;
        let mut expiry = self.token_expiry.lock().await;
        *token = Some(resp.access_token.clone());
        *expiry = Some(Utc::now() + chrono::Duration::seconds(resp.expire_in.unwrap_or(7200)));

        Ok(resp.access_token)
    }

    /// 获取 Stream 连接 endpoint 和 ticket
    async fn get_stream_connection(&self) -> Result<(String, String)> {
        let access_token = self.get_access_token().await?;
        let url = format!("{}/v1.0/gateway/connections/open", DINGTALK_API_BASE);

        let response = self
            .client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&json!({
                "client_id": self.client_id,
                "client_secret": self.client_secret,
            }))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to open stream connection: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Stream connection API error: {} - {}", status, body));
        }

        let resp: StreamConnectionResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse stream connection response: {}", e))?;

        Ok((resp.endpoint, resp.ticket))
    }

    /// 发送机器人单聊消息
    async fn send_robot_o2o_message(
        &self,
        user_id: &str,
        msg_key: &str,
        msg_param: serde_json::Value,
    ) -> Result<()> {
        let access_token = self.get_access_token().await?;
        let url = format!("{}/v1.0/robot/oToMessages/batchSend", DINGTALK_API_BASE);

        let response = self
            .client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&json!({
                "robotCode": self.client_id,
                "userIds": [user_id],
                "msgKey": msg_key,
                "msgParam": msg_param.to_string(),
            }))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send robot message: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Send robot message API error: {} - {}", status, body));
        }

        Ok(())
    }

    /// 发送群聊机器人消息（使用 webhook）
    async fn send_group_robot_message(
        &self,
        webhook_url: &str,
        msg_type: &str,
        content: serde_json::Value,
    ) -> Result<()> {
        let mut body = json!({
            "msgtype": msg_type,
        });
        body[msg_type] = content;

        let response = self
            .client
            .post(webhook_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send group robot message: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Send group robot message API error: {} - {}",
                status,
                body_text
            ));
        }

        Ok(())
    }

    /// 解析钉钉消息为统一格式
    fn parse_message(&self, raw: serde_json::Value) -> Option<UnifiedMessage> {
        let msg: DingTalkBotMessage = match serde_json::from_value(raw.clone()) {
            Ok(m) => m,
            Err(_) => {
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
                let content = raw
                    .get("text")
                    .and_then(|t| t.get("content"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();

                return Some(UnifiedMessage {
                    platform: "dingtalk".to_string(),
                    user_id,
                    chat_id,
                    message_type: MessageType::Text,
                    content,
                    timestamp: Utc::now(),
                    raw_data: raw,
                    thread_id: None,
                });
            }
        };

        let chat_id = msg
            .conversation_id
            .or(msg.conversation_id_alt)
            .unwrap_or_default();
        let user_id = msg.sender_staff_id.unwrap_or_default();

        let (content, msg_type) = if let Some(text) = msg.text {
            (text.content, MessageType::Text)
        } else if let Some(md) = msg.markdown {
            (md.text, MessageType::Card)
        } else {
            ("[unsupported]".to_string(), MessageType::Text)
        };

        Some(UnifiedMessage {
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

    /// WebSocket 消息处理循环
    async fn ws_message_loop(
        &self,
        mut ws_stream: tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        mut shutdown_rx: mpsc::Receiver<()>,
    ) {
        let mut heartbeat_interval = interval(Duration::from_secs(30));
        let mut retry_delay = Duration::from_secs(1);
        const MAX_RETRY_DELAY: Duration = Duration::from_secs(60);

        loop {
            if !self.running.load(Ordering::Relaxed) {
                break;
            }

            tokio::select! {
                _ = shutdown_rx.recv() => {
                    tracing::info!("DingTalk WebSocket shutting down");
                    break;
                }
                _ = heartbeat_interval.tick() => {
                    let ping = tokio_tungstenite::tungstenite::protocol::Message::Ping(vec![]);
                    if let Err(e) = ws_stream.send(ping).await {
                        tracing::error!("DingTalk heartbeat ping failed: {}", e);
                        break;
                    }
                }
                msg = ws_stream.next() => {
                    match msg {
                        Some(Ok(tokio_tungstenite::tungstenite::protocol::Message::Text(text))) => {
                            retry_delay = Duration::from_secs(1);
                            if let Err(e) = self.handle_ws_text(&text).await {
                                tracing::error!("Failed to handle WebSocket text: {}", e);
                            }
                        }
                        Some(Ok(tokio_tungstenite::tungstenite::protocol::Message::Binary(bin))) => {
                            if let Ok(text) = String::from_utf8(bin) {
                                if let Err(e) = self.handle_ws_text(&text).await {
                                    tracing::error!("Failed to handle WebSocket binary: {}", e);
                                }
                            }
                        }
                        Some(Ok(tokio_tungstenite::tungstenite::protocol::Message::Close(_))) => {
                            tracing::warn!("DingTalk WebSocket closed by server");
                            break;
                        }
                        Some(Ok(tokio_tungstenite::tungstenite::protocol::Message::Pong(_))) => {
                        }
                        Some(Ok(tokio_tungstenite::tungstenite::protocol::Message::Ping(_))) => {
                        }
                        Some(Ok(tokio_tungstenite::tungstenite::protocol::Message::Frame(_))) => {
                        }
                        Some(Err(e)) => {
                            tracing::error!("DingTalk WebSocket error: {}", e);
                            break;
                        }
                        None => {
                            tracing::warn!("DingTalk WebSocket stream ended");
                            break;
                        }
                    }
                }
            }
        }

        tracing::info!("DingTalk WebSocket loop stopped, reconnecting in {:?}", retry_delay);
        sleep(retry_delay).await;
    }

    /// 处理 WebSocket 文本消息
    async fn handle_ws_text(&self, text: &str) -> Result<()> {
        let frame: StreamFrame = serde_json::from_str(text)
            .map_err(|e| anyhow!("Failed to parse stream frame: {}", e))?;

        let frame_type = frame.frame_type.as_deref().unwrap_or("");

        match frame_type {
            "SYSTEM" | "EVENT" | "CALLBACK" => {
                if let Some(data) = frame.data {
                    if let Some(msg) = self.parse_message(data.clone()) {
                        let tx = self.message_tx.lock().await;
                        if let Some(sender) = tx.as_ref() {
                            if let Err(e) = sender.send(msg).await {
                                tracing::error!("Failed to send message to channel: {}", e);
                            }
                        }
                    }
                } else if let Some(message) = frame.message {
                    if let Some(msg) = self.parse_message(message) {
                        let tx = self.message_tx.lock().await;
                        if let Some(sender) = tx.as_ref() {
                            if let Err(e) = sender.send(msg).await {
                                tracing::error!("Failed to send message to channel: {}", e);
                            }
                        }
                    }
                }
            }
            _ => {
                tracing::debug!("DingTalk unknown frame type: {}", frame_type);
            }
        }

        Ok(())
    }

    /// 主连接循环（含自动重连）
    async fn connection_loop(&self) {
        let mut retry_delay = Duration::from_secs(1);
        const MAX_RETRY_DELAY: Duration = Duration::from_secs(60);

        while self.running.load(Ordering::Relaxed) {
            {
                let mut status = self.status.write().await;
                *status = ConnectionStatus::Connecting;
            }

            match self.get_stream_connection().await {
                Ok((endpoint, ticket)) => {
                    tracing::info!(
                        "DingTalk stream connection opened, endpoint: {}",
                        endpoint
                    );

                    let ws_url = format!("{}?ticket={}", endpoint, ticket);
                    match tokio_tungstenite::connect_async(&ws_url).await {
                        Ok((ws_stream, _)) => {
                            retry_delay = Duration::from_secs(1);
                            {
                                let mut status = self.status.write().await;
                                *status = ConnectionStatus::Connected;
                            }

                            let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
                            {
                                let mut tx = self.ws_shutdown_tx.lock().await;
                                *tx = Some(shutdown_tx);
                            }

                            self.ws_message_loop(ws_stream, shutdown_rx).await;
                        }
                        Err(e) => {
                            tracing::error!("DingTalk WebSocket connect failed: {}", e);
                            {
                                let mut status = self.status.write().await;
                                *status = ConnectionStatus::Error(e.to_string());
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("DingTalk stream connection failed: {}", e);
                    {
                        let mut status = self.status.write().await;
                        *status = ConnectionStatus::Error(e.to_string());
                    }
                }
            }

            if self.running.load(Ordering::Relaxed) {
                tracing::info!(
                    "DingTalk reconnecting in {:?}",
                    retry_delay
                );
                sleep(retry_delay).await;
                retry_delay = std::cmp::min(retry_delay * 2, MAX_RETRY_DELAY);
            }
        }

        {
            let mut status = self.status.write().await;
            *status = ConnectionStatus::Disconnected;
        }
        tracing::info!("DingTalk connection loop stopped");
    }

    /// 生成钉钉应用授权 URL（简化版扫码授权）
    pub fn generate_auth_url(&self, redirect_uri: &str, state: &str) -> String {
        format!(
            "https://login.dingtalk.com/oauth2/auth?client_id={}&response_type=code&scope=openid&state={}&redirect_uri={}",
            self.client_id,
            state,
            urlencoding::encode(redirect_uri)
        )
    }

    /// 验证 Client ID 和 Secret 是否有效
    pub async fn validate_credentials(&self) -> Result<bool> {
        match self.get_access_token().await {
            Ok(_) => Ok(true),
            Err(e) => {
                tracing::error!("DingTalk credentials validation failed: {}", e);
                Ok(false)
            }
        }
    }
}

#[async_trait]
impl PlatformAdapter for DingTalkAdapter {
    async fn connect(&self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        self.validate_credentials().await?;
        self.running.store(true, Ordering::Relaxed);

        let adapter = self.clone_for_spawn();
        tokio::spawn(async move {
            adapter.connection_loop().await;
        });

        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        self.running.store(false, Ordering::Relaxed);

        let mut tx = self.ws_shutdown_tx.lock().await;
        if let Some(sender) = tx.take() {
            let _ = sender.send(()).await;
        }

        let mut status = self.status.write().await;
        *status = ConnectionStatus::Disconnected;

        Ok(())
    }

    async fn send_message(&self, chat_id: &str, content: &str, msg_type: MessageType) -> Result<()> {
        match msg_type {
            MessageType::Text => {
                if chat_id.starts_with("http") {
                    self.send_group_robot_message(
                        chat_id,
                        "text",
                        json!({ "content": content }),
                    )
                    .await?;
                } else {
                    self.send_robot_o2o_message(
                        chat_id,
                        "sampleText",
                        json!({ "content": content }),
                    )
                    .await?;
                }
            }
            MessageType::Card | MessageType::Image => {
                if chat_id.starts_with("http") {
                    self.send_group_robot_message(
                        chat_id,
                        "markdown",
                        json!({
                            "title": "消息",
                            "text": content,
                        }),
                    )
                    .await?;
                } else {
                    self.send_robot_o2o_message(
                        chat_id,
                        "sampleMarkdown",
                        json!({
                            "title": "消息",
                            "text": content,
                        }),
                    )
                    .await?;
                }
            }
            _ => {
                if chat_id.starts_with("http") {
                    self.send_group_robot_message(
                        chat_id,
                        "text",
                        json!({ "content": content }),
                    )
                    .await?;
                } else {
                    self.send_robot_o2o_message(
                        chat_id,
                        "sampleText",
                        json!({ "content": content }),
                    )
                    .await?;
                }
            }
        }

        Ok(())
    }

    async fn on_message(&self, handler: MessageHandler) -> Result<()> {
        let (tx, mut rx) = mpsc::channel::<UnifiedMessage>(1024);
        {
            let mut msg_tx = self.message_tx.lock().await;
            *msg_tx = Some(tx);
        }

        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Err(e) = handler(msg) {
                    tracing::error!("DingTalk message handler error: {}", e);
                }
            }
        });

        Ok(())
    }

    fn get_status(&self) -> ConnectionStatus {
        match tokio::runtime::Handle::try_current() {
            Ok(rt) => {
                let status = self.status.clone();
                tokio::task::block_in_place(move || {
                    rt.block_on(async move {
                        (*status.read().await).clone()
                    })
                })
            }
            Err(_) => ConnectionStatus::Disconnected,
        }
    }

    fn get_platform(&self) -> &str {
        "dingtalk"
    }
}

impl DingTalkAdapter {
    fn clone_for_spawn(&self) -> Self {
        Self {
            client_id: self.client_id.clone(),
            client_secret: self.client_secret.clone(),
            client: self.client.clone(),
            running: AtomicBool::new(self.running.load(Ordering::Relaxed)),
            status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            message_tx: Mutex::new(None),
            access_token: Mutex::new(None),
            token_expiry: Mutex::new(None),
            ws_shutdown_tx: Mutex::new(None),
        }
    }
}

use futures::StreamExt;
