use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::{interval, sleep};

use crate::im_integration::message_router::{
    ConnectionStatus, MessageHandler, MessageType, PlatformAdapter, UnifiedMessage,
};

const WECOM_API_BASE: &str = "https://qyapi.weixin.qq.com/cgi-bin";
const WECOM_WS_URL: &str = "wss://openws.work.weixin.qq.com";

#[derive(Debug, Deserialize)]
struct AccessTokenResponse {
    access_token: Option<String>,
    expires_in: Option<i64>,
    errcode: Option<i32>,
    errmsg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WeComMessage {
    #[serde(rename = "FromUserName")]
    from_user: Option<String>,
    #[serde(rename = "ToUserName")]
    to_user: Option<String>,
    #[serde(rename = "MsgType")]
    msg_type: Option<String>,
    #[serde(rename = "Content")]
    content: Option<String>,
    #[serde(rename = "MsgId")]
    msg_id: Option<String>,
    #[serde(rename = "CreateTime")]
    create_time: Option<i64>,
    #[serde(rename = "PicUrl")]
    pic_url: Option<String>,
    #[serde(rename = "MediaId")]
    media_id: Option<String>,
}

pub struct WeComAdapter {
    corp_id: String,
    corp_secret: String,
    agent_id: Option<String>,
    client: reqwest::Client,
    running: AtomicBool,
    status: Arc<RwLock<ConnectionStatus>>,
    message_tx: Mutex<Option<mpsc::Sender<UnifiedMessage>>>,
    access_token: Mutex<Option<String>>,
    token_expiry: Mutex<Option<chrono::DateTime<Utc>>>,
    ws_shutdown_tx: Mutex<Option<mpsc::Sender<()>>>,
    message_count: AtomicBool,
}

impl WeComAdapter {
    pub fn new(corp_id: String, corp_secret: String, agent_id: Option<String>) -> Self {
        Self {
            corp_id,
            corp_secret,
            agent_id,
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
            message_count: AtomicBool::new(false),
        }
    }

    async fn get_access_token(&self) -> Result<String> {
        {
            let token = self.access_token.lock().await;
            let expiry = self.token_expiry.lock().await;
            if let (Some(t), Some(e)) = (token.as_ref(), expiry.as_ref()) {
                if *e > Utc::now() + chrono::Duration::seconds(300) {
                    return Ok(t.clone());
                }
            }
        }

        let url = format!(
            "{}/gettoken?corpid={}&corpsecret={}",
            WECOM_API_BASE, self.corp_id, self.corp_secret
        );

        let response = self
            .client
            .get(&url)
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

        if let Some(errcode) = resp.errcode {
            if errcode != 0 {
                return Err(anyhow!(
                    "WeCom API error: {} - {}",
                    errcode,
                    resp.errmsg.unwrap_or_default()
                ));
            }
        }

        let token = resp.access_token.ok_or_else(|| anyhow!("No access token in response"))?;

        let mut stored_token = self.access_token.lock().await;
        let mut expiry = self.token_expiry.lock().await;
        *stored_token = Some(token.clone());
        *expiry = Some(Utc::now() + chrono::Duration::seconds(resp.expires_in.unwrap_or(7200)));

        Ok(token)
    }

    fn parse_message(&self, raw: serde_json::Value) -> Option<UnifiedMessage> {
        let msg: WeComMessage = match serde_json::from_value(raw.clone()) {
            Ok(m) => m,
            Err(_) => {
                let chat_id = raw
                    .get("FromUserName")
                    .and_then(|f| f.as_str())
                    .unwrap_or("")
                    .to_string();
                let content = raw
                    .get("Content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                let msg_type = raw
                    .get("MsgType")
                    .and_then(|m| m.as_str())
                    .unwrap_or("text");

                return Some(UnifiedMessage {
                    platform: "wecom".to_string(),
                    user_id: chat_id.clone(),
                    chat_id,
                    message_type: MessageType::from_str(msg_type).unwrap_or(MessageType::Text),
                    content,
                    timestamp: Utc::now(),
                    raw_data: raw,
                    thread_id: None,
                });
            }
        };

        let chat_id = msg.from_user.unwrap_or_default();
        let user_id = chat_id.clone();
        let content = msg.content.unwrap_or_default();
        let msg_type = msg.msg_type.as_deref().unwrap_or("text");

        Some(UnifiedMessage {
            platform: "wecom".to_string(),
            user_id,
            chat_id,
            message_type: MessageType::from_str(msg_type).unwrap_or(MessageType::Text),
            content,
            timestamp: Utc::now(),
            raw_data: raw,
            thread_id: None,
        })
    }

    async fn send_text_message(&self, user_id: &str, content: &str) -> Result<()> {
        let access_token = self.get_access_token().await?;
        let url = format!("{}/message/send?access_token={}", WECOM_API_BASE, access_token);

        let mut body = json!({
            "touser": user_id,
            "msgtype": "text",
            "agentid": self.agent_id.as_deref().unwrap_or(""),
            "text": {
                "content": content
            },
            "safe": 0
        });

        if self.agent_id.as_ref().map_or(true, |s| s.is_empty()) {
            body["agentid"] = json!("");
        }

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send WeCom message: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Send message API error: {} - {}", status, body_text));
        }

        let resp: serde_json::Value = response.json().await?;
        if let Some(errcode) = resp.get("errcode").and_then(|v| v.as_i64()) {
            if errcode != 0 {
                return Err(anyhow!(
                    "WeCom send error: {} - {}",
                    errcode,
                    resp.get("errmsg").and_then(|v| v.as_str()).unwrap_or("Unknown")
                ));
            }
        }

        Ok(())
    }

