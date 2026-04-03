//! 代理执行器
//! 
//! 这个模块实现了代理执行和管理功能

use crate::error::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use super::types::{AgentDefinition, AgentConfig, AgentResult, AgentId, AgentStatus};

/// 代理执行器
pub struct AgentExecutor {
    /// 代理配置
    config: AgentConfig,
    
    /// 代理 ID
    agent_id: AgentId,
    
    /// 执行状态
    status: Arc<RwLock<AgentStatus>>,
}

impl AgentExecutor {
    /// 创建新的代理执行器
    pub fn new(config: AgentConfig) -> Self {
        let agent_id = uuid::Uuid::new_v4().to_string();
        
        Self {
            config,
            agent_id,
            status: Arc::new(RwLock::new(AgentStatus::Idle)),
        }
    }
    
    /// 获取代理 ID
    pub fn id(&self) -> &str {
        &self.agent_id
    }
    
    /// 获取代理配置
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }
    
    /// 获取代理状态
    pub async fn status(&self) -> AgentStatus {
        *self.status.read().await
    }
    
    /// 执行代理
    pub async fn execute(&self, input: String) -> Result<AgentResult> {
        // 更新状态为运行中
        *self.status.write().await = AgentStatus::Running;
        
        // TODO: 实现实际的代理执行逻辑
        // 1. 准备系统提示
        // 2. 准备工具集
        // 3. 调用 API
        // 4. 处理响应
        
        // 模拟执行
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // 更新状态为完成
        *self.status.write().await = AgentStatus::Completed;
        
        Ok(AgentResult {
            agent_id: self.agent_id.clone(),
            messages: vec![format!("Processed: {}", input)],
            usage: Default::default(),
            status: AgentStatus::Completed,
            error: None,
        })
    }
    
    /// 取消代理
    pub async fn cancel(&self) -> Result<()> {
        *self.status.write().await = AgentStatus::Cancelled;
        Ok(())
    }
}

impl std::fmt::Debug for AgentExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentExecutor")
            .field("agent_id", &self.agent_id)
            .field("status", &self.status.try_read())
            .finish_non_exhaustive()
    }
}

/// 代理管理器
pub struct AgentManager {
    /// 已注册的代理定义
    agents: Arc<RwLock<HashMap<String, AgentDefinition>>>,
    
    /// 活动的代理执行器
    executors: Arc<RwLock<HashMap<AgentId, Arc<AgentExecutor>>>>,
}

impl AgentManager {
    /// 创建新的代理管理器
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            executors: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 注册代理
    pub async fn register(&self, definition: AgentDefinition) {
        self.agents.write().await.insert(definition.name.clone(), definition);
    }
    
    /// 注销代理
    pub async fn unregister(&self, name: &str) -> Option<AgentDefinition> {
        self.agents.write().await.remove(name)
    }
    
    /// 获取代理定义
    pub async fn get(&self, name: &str) -> Option<AgentDefinition> {
        self.agents.read().await.get(name).cloned()
    }
    
    /// 列出所有代理
    pub async fn list(&self) -> Vec<AgentDefinition> {
        self.agents.read().await.values().cloned().collect()
    }
    
    /// 创建代理执行器
    pub async fn create_executor(&self, name: &str) -> Option<Arc<AgentExecutor>> {
        let definition = self.get(name).await?;
        let config = AgentConfig::new(definition);
        let executor = Arc::new(AgentExecutor::new(config));
        
        let agent_id = executor.id().to_string();
        self.executors.write().await.insert(agent_id, executor.clone());
        
        Some(executor)
    }
    
    /// 获取执行器
    pub async fn get_executor(&self, agent_id: &str) -> Option<Arc<AgentExecutor>> {
        self.executors.read().await.get(agent_id).cloned()
    }
    
    /// 移除执行器
    pub async fn remove_executor(&self, agent_id: &str) -> Option<Arc<AgentExecutor>> {
        self.executors.write().await.remove(agent_id)
    }
    
    /// 清理已完成的执行器
    pub async fn cleanup(&self) {
        let mut executors = self.executors.write().await;
        executors.retain(|_, executor| {
            executor.status.try_read().map(|s| *s != AgentStatus::Completed && *s != AgentStatus::Cancelled && *s != AgentStatus::Error).unwrap_or(false)
        });
    }
}

impl Default for AgentManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for AgentManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentManager")
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::AgentType;
    
    #[tokio::test]
    async fn test_agent_executor() {
        let definition = AgentDefinition::new(
            "test".to_string(),
            AgentType::GeneralPurpose,
            "Test agent".to_string(),
        );
        
        let config = AgentConfig::new(definition);
        let executor = AgentExecutor::new(config);
        
        assert_eq!(executor.status().await, AgentStatus::Idle);
        
        let result = executor.execute("test input".to_string()).await.unwrap();
        assert_eq!(result.status, AgentStatus::Completed);
    }
    
    #[tokio::test]
    async fn test_agent_manager() {
        let manager = AgentManager::new();
        
        let definition = AgentDefinition::new(
            "test".to_string(),
            AgentType::GeneralPurpose,
            "Test agent".to_string(),
        );
        
        manager.register(definition).await;
        
        let retrieved = manager.get("test").await;
        assert!(retrieved.is_some());
        
        let executor = manager.create_executor("test").await;
        assert!(executor.is_some());
    }
}
