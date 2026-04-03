//! JWT 认证模块

use super::types::*;
use crate::error::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, Duration};

/// JWT 令牌
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtToken {
    /// 令牌字符串
    pub token: String,
    /// 过期时间
    pub expires_at: Option<DateTime<Utc>>,
    /// 载荷
    pub payload: Option<JwtPayload>,
}

/// JWT 载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtPayload {
    /// 过期时间（Unix 时间戳）
    pub exp: Option<i64>,
    /// 签发时间（Unix 时间戳）
    pub iat: Option<i64>,
    /// 签发者
    pub iss: Option<String>,
    /// 主题
    pub sub: Option<String>,
    /// 其他声明
    #[serde(flatten)]
    pub claims: HashMap<String, serde_json::Value>,
}

impl JwtToken {
    /// 从字符串解析 JWT 令牌
    pub fn parse(token_str: &str) -> Result<Self> {
        let token = if token_str.starts_with("sk-ant-si-") {
            &token_str["sk-ant-si-".len()..]
        } else {
            token_str
        };
        
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(crate::error::ClaudeError::Bridge(
                "Invalid JWT format".to_string()
            ));
        }
        
        let payload_json = URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|e| crate::error::ClaudeError::Bridge(
                format!("Failed to decode JWT: {}", e)
            ))?;
        
        let payload: JwtPayload = serde_json::from_slice(&payload_json)?;
        
        let expires_at = payload.exp
            .map(|exp| DateTime::from_timestamp(exp, 0).unwrap_or_else(|| Utc::now()));
        
        Ok(Self {
            token: token_str.to_string(),
            expires_at,
            payload: Some(payload),
        })
    }
    
    /// 检查令牌是否过期
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() >= expires_at
        } else {
            false
        }
    }
    
    /// 检查令牌是否即将过期（在指定缓冲时间内）
    pub fn is_expiring_soon(&self, buffer_secs: i64) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = Utc::now();
            let buffer = chrono::Duration::seconds(buffer_secs);
            now + buffer >= expires_at
        } else {
            false
        }
    }
    
    /// 获取剩余有效时间（秒）
    pub fn remaining_secs(&self) -> Option<i64> {
        self.expires_at.map(|expires_at| {
            let now = Utc::now();
            (expires_at - now).num_seconds().max(0)
        })
    }
}

/// 令牌刷新调度器
#[derive(Debug)]
pub struct TokenRefreshScheduler {
    /// 定时器映射
    timers: Arc<RwLock<HashMap<String, tokio::time::Interval>>>,
    /// 失败计数
    failure_counts: Arc<RwLock<HashMap<String, u32>>>,
    /// 代数计数器（用于取消过期的刷新操作）
    generations: Arc<RwLock<HashMap<String, u64>>>,
    /// 刷新缓冲时间（毫秒）
    refresh_buffer_ms: u64,
    /// 最大失败次数
    max_failures: u32,
    /// 重试延迟（毫秒）
    retry_delay_ms: u64,
}

impl TokenRefreshScheduler {
    /// 创建新的令牌刷新调度器
    pub fn new() -> Self {
        Self {
            timers: Arc::new(RwLock::new(HashMap::new())),
            failure_counts: Arc::new(RwLock::new(HashMap::new())),
            generations: Arc::new(RwLock::new(HashMap::new())),
            refresh_buffer_ms: 5 * 60 * 1000, // 5 分钟
            max_failures: 3,
            retry_delay_ms: 60 * 1000, // 1 分钟
        }
    }
    
    /// 调度令牌刷新
    pub async fn schedule(&self, session_id: String, token: String) -> Result<()> {
        let jwt = JwtToken::parse(&token)?;
        
        if let Some(remaining_secs) = jwt.remaining_secs() {
            let delay_secs = (remaining_secs * 1000 - self.refresh_buffer_ms as i64).max(30) as u64;
            
            tracing::info!(
                "Scheduling token refresh for session {} in {} seconds",
                session_id,
                delay_secs
            );
            
            self.schedule_refresh(session_id, delay_secs).await?;
        }
        
        Ok(())
    }
    
