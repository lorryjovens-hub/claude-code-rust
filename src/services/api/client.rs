//! HTTP 客户端
//! 
//! 这个模块实现了通用 HTTP 客户端功能

use crate::error::Result;
use reqwest::Client;

/// HTTP 客户端
pub struct ApiClient {
    /// reqwest 客户端
    client: Client,
    
    /// 基础 URL
    base_url: String,
}

impl ApiClient {
    /// 创建新的 HTTP 客户端
    pub fn new(base_url: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| crate::error::ClaudeError::Network(e))?;
        
        Ok(Self { client, base_url })
    }
    
    /// GET 请求
    pub async fn get(&self, path: &str) -> Result<String> {
        let url = format!("{}/{}", self.base_url, path);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| crate::error::ClaudeError::Network(e))?;
        
        let text = response.text().await
            .map_err(|e| crate::error::ClaudeError::Network(e))?;
        
        Ok(text)
    }
    
    /// POST 请求
    pub async fn post(&self, path: &str, body: &str) -> Result<String> {
        let url = format!("{}/{}", self.base_url, path);
        
        let response = self.client
            .post(&url)
            .body(body.to_string())
            .send()
            .await
            .map_err(|e| crate::error::ClaudeError::Network(e))?;
        
        let text = response.text().await
            .map_err(|e| crate::error::ClaudeError::Network(e))?;
        
        Ok(text)
    }
    
    /// 获取基础 URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_client_creation() {
        let client = ApiClient::new("https://api.example.com".to_string());
        assert!(client.is_ok());
    }
}
