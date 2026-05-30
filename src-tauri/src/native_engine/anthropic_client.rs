use crate::native_engine::provider_manager::ResolvedProvider;
use crate::tools::ToolDefinition;
use anyhow::Result;
use futures::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::pin::Pin;
use tokio_stream::StreamExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: AnthropicContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnthropicContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        source: ImageSource,
    },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
    #[serde(rename = "thinking")]
    Thinking {
        thinking: String,
        signature: Option<String>,
    },
    #[serde(rename = "redacted_thinking")]
    RedactedThinking {
        data: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub media_type: String,
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnthropicResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub role: String,
    pub content: Vec<ContentBlock>,
    pub model: String,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: Option<UsageInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageInfo {
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(default)]
    pub cache_creation_input_tokens: Option<u32>,
    #[serde(default)]
    pub cache_read_input_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub index: Option<usize>,
    pub delta: Option<StreamDelta>,
    pub content_block: Option<ContentBlock>,
    pub message: Option<StreamMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamDelta {
    #[serde(rename = "type")]
    pub delta_type: String,
    pub text: Option<String>,
    pub partial_json: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamMessage {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub message_type: Option<String>,
    pub role: Option<String>,
    pub content: Option<Vec<ContentBlock>>,
    pub model: Option<String>,
    pub stop_reason: Option<String>,
    pub usage: Option<UsageInfo>,
}


/// Retry with exponential backoff for transient errors (429, 5xx, connection).
async fn retry_with_backoff<F, Fut, T>(mut f: F) -> anyhow::Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<T>>,
{
    let max_retries = crate::config::OrchestrationConfig::max_retries();
    let mut attempt = 0;
    loop {
        attempt += 1;
        match f().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                let msg = format!("{:#}", e);
                let retryable = msg.contains("429") || msg.contains("5")
                    || msg.contains("500") || msg.contains("502")
                    || msg.contains("503") || msg.contains("504")
                    || msg.contains("connection") || msg.contains("timeout")
                    || msg.contains("tls");
                if !retryable || attempt > max_retries {
                    return Err(e);
                }
                let base_ms = crate::config::OrchestrationConfig::retry_base_backoff_ms();
                let delay = std::time::Duration::from_millis(base_ms * (1u64 << (attempt - 1)));
                tracing::warn!("Retry {}/{} after {:?}: {}", attempt, max_retries, delay, msg);
                tokio::time::sleep(delay).await;
            }
        }
    }
}

pub struct AnthropicClient {
    client: Client,
}

impl AnthropicClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { client }
    }

    pub async fn send_message(
        &self,
        provider: &ResolvedProvider,
        messages: Vec<AnthropicMessage>,
        system_prompt: Option<&str>,
        tools: Vec<ToolDefinition>,
        max_tokens: u32,
    ) -> Result<AnthropicResponse> {
        let base_url = provider.provider.base_url.trim_end_matches('/');
        let url = if base_url.contains("/v1") {
            format!("{}/messages", base_url)
        } else {
            format!("{}/v1/messages", base_url)
        };

        let mut body = json!({
            "model": provider.model.id,
            "max_tokens": max_tokens,
            "messages": messages,
        });

        if let Some(system) = system_prompt {
            body["system"] = json!(system);
        }

        if !tools.is_empty() {
            let tool_defs: Vec<Value> = tools.iter().map(|t| {
                json!({
                    "name": t.name,
                    "description": t.description,
                    "input_schema": t.input_schema,
                })
            }).collect();
            body["tools"] = json!(tool_defs);
        }

        let data: AnthropicResponse = retry_with_backoff(|| async {
            let response = self.client
                .post(&url)
                .header("x-api-key", &provider.provider.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Anthropic API error {}: {}", status, text);
            }

            response.json().await.map_err(|e| anyhow::anyhow!("{:#}", e))
        }).await?;

        Ok(data)
    }

    pub async fn send_message_stream(
        &self,
        provider: &ResolvedProvider,
        messages: Vec<AnthropicMessage>,
        system_prompt: Option<&str>,
        tools: Vec<ToolDefinition>,
        max_tokens: u32,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let base_url = provider.provider.base_url.trim_end_matches('/');
        let url = if base_url.contains("/v1") {
            format!("{}/messages", base_url)
        } else {
            format!("{}/v1/messages", base_url)
        };

        let mut body = json!({
            "model": provider.model.id,
            "max_tokens": max_tokens,
            "messages": messages,
            "stream": true,
        });

        if let Some(system) = system_prompt {
            body["system"] = json!(system);
        }

        if !tools.is_empty() {
            let tool_defs: Vec<Value> = tools.iter().map(|t| {
                json!({
                    "name": t.name,
                    "description": t.description,
                    "input_schema": t.input_schema,
                })
            }).collect();
            body["tools"] = json!(tool_defs);
        }

        let response = retry_with_backoff(|| async {
            let resp = self.client
                .post(&url)
                .header("x-api-key", &provider.provider.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await?;

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                anyhow::bail!("Anthropic API error {}: {}", status, text);
            }
            Ok(resp)
        }).await?;

        let stream = response.bytes_stream();
        let event_stream = stream
            .map(|chunk| {
                match chunk {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        Ok(text.to_string())
                    }
                    Err(e) => Err(anyhow::anyhow!("Stream error: {}", e)),
                }
            });

        Ok(Box::pin(event_stream))
    }

    pub async fn send_message_stream_with_thinking(
        &self,
        provider: &ResolvedProvider,
        messages: Vec<AnthropicMessage>,
        system_prompt: Option<&str>,
        tools: Vec<ToolDefinition>,
        max_tokens: u32,
        enable_thinking: bool,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let base_url = provider.provider.base_url.trim_end_matches('/');
        let url = if base_url.contains("/v1") {
            format!("{}/messages", base_url)
        } else {
            format!("{}/v1/messages", base_url)
        };

        let mut body = json!({
            "model": provider.model.id,
            "max_tokens": max_tokens,
            "messages": messages,
            "stream": true,
        });

        if let Some(system) = system_prompt {
            body["system"] = json!(system);
        }

        if !tools.is_empty() {
            let tool_defs: Vec<Value> = tools.iter().map(|t| {
                json!({
                    "name": t.name,
                    "description": t.description,
                    "input_schema": t.input_schema,
                })
            }).collect();
            body["tools"] = json!(tool_defs);
        }

        if enable_thinking {
            body["thinking"] = json!({
                "type": "enabled",
                "budget_tokens": 10000
            });
        }

        let response = retry_with_backoff(|| async {
            let resp = self.client
                .post(&url)
                .header("x-api-key", &provider.provider.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await?;

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                anyhow::bail!("Anthropic API error {}: {}", status, text);
            }
            Ok(resp)
        }).await?;

        let stream = response.bytes_stream();
        let event_stream = stream
            .map(|chunk| {
                match chunk {
                    Ok(bytes) => {
                        let text = String::from_utf8_lossy(&bytes);
                        Ok(text.to_string())
                    }
                    Err(e) => Err(anyhow::anyhow!("Stream error: {}", e)),
                }
            });

        Ok(Box::pin(event_stream))
    }
}

impl Default for AnthropicClient {
    fn default() -> Self {
        Self::new()
    }
}
