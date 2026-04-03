//! 工作器管理模块

use super::types::*;
use crate::error::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 工作密钥
#[derive(Debug, Clone)]
pub struct WorkKey {
    /// 密钥ID
    pub id: String,
    /// 环境ID
    pub environment_id: String,
    /// 会话入口令牌
    pub session_token: String,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 过期时间
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl WorkKey {
    /// 创建新的工作密钥
    pub fn new(
        id: String,
        environment_id: String,
        session_token: String,
        ttl_secs: Option<u64>,
    ) -> Self {
        let created_at = chrono::Utc::now();
        let expires_at = ttl_secs.map(|ttl| {
            created_at + chrono::Duration::seconds(ttl as i64)
        });
        
        Self {
            id,
            environment_id,
            session_token,
            created_at,
            expires_at,
        }
    }
    
    /// 检查密钥是否过期
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now() >= expires_at
        } else {
            false
        }
    }
    
    /// 验证密钥
    pub fn validate(&self, environment_id: &str) -> Result<()> {
        if self.environment_id != environment_id {
            return Err(crate::error::ClaudeError::Bridge(
                "Environment ID mismatch".to_string()
            ));
        }
        
        if self.is_expired() {
            return Err(crate::error::ClaudeError::Bridge(
                "Work key expired".to_string()
            ));
        }
        
        Ok(())
    }
}

/// 工作器管理器
#[derive(Debug)]
pub struct WorkerManager {
    /// 活动工作密钥
    work_keys: Arc<RwLock<HashMap<String, WorkKey>>>,
    /// 环境密钥
    environment_secret: Arc<RwLock<Option<String>>>,
    /// 最大工作器数量
    max_workers: usize,
}

impl WorkerManager {
    /// 创建新的工作器管理器
    pub fn new(max_workers: usize) -> Self {
        Self {
            work_keys: Arc::new(RwLock::new(HashMap::new())),
            environment_secret: Arc::new(RwLock::new(None)),
            max_workers,
        }
    }
    
    /// 设置环境密钥
    pub async fn set_environment_secret(&self, secret: String) {
        let mut env_secret = self.environment_secret.write().await;
        *env_secret = Some(secret);
    }
    
    /// 获取环境密钥
    pub async fn get_environment_secret(&self) -> Option<String> {
        let env_secret = self.environment_secret.read().await;
        env_secret.clone()
    }
    
    /// 创建工作密钥
    pub async fn create_work_key(
        &self,
        work_response: &WorkResponse,
        work_secret: &WorkSecret,
    ) -> Result<WorkKey> {
        let mut keys = self.work_keys.write().await;
        
        if keys.len() >= self.max_workers {
            return Err(crate::error::ClaudeError::Bridge(
                "Maximum workers reached".to_string()
            ));
        }
        
        let key = WorkKey::new(
            work_response.id.clone(),
            work_response.environment_id.clone(),
            work_secret.session_ingress_token.clone(),
            Some(DEFAULT_SESSION_TIMEOUT_MS / 1000),
        );
        
        keys.insert(key.id.clone(), key.clone());
        
        Ok(key)
    }
    
    /// 获取工作密钥
    pub async fn get_work_key(&self, key_id: &str) -> Option<WorkKey> {
        let keys = self.work_keys.read().await;
        keys.get(key_id).cloned()
    }
    
    /// 移除工作密钥
    pub async fn remove_work_key(&self, key_id: &str) {
        let mut keys = self.work_keys.write().await;
        keys.remove(key_id);
    }
    
    /// 验证工作密钥
    pub async fn validate_work_key(&self, key_id: &str, environment_id: &str) -> Result<WorkKey> {
        let keys = self.work_keys.read().await;
        
        let key = keys.get(key_id).ok_or_else(|| {
            crate::error::ClaudeError::Bridge("Work key not found".to_string())
        })?;
        
        key.validate(environment_id)?;
        
        Ok(key.clone())
    }
    
    /// 获取活动工作器数量
    pub async fn active_worker_count(&self) -> usize {
        let keys = self.work_keys.read().await;
        keys.len()
    }
    
    /// 清理过期的工作密钥
    pub async fn cleanup_expired_keys(&self) {
        let mut keys = self.work_keys.write().await;
        keys.retain(|_, key| !key.is_expired());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_work_key() {
        let key = WorkKey::new(
            "test-key".to_string(),
            "test-env".to_string(),
            "test-token".to_string(),
            Some(3600),
        );
        
        assert_eq!(key.id, "test-key");
        assert!(!key.is_expired());
        
        let result = key.validate("test-env");
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_worker_manager() {
        let manager = WorkerManager::new(10);
        
        manager.set_environment_secret("test-secret".to_string()).await;
        
        let secret = manager.get_environment_secret().await;
        assert_eq!(secret, Some("test-secret".to_string()));
        
        assert_eq!(manager.active_worker_count().await, 0);
    }
}
