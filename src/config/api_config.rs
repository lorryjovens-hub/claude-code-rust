//! API Configuration

use serde::{Deserialize, Serialize};

/// Anthropic API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// API key (can be set via environment variable)
    pub api_key: Option<String>,
    /// Base URL for API requests
    pub base_url: String,
    /// Maximum tokens per request
    pub max_tokens: usize,
    /// Request timeout in seconds
    pub timeout: u64,
    /// Enable streaming responses
    pub streaming: bool,
    /// Beta headers to include
    pub beta_headers: Vec<String>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("ANTHROPIC_API_KEY").ok()
                .or(std::env::var("DASHSCOPE_API_KEY").ok())
                .or(std::env::var("DEEPSEEK_API_KEY").ok()),
            base_url: std::env::var("API_BASE_URL")
                .unwrap_or_else(|_| "https://api.anthropic.com".to_string()),
            max_tokens: 4096,
            timeout: 120,
            streaming: true,
            beta_headers: vec![],
        }
    }
}

impl ApiConfig {
    /// Get the API key, checking environment variable first
    pub fn get_api_key(&self) -> Option<String> {
        std::env::var("ANTHROPIC_API_KEY").ok()
            .or(std::env::var("DASHSCOPE_API_KEY").ok())
            .or(std::env::var("DEEPSEEK_API_KEY").ok())
            .or(self.api_key.clone())
    }

    /// Get the base URL, checking environment variable first
    pub fn get_base_url(&self) -> String {
        std::env::var("API_BASE_URL")
            .unwrap_or_else(|_| self.base_url.clone())
    }

    /// Get the model ID for the given model name
    pub fn get_model_id(&self, model: &str) -> String {
        match model {
            "opus" => "claude-3-opus-20240229".to_string(),
            "sonnet" => "claude-3-5-sonnet-20241022".to_string(),
            "haiku" => "claude-3-5-haiku-20241022".to_string(),
            _ => model.to_string(),
        }
    }
}