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
pub struct OpenAIMessage {
    pub role: String,
    pub content: OpenAIContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAIToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OpenAIContent {
    Text(String),
    Multi(Vec<OpenAIContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OpenAIContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    Image { image_url: ImageUrl },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<OpenAIChoice>,
    pub usage: Option<OpenAIUsage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIChoice {
    pub index: usize,
    pub message: OpenAIMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIStreamChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<OpenAIStreamChoice>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIStreamChoice {
    pub index: usize,
    pub delta: OpenAIDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIDelta {
    pub role: Option<String>,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAIToolCallDelta>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIToolCallDelta {
    pub index: usize,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub call_type: Option<String>,
    pub function: Option<FunctionCallDelta>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionCallDelta {
    pub name: Option<String>,
    pub arguments: Option<String>,
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

pub struct OpenAIClient {
    client: Client,
}

impl OpenAIClient {
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
        messages: Vec<OpenAIMessage>,
        system_prompt: Option<&str>,
        tools: Vec<ToolDefinition>,
        max_tokens: u32,
    ) -> Result<OpenAIResponse> {
        let base_url = crate::native_engine::provider_manager::ProviderManager::normalize_base_url(&provider.provider.base_url);
        let url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));

        let mut body_messages = Vec::new();
        
        if let Some(system) = system_prompt {
            body_messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: OpenAIContent::Text(system.to_string()),
                tool_calls: None,
                tool_call_id: None,
                reasoning_content: None,
            });
        }
        
        body_messages.extend(messages);

        let mut body = json!({
            "model": provider.model.id,
            "max_tokens": max_tokens,
            "messages": body_messages,
        });

        if !tools.is_empty() {
            let tool_defs: Vec<Value> = tools.iter().map(|t| {
                json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.input_schema,
                    }
                })
            }).collect();
            body["tools"] = json!(tool_defs);
        }

        let data: OpenAIResponse = retry_with_backoff(|| async {
            let response = self.client
                .post(&url)
                .header("Authorization", format!("Bearer {}", provider.provider.api_key))
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("OpenAI API error {}: {}", status, text);
            }

            response.json().await.map_err(|e| anyhow::anyhow!("{:#}", e))
        }).await?;

        Ok(data)
    }

    pub async fn send_message_stream(
        &self,
        provider: &ResolvedProvider,
        messages: Vec<OpenAIMessage>,
        system_prompt: Option<&str>,
        tools: Vec<ToolDefinition>,
        max_tokens: u32,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let base_url = crate::native_engine::provider_manager::ProviderManager::normalize_base_url(&provider.provider.base_url);
        let url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));

        let mut body_messages = Vec::new();
        
        if let Some(system) = system_prompt {
            body_messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: OpenAIContent::Text(system.to_string()),
                tool_calls: None,
                tool_call_id: None,
                reasoning_content: None,
            });
        }
        
        body_messages.extend(messages);

        let mut body = json!({
            "model": provider.model.id,
            "max_tokens": max_tokens,
            "messages": body_messages,
            "stream": true,
        });

        if !tools.is_empty() {
            let tool_defs: Vec<Value> = tools.iter().map(|t| {
                json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.input_schema,
                    }
                })
            }).collect();
            body["tools"] = json!(tool_defs);
        }

        let response = retry_with_backoff(|| async {
            let resp = self.client
                .post(&url)
                .header("Authorization", format!("Bearer {}", provider.provider.api_key))
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await?;

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                anyhow::bail!("OpenAI API error {}: {}", status, text);
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

impl Default for OpenAIClient {
    fn default() -> Self {
        Self::new()
    }
}