    async fn send_markdown_message(&self, user_id: &str, content: &str) -> Result<()> {
        let access_token = self.get_access_token().await?;
        let url = format!("{}/message/send?access_token={}", WECOM_API_BASE, access_token);

        let body = json!({
            "touser": user_id,
            "msgtype": "markdown",
            "agentid": self.agent_id.as_deref().unwrap_or(""),
            "markdown": {
                "content": content
            },
        });

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send WeCom markdown: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Send markdown API error: {} - {}", status, body_text));
        }

        Ok(())
    }

    async fn connection_loop(&self) {
        let mut retry_delay = Duration::from_secs(1);
        const MAX_RETRY_DELAY: Duration = Duration::from_secs(60);
        let mut messages_this_minute = 0u32;
        let mut messages_this_hour = 0u32;
        let mut minute_start = tokio::time::Instant::now();
        let mut hour_start = tokio::time::Instant::now();

        while self.running.load(Ordering::Relaxed) {
            if minute_start.elapsed() >= Duration::from_secs(60) {
                messages_this_minute = 0;
                minute_start = tokio::time::Instant::now();
            }
            if hour_start.elapsed() >= Duration::from_secs(3600) {
                messages_this_hour = 0;
                hour_start = tokio::time::Instant::now();
            }

            if messages_this_minute >= 30 || messages_this_hour >= 1000 {
                tracing::warn!("WeCom rate limit reached, waiting...");
                sleep(Duration::from_secs(60)).await;
                continue;
            }

            {
                let mut status = self.status.write().await;
                *status = ConnectionStatus::Connecting;
            }

            match self.get_access_token().await {
                Ok(_token) => {
                    tracing::info!("WeCom access token obtained, starting message polling");
                    retry_delay = Duration::from_secs(1);

                    {
                        let mut status = self.status.write().await;
                        *status = ConnectionStatus::Connected;
                    }

                    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
                    {
                        let mut tx = self.ws_shutdown_tx.lock().await;
                        *tx = Some(shutdown_tx);
                    }

                    let mut heartbeat = interval(Duration::from_secs(30));

                    loop {
                        tokio::select! {
                            _ = shutdown_rx.recv() => {
                                tracing::info!("WeCom polling shutting down");
                                break;
                            }
                            _ = heartbeat.tick() => {
                                tracing::debug!("WeCom heartbeat");
                            }
                            _ = sleep(Duration::from_secs(5)) => {
                                messages_this_minute += 1;
                                messages_this_hour += 1;
                            }
                        }

                        if !self.running.load(Ordering::Relaxed) {
                            break;
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("WeCom access token failed: {}", e);
                    {
                        let mut status = self.status.write().await;
                        *status = ConnectionStatus::Error(e.to_string());
                    }
                }
            }

            if self.running.load(Ordering::Relaxed) {
                tracing::info!("WeCom reconnecting in {:?}", retry_delay);
                sleep(retry_delay).await;
                retry_delay = std::cmp::min(retry_delay * 2, MAX_RETRY_DELAY);
            }
        }

        {
            let mut status = self.status.write().await;
            *status = ConnectionStatus::Disconnected;
        }
        tracing::info!("WeCom connection loop stopped");
    }

    pub async fn validate_credentials(&self) -> Result<bool> {
        match self.get_access_token().await {
            Ok(_) => Ok(true),
            Err(e) => {
                tracing::error!("WeCom credentials validation failed: {}", e);
                Ok(false)
            }
        }
    }

    pub fn generate_auth_url(&self, redirect_uri: &str, state: &str) -> String {
        format!(
            "https://open.weixin.qq.com/connect/oauth2/authorize?appid={}&redirect_uri={}&response_type=code&scope=snsapi_base&state={}#wechat_redirect",
            self.corp_id,
            urlencoding::encode(redirect_uri),
            state
        )
    }
}

#[async_trait]
impl PlatformAdapter for WeComAdapter {
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
                self.send_text_message(chat_id, content).await?;
            }
            MessageType::Card | MessageType::Image => {
                self.send_markdown_message(chat_id, content).await?;
            }
            _ => {
                self.send_text_message(chat_id, content).await?;
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
                    tracing::error!("WeCom message handler error: {}", e);
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
        "wecom"
    }
}

impl WeComAdapter {
    fn clone_for_spawn(&self) -> Self {
        Self {
            corp_id: self.corp_id.clone(),
            corp_secret: self.corp_secret.clone(),
            agent_id: self.agent_id.clone(),
            client: self.client.clone(),
            running: AtomicBool::new(self.running.load(Ordering::Relaxed)),
            status: Arc::new(RwLock::new(ConnectionStatus::Disconnected)),
            message_tx: Mutex::new(None),
            access_token: Mutex::new(None),
            token_expiry: Mutex::new(None),
            ws_shutdown_tx: Mutex::new(None),
            message_count: AtomicBool::new(false),
        }
    }
}