    /// 使用过期时间调度刷新
    pub async fn schedule_from_expires_in(
        &self,
        session_id: String,
        expires_in_secs: u64,
    ) -> Result<()> {
        let delay_secs = (expires_in_secs * 1000 - self.refresh_buffer_ms).max(30_000) / 1000;
        
        tracing::info!(
            "Scheduling token refresh for session {} in {} seconds (expires_in={})",
            session_id,
            delay_secs,
            expires_in_secs
        );
        
        self.schedule_refresh(session_id, delay_secs).await?;
        
        Ok(())
    }
    
    /// 调度刷新操作
    async fn schedule_refresh(&self, session_id: String, delay_secs: u64) -> Result<()> {
        let mut timers = self.timers.write().await;
        
        let mut interval = interval(Duration::from_secs(delay_secs));
        interval.tick().await; // 立即触发第一次
        
        timers.insert(session_id.clone(), interval);
        
        // 增加代数
        let mut generations = self.generations.write().await;
        let gen = generations.entry(session_id.clone()).or_insert(0);
        *gen += 1;
        
        Ok(())
    }
    
    /// 取消调度
    pub async fn cancel(&self, session_id: &str) {
        let mut timers = self.timers.write().await;
        timers.remove(session_id);
        
        let mut failure_counts = self.failure_counts.write().await;
        failure_counts.remove(session_id);
        
        let mut generations = self.generations.write().await;
        generations.remove(session_id);
    }
    
    /// 取消所有调度
    pub async fn cancel_all(&self) {
        let mut timers = self.timers.write().await;
        timers.clear();
        
        let mut failure_counts = self.failure_counts.write().await;
        failure_counts.clear();
        
        let mut generations = self.generations.write().await;
        generations.clear();
    }
}

impl Default for TokenRefreshScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// 认证管理器
#[derive(Debug)]
pub struct AuthManager {
    /// OAuth 访问令牌
    oauth_token: Arc<RwLock<Option<String>>>,
    /// 令牌刷新调度器
    refresh_scheduler: TokenRefreshScheduler,
}

impl AuthManager {
    /// 创建新的认证管理器
    pub fn new() -> Self {
        Self {
            oauth_token: Arc::new(RwLock::new(None)),
            refresh_scheduler: TokenRefreshScheduler::new(),
        }
    }
    
    /// 设置 OAuth 访问令牌
    pub async fn set_oauth_token(&self, token: String) {
        let mut oauth_token = self.oauth_token.write().await;
        *oauth_token = Some(token);
    }
    
    /// 获取 OAuth 访问令牌
    pub async fn get_oauth_token(&self) -> Option<String> {
        let oauth_token = self.oauth_token.read().await;
        oauth_token.clone()
    }
    
    /// 获取刷新调度器
    pub fn refresh_scheduler(&self) -> &TokenRefreshScheduler {
        &self.refresh_scheduler
    }
    
    /// 验证令牌
    pub fn validate_token(&self, token: &str) -> Result<JwtToken> {
        JwtToken::parse(token)
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_jwt_token_parse() {
        let token_str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE3NzUxOTQ3NjIsImlhdCI6MTc3NTE5Mzc2Miwic3ViIjoidGVzdC11c2VyIn0.test-signature";
        
        let result = JwtToken::parse(token_str);
        assert!(result.is_ok());
        
        let token = result.unwrap();
        assert!(token.payload.is_some());
        assert!(token.expires_at.is_some());
    }
    
    #[tokio::test]
    async fn test_auth_manager() {
        let manager = AuthManager::new();
        
        manager.set_oauth_token("test-token".to_string()).await;
        
        let token = manager.get_oauth_token().await;
        assert_eq!(token, Some("test-token".to_string()));
    }
    
    #[tokio::test]
    async fn test_token_refresh_scheduler() {
        let scheduler = TokenRefreshScheduler::new();
        
        scheduler.schedule_from_expires_in("test-session".to_string(), 3600).await.unwrap();
        
        scheduler.cancel("test-session").await;
    }
}
