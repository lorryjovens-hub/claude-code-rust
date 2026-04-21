//! Remote Execution Support

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    pub default_timeout_secs: u32,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub enable_caching: bool,
    pub cache_ttl_secs: u64,
}

impl Default for RemoteConfig {
    fn default() -> Self {
        Self {
            default_timeout_secs: 60,
            max_retries: 3,
            retry_delay_ms: 1000,
            enable_caching: true,
            cache_ttl_secs: 300,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteRequest {
    pub id: String,
    pub endpoint: String,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub timeout_secs: Option<u32>,
}

impl RemoteRequest {
    pub fn new(method: HttpMethod, endpoint: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            endpoint: endpoint.to_string(),
            method,
            headers: HashMap::new(),
            body: None,
            timeout_secs: None,
        }
    }
    
    pub fn get(endpoint: &str) -> Self {
        Self::new(HttpMethod::Get, endpoint)
    }
    
    pub fn post(endpoint: &str) -> Self {
        Self::new(HttpMethod::Post, endpoint)
    }
    
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
    
    pub fn with_body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }
    
    pub fn with_json_body(mut self, body: &serde_json::Value) -> Self {
        self.body = Some(body.to_string());
        self
    }
    
    pub fn with_timeout(mut self, secs: u32) -> Self {
        self.timeout_secs = Some(secs);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Patch => write!(f, "PATCH"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteResult {
    pub request_id: String,
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub duration_ms: u64,
    pub cached: bool,
    pub timestamp: DateTime<Utc>,
}

impl RemoteResult {
    pub fn is_success(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }
    
    pub fn json<T: for<'de> Deserialize<'de>>(&self) -> anyhow::Result<T> {
        Ok(serde_json::from_str(&self.body)?)
    }
}

struct CacheEntry {
    result: RemoteResult,
    cached_at: DateTime<Utc>,
}

pub struct RemoteExecutor {
    config: RemoteConfig,
    client: reqwest::Client,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

impl RemoteExecutor {
    pub fn new(config: RemoteConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.default_timeout_secs as u64))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        
        Self {
            config,
            client,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn execute(&self, request: RemoteRequest) -> anyhow::Result<RemoteResult> {
        let cache_key = self.cache_key(&request);
        
        if self.config.enable_caching && request.method == HttpMethod::Get {
            if let Some(cached) = self.get_cached(&cache_key).await {
                return Ok(cached);
            }
        }
        
        let start = std::time::Instant::now();
        
        let mut retries = 0;
        let mut last_error = None;
        
        while retries <= self.config.max_retries {
            match self.execute_request(&request).await {
                Ok(result) => {
                    let result = RemoteResult {
                        request_id: request.id.clone(),
                        status_code: result.status().as_u16(),
                        headers: result.headers().iter()
                            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                            .collect(),
                        body: result.text().await.unwrap_or_default(),
                        duration_ms: start.elapsed().as_millis() as u64,
                        cached: false,
                        timestamp: Utc::now(),
                    };
                    
                    if self.config.enable_caching && request.method == HttpMethod::Get {
                        self.set_cached(&cache_key, &result).await;
                    }
                    
                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e);
                    retries += 1;
                    if retries <= self.config.max_retries {
                        tokio::time::sleep(
                            std::time::Duration::from_millis(self.config.retry_delay_ms)
                        ).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Request failed")))
    }
    
    async fn execute_request(&self, request: &RemoteRequest) -> anyhow::Result<reqwest::Response> {
        let mut req = match request.method {
            HttpMethod::Get => self.client.get(&request.endpoint),
            HttpMethod::Post => self.client.post(&request.endpoint),
            HttpMethod::Put => self.client.put(&request.endpoint),
            HttpMethod::Delete => self.client.delete(&request.endpoint),
            HttpMethod::Patch => self.client.patch(&request.endpoint),
        };
        
        for (key, value) in &request.headers {
            req = req.header(key, value);
        }
        
        if let Some(body) = &request.body {
            req = req.body(body.clone());
        }
        
        if let Some(timeout) = request.timeout_secs {
            req = req.timeout(std::time::Duration::from_secs(timeout as u64));
        }
        
        Ok(req.send().await?)
    }
    
    fn cache_key(&self, request: &RemoteRequest) -> String {
        format!("{}:{}", request.method, request.endpoint)
    }
    
    async fn get_cached(&self, key: &str) -> Option<RemoteResult> {
        let cache = self.cache.read().await;
        if let Some(entry) = cache.get(key) {
            let age = (Utc::now() - entry.cached_at).num_seconds() as u64;
            if age < self.config.cache_ttl_secs {
                let mut result = entry.result.clone();
                result.cached = true;
                return Some(result);
            }
        }
        None
    }
    
    async fn set_cached(&self, key: &str, result: &RemoteResult) {
        let mut cache = self.cache.write().await;
        cache.insert(key.to_string(), CacheEntry {
            result: result.clone(),
            cached_at: Utc::now(),
        });
    }
    
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
    
    pub async fn get(&self, endpoint: &str) -> anyhow::Result<RemoteResult> {
        self.execute(RemoteRequest::get(endpoint)).await
    }
    
    pub async fn post(&self, endpoint: &str, body: &serde_json::Value) -> anyhow::Result<RemoteResult> {
        self.execute(
            RemoteRequest::post(endpoint)
                .with_json_body(body)
        ).await
    }
    
    pub async fn post_json<T: for<'de> Deserialize<'de>>(
        &self, 
        endpoint: &str, 
        body: &serde_json::Value
    ) -> anyhow::Result<T> {
        let result = self.post(endpoint, body).await?;
        result.json()
    }
}

impl Default for RemoteExecutor {
    fn default() -> Self {
        Self::new(Default::default())
    }
}
