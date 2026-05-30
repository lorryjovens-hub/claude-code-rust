use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

use crate::im_integration::message_router::{MessageType, UnifiedMessage};
use crate::im_integration::ImPlatformConfig;

/// 平台适配器 trait，定义所有 IM 适配器需实现的能力
#[async_trait]
pub trait PlatformAdapter: Send + Sync {
    /// 启动适配器，开始接收消息
    async fn start(&self) -> Result<()>;
    /// 停止适配器
    async fn stop(&self) -> Result<()>;
    /// 发送文本消息
    async fn send_text(&self, chat_id: &str, text: &str) -> Result<()>;
    /// 发送图片消息（通过 URL）
    async fn send_photo(&self, chat_id: &str, photo_url: &str, caption: Option<&str>) -> Result<()>;
    /// 发送带 Inline Keyboard 的消息
    async fn send_inline_keyboard(
        &self,
        chat_id: &str,
        text: &str,
        buttons: Vec<Vec<InlineKeyboardButton>>,
    ) -> Result<()>;
    /// 检查适配器是否正在运行
    fn is_running(&self) -> bool;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineKeyboardButton {
    pub text: String,
    pub callback_data: String,
}

/// Telegram Bot API 响应结构
#[derive(Debug, Deserialize)]
struct TelegramResponse<T> {
    ok: bool,
    result: Option<T>,
    description: Option<String>,
}

/// Telegram Update 结构
#[derive(Debug, Serialize, Deserialize)]
struct Update {
    update_id: i64,
    message: Option<TelegramMessage>,
    callback_query: Option<CallbackQuery>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TelegramMessage {
    message_id: i64,
    from: Option<TelegramUser>,
    chat: TelegramChat,
    date: i64,
    text: Option<String>,
    photo: Option<Vec<PhotoSize>>,
    document: Option<TelegramDocument>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TelegramUser {
    id: i64,
    username: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TelegramChat {
    id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct PhotoSize {
    file_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TelegramDocument {
    file_id: String,
    file_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CallbackQuery {
    id: String,
    from: TelegramUser,
    message: Option<TelegramMessage>,
    data: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TelegramUserInfo {
    id: i64,
    username: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
}

/// Telegram 长轮询适配器
pub struct TelegramAdapter {
    config: ImPlatformConfig,
    client: reqwest::Client,
    running: AtomicBool,
    message_tx: mpsc::Sender<UnifiedMessage>,
    base_url: String,
}

impl TelegramAdapter {
    pub fn new(config: ImPlatformConfig, message_tx: mpsc::Sender<UnifiedMessage>) -> Self {
        let base_url = format!("https://api.telegram.org/bot{}/", config.token);
        Self {
            client: reqwest::Client::new(),
            running: AtomicBool::new(false),
            message_tx,
            base_url,
            config,
        }
    }

    fn clone_for_spawn(&self) -> Self {
        Self {
            client: self.client.clone(),
            running: AtomicBool::new(self.running.load(Ordering::Relaxed)),
            message_tx: self.message_tx.clone(),
            base_url: self.base_url.clone(),
            config: self.config.clone(),
        }
    }

    /// 验证 Bot Token 是否有效（调用 getMe）
    pub async fn validate_token(&self) -> Result<TelegramUserInfo> {
        let url = format!("{}getMe", self.base_url);
        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to validate token: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Token validation failed with status: {}",
                response.status()
            ));
        }

        let body: TelegramResponse<TelegramUserInfo> = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse getMe response: {}", e))?;

        if !body.ok {
            return Err(anyhow!(
                "Telegram API error: {}",
                body.description.unwrap_or_default()
            ));
        }

        body.result.ok_or_else(|| anyhow!("Empty result from getMe"))
    }

    /// 长轮询循环，在后台任务中运行
    async fn polling_loop(&self, mut shutdown_rx: mpsc::Receiver<()>) {
        let mut offset: Option<i64> = None;
        let mut retry_delay = Duration::from_secs(1);
        const MAX_RETRY_DELAY: Duration = Duration::from_secs(60);
        const POLLING_TIMEOUT: u64 = 30;

        loop {
            if !self.running.load(Ordering::Relaxed) {
                break;
            }

            tokio::select! {
                _ = shutdown_rx.recv() => {
                    break;
                }
                result = self.poll_updates(offset, POLLING_TIMEOUT) => {
                    match result {
                        Ok(updates) => {
                            retry_delay = Duration::from_secs(1);
                            for update in updates {
                                offset = Some(update.update_id + 1);
                                if let Some(msg) = self.convert_update(update) {
                                    if let Err(e) = self.message_tx.send(msg).await {
                                        tracing::error!("Failed to send message to channel: {}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Polling error: {}, retry in {:?}", e, retry_delay);
                            sleep(retry_delay).await;
                            retry_delay = std::cmp::min(retry_delay * 2, MAX_RETRY_DELAY);
                        }
                    }
                }
            }
        }

        tracing::info!("Telegram polling loop stopped");
    }

    /// 单次轮询请求
    async fn poll_updates(&self, offset: Option<i64>, timeout: u64) -> Result<Vec<Update>> {
        let url = format!("{}getUpdates", self.base_url);
        let mut body = json!({
            "limit": 100,
            "timeout": timeout,
        });

        if let Some(off) = offset {
            body["offset"] = json!(off);
        }

        let response = self
            .client
            .post(&url)
            .json(&body)
            .timeout(Duration::from_secs(timeout + 10))
            .send()
            .await
            .map_err(|e| anyhow!("getUpdates request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("getUpdates HTTP error: {} - {}", status, text));
        }

        let resp: TelegramResponse<Vec<Update>> = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse getUpdates response: {}", e))?;

        if !resp.ok {
            return Err(anyhow!(
                "Telegram API error: {}",
                resp.description.unwrap_or_default()
            ));
        }

        Ok(resp.result.unwrap_or_default())
    }

    /// 将 Telegram Update 转换为统一消息格式
    fn convert_update(&self, update: Update) -> Option<UnifiedMessage> {
        let timestamp = Utc::now();

        if let Some(ref msg) = update.message {
            let chat_id = msg.chat.id.to_string();
            let user_id = msg.from.as_ref().map(|u| u.id.to_string()).unwrap_or_default();

            let (content, msg_type) = if let Some(ref text) = msg.text {
                (text.clone(), MessageType::Text)
            } else if msg.photo.is_some() {
                let caption = msg.text.clone().unwrap_or_default();
                (caption, MessageType::Image)
            } else if msg.document.is_some() {
                let file_name = msg
                    .document
                    .as_ref()
                    .and_then(|d| d.file_name.clone())
                    .unwrap_or_else(|| "[file]".to_string());
                (file_name, MessageType::File)
            } else {
                ("[unsupported]".to_string(), MessageType::Text)
            };

            let raw_data = serde_json::to_value(&update).unwrap_or_default();

            return Some(UnifiedMessage {
                platform: "telegram".to_string(),
                user_id,
                chat_id,
                message_type: msg_type,
                content,
                timestamp,
                raw_data,
                thread_id: None,
            });
        }

        if let Some(ref cb) = update.callback_query {
            let chat_id = cb
                .message
                .as_ref()
                .map(|m| m.chat.id.to_string())
                .unwrap_or_default();
            let user_id = cb.from.id.to_string();
            let content = cb.data.clone().unwrap_or_default();
            let raw_data = serde_json::to_value(&update).unwrap_or_default();

            return Some(UnifiedMessage {
                platform: "telegram".to_string(),
                user_id,
                chat_id,
                message_type: MessageType::Text,
                content,
                timestamp,
                raw_data,
                thread_id: None,
            });
        }

        None
    }
}

#[async_trait]
impl PlatformAdapter for TelegramAdapter {
    async fn start(&self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        self.validate_token().await?;
        self.running.store(true, Ordering::Relaxed);

        let (_shutdown_tx, shutdown_rx) = mpsc::channel(1);

        tokio::spawn({
            let adapter = self.clone_for_spawn();
            async move {
                adapter.polling_loop(shutdown_rx).await;
            }
        });

        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        self.running.store(false, Ordering::Relaxed);
        Ok(())
    }

    async fn send_text(&self, chat_id: &str, text: &str) -> Result<()> {
        let url = format!("{}sendMessage", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&json!({
                "chat_id": chat_id,
                "text": text,
                "parse_mode": "HTML",
            }))
            .send()
            .await
            .map_err(|e| anyhow!("sendMessage request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("sendMessage API error: {} - {}", status, body));
        }

        Ok(())
    }

    async fn send_photo(&self, chat_id: &str, photo_url: &str, caption: Option<&str>) -> Result<()> {
        let url = format!("{}sendPhoto", self.base_url);
        let mut body = json!({
            "chat_id": chat_id,
            "photo": photo_url,
        });

        if let Some(cap) = caption {
            body["caption"] = json!(cap);
            body["parse_mode"] = json!("HTML");
        }

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow!("sendPhoto request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("sendPhoto API error: {} - {}", status, body));
        }

        Ok(())
    }

    async fn send_inline_keyboard(
        &self,
        chat_id: &str,
        text: &str,
        buttons: Vec<Vec<InlineKeyboardButton>>,
    ) -> Result<()> {
        let url = format!("{}sendMessage", self.base_url);
        let inline_keyboard: Vec<Vec<serde_json::Value>> = buttons
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|btn| {
                        json!({
                            "text": btn.text,
                            "callback_data": btn.callback_data,
                        })
                    })
                    .collect()
            })
            .collect();

        let response = self
            .client
            .post(&url)
            .json(&json!({
                "chat_id": chat_id,
                "text": text,
                "parse_mode": "HTML",
                "reply_markup": {
                    "inline_keyboard": inline_keyboard,
                },
            }))
            .send()
            .await
            .map_err(|e| anyhow!("sendMessage with keyboard request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "sendMessage with keyboard API error: {} - {}",
                status,
                body
            ));
        }

        Ok(())
    }

    fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}
