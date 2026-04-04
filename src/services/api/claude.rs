//! Claude API 客户端
//!
//! 这个模块实现了 Claude API 客户端功能

use crate::api::{ApiClient, ApiClientConfig};
use crate::api::error::ApiError;
use crate::error::{Result, ClaudeError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Claude API 客户端
pub struct ClaudeApi {
    /// API 密钥
    api_key: Option<String>,

    /// 基础 URL
    base_url: String,

    /// 模型
    model: String,

    /// HTTP 客户端
    client: ApiClient,
}

impl ClaudeApi {
    /// 创建新的 Claude API 客户端
    pub fn new() -> Self {
        let base_url = "https://api.anthropic.com".to_string();
        let config = ApiClientConfig::default();
        let client = ApiClient::new(&base_url, config);

        Self {
            api_key: None,
            base_url,
            model: "claude-3-5-sonnet-20241022".to_string(),
            client,
        }
    }
    
    /// 设置 API 密钥
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key.clone());
        self.client = self.client.with_api_key(&api_key);
        self
    }
    
    /// 设置基础 URL
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url.clone();
        let config = ApiClientConfig::default();
        let mut client = ApiClient::new(&base_url, config);
        if let Some(ref api_key) = self.api_key {
            client = client.with_api_key(api_key);
        }
        self.client = client;
        self
    }
    
    /// 设置模型
    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }
    
    /// 发送消息
    pub async fn send_message(&self, message: &str) -> Result<String> {
        // 构建请求
        let request = ClaudeRequest {
            model: self.model.clone(),
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: message.to_string(),
            }],
            max_tokens: Some(1024),
        };

        // 构建请求头
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());
        if let Some(ref api_key) = self.api_key {
            headers.insert("x-api-key".to_string(), api_key.clone());
        }

        // 创建API请求
        let api_request = crate::api::Request::post("/v1/messages", request)
            .with_headers(headers)
            .with_auth(false); // 禁用默认的Authorization头

        // 发送请求并处理API错误
        let response = self.client.send::<ClaudeRequest, ClaudeResponse>(api_request)
            .await
            .map_err(|e| {
                match e {
                    ApiError::Network(err) => ClaudeError::Network(err),
                    ApiError::Http { status, message } => ClaudeError::Other(format!("HTTP error {}: {}", status, message)),
                    ApiError::Timeout => ClaudeError::Other("Request timeout".to_string()),
                    ApiError::Serialization(err) => ClaudeError::Serialization(err),
                    ApiError::Compression(err) => ClaudeError::Other(format!("Compression error: {}", err)),
                    ApiError::Business { code, message } => ClaudeError::Other(format!("Business error [{}]: {}", code, message)),
                    ApiError::Other(msg) => ClaudeError::Other(msg),
                }
            })?;

        // 解析响应
        match response.body {
            Some(claude_response) => {
                // 提取文本内容
                let text = claude_response.content
                    .iter()
                    .filter_map(|c| c.text.as_ref())
                    .map(|t| t.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                Ok(text)
            },
            None => Err(crate::error::ClaudeError::Other("Empty response body".to_string()))?,
        }
    }
    
    /// 获取模型
    pub fn model(&self) -> &str {
        &self.model
    }
    
    /// 获取基础 URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl Default for ClaudeApi {
    fn default() -> Self {
        Self::new()
    }
}

/// Claude API 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMessage {
    /// 角色
    pub role: String,
    
    /// 内容
    pub content: String,
}

/// Claude API 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeRequest {
    /// 模型
    pub model: String,
    
    /// 消息列表
    pub messages: Vec<ClaudeMessage>,
    
    /// 最大 token 数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

/// Claude API 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeResponse {
    /// ID
    pub id: String,
    
    /// 类型
    #[serde(rename = "type")]
    pub response_type: String,
    
    /// 内容
    pub content: Vec<ClaudeContent>,
    
    /// 模型
    pub model: String,
    
    /// 使用的 token 数
    pub usage: ClaudeUsage,
}

/// Claude API 内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeContent {
    /// 类型
    #[serde(rename = "type")]
    pub content_type: String,
    
    /// 文本
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// Claude API 使用统计
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClaudeUsage {
    /// 输入 token 数
    pub input_tokens: u32,
    
    /// 输出 token 数
    pub output_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_claude_api_creation() {
        let api = ClaudeApi::new()
            .with_api_key("test-key".to_string())
            .with_model("claude-3-opus".to_string());
        
        assert_eq!(api.model(), "claude-3-opus");
        assert!(api.api_key.is_some());
    }
    
    #[test]
    fn test_claude_message() {
        let message = ClaudeMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        };
        
        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("\"role\":\"user\""));
    }
}
