//! Bridge 管理器

use super::types::*;
use super::session::SessionManager;
use super::auth::AuthManager;
use super::worker::WorkerManager;
use super::connection::ConnectionManager;
use crate::error::Result;
use crate::config::Config;
use crate::state::AppState;
use std::path::PathBuf;
use std::sync::Arc;

/// Bridge 管理器
pub struct BridgeManager {
    /// 配置
    config: Config,
    /// 应用状态
    state: AppState,
    /// Bridge 配置
    bridge_config: BridgeConfig,
    /// 会话管理器
    session_manager: Arc<SessionManager>,
    /// 认证管理器
    auth_manager: Arc<AuthManager>,
    /// 工作器管理器
    worker_manager: Arc<WorkerManager>,
    /// 连接管理器
    connection_manager: Arc<ConnectionManager>,
}

impl BridgeManager {
    /// 创建新的 Bridge 管理器
    pub fn new(config: Config, state: AppState) -> Self {
        let bridge_config = BridgeConfig::default();
        
        let session_manager = Arc::new(SessionManager::new(
            bridge_config.spawn_mode,
            bridge_config.max_sessions,
            PathBuf::from(&bridge_config.dir),
        ));
        
        let auth_manager = Arc::new(AuthManager::new());
        let worker_manager = Arc::new(WorkerManager::new(bridge_config.max_sessions));
        let connection_manager = Arc::new(ConnectionManager::new());
        
        Self {
            config,
            state,
            bridge_config,
            session_manager,
            auth_manager,
            worker_manager,
            connection_manager,
        }
    }
    
    /// 使用 Bridge 配置创建
    pub fn with_bridge_config(
        config: Config,
        state: AppState,
        bridge_config: BridgeConfig,
    ) -> Self {
        let session_manager = Arc::new(SessionManager::new(
            bridge_config.spawn_mode,
            bridge_config.max_sessions,
            PathBuf::from(&bridge_config.dir),
        ));
        
        let auth_manager = Arc::new(AuthManager::new());
        let worker_manager = Arc::new(WorkerManager::new(bridge_config.max_sessions));
        let connection_manager = Arc::new(ConnectionManager::new());
        
        Self {
            config,
            state,
            bridge_config,
            session_manager,
            auth_manager,
            worker_manager,
            connection_manager,
        }
    }
    
    /// 初始化 Bridge
    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing Bridge manager");
        
        self.connection_manager.start_heartbeat().await;
        
        tracing::info!("Bridge manager initialized successfully");
        Ok(())
    }
    
    /// 启动 Bridge 服务器
    pub async fn start_server(&mut self) -> Result<()> {
        tracing::info!("Starting Bridge server");
        
        self.initialize().await?;
        
        tracing::info!("Bridge server started on environment {}", self.bridge_config.environment_id);
        
        Ok(())
    }
    
    /// 连接到 Bridge 服务器
    pub async fn connect_client(&mut self) -> Result<()> {
        tracing::info!("Connecting to Bridge server");
        
        self.initialize().await?;
        
        tracing::info!("Connected to Bridge server");
        
        Ok(())
    }
    
    /// 注册环境
    pub async fn register_environment(&self) -> Result<(String, String)> {
        tracing::info!("Registering Bridge environment");
        
        let environment_id = self.bridge_config.environment_id.clone();
        let environment_secret = uuid::Uuid::new_v4().to_string();
        
        self.worker_manager.set_environment_secret(environment_secret.clone()).await;
        
        Ok((environment_id, environment_secret))
    }
    
    /// 轮询工作
    pub async fn poll_for_work(&self) -> Result<Option<WorkResponse>> {
        tracing::debug!("Polling for work");
        
        Ok(None)
    }
    
    /// 确认工作
    pub async fn acknowledge_work(
        &self,
        work_id: &str,
        session_token: &str,
    ) -> Result<()> {
        tracing::info!("Acknowledging work: {}", work_id);
        
        Ok(())
    }
    
    /// 停止工作
    pub async fn stop_work(&self, work_id: &str, force: bool) -> Result<()> {
        tracing::info!("Stopping work: {} (force={})", work_id, force);
        
        self.worker_manager.remove_work_key(work_id).await;
        
        Ok(())
    }
    
    /// 注销环境
    pub async fn deregister_environment(&self) -> Result<()> {
        tracing::info!("Deregistering Bridge environment");
        
        self.worker_manager.set_environment_secret(String::new()).await;
        
        Ok(())
    }
    
    /// 获取会话管理器
    pub fn session_manager(&self) -> Arc<SessionManager> {
        Arc::clone(&self.session_manager)
    }
    
    /// 获取认证管理器
    pub fn auth_manager(&self) -> Arc<AuthManager> {
        Arc::clone(&self.auth_manager)
    }
    
    /// 获取工作器管理器
    pub fn worker_manager(&self) -> Arc<WorkerManager> {
        Arc::clone(&self.worker_manager)
    }
    
    /// 获取连接管理器
    pub fn connection_manager(&self) -> Arc<ConnectionManager> {
        Arc::clone(&self.connection_manager)
    }
    
    /// 获取 Bridge 配置
    pub fn bridge_config(&self) -> &BridgeConfig {
        &self.bridge_config
    }
    
    /// 关闭 Bridge
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down Bridge manager");
        
        self.connection_manager.cleanup_timeout_connections().await;
        
        tracing::info!("Bridge manager shutdown complete");
        Ok(())
    }
}

/// Bridge 命令入口
pub async fn run(config: Config, state: AppState) -> Result<()> {
    let mut manager = BridgeManager::new(config, state);
    
    manager.start_server().await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_bridge_manager() {
        let config = Config::default();
        let state = crate::state::new_app_state();
        
        let manager = BridgeManager::new(config, state);
        
        assert_eq!(manager.bridge_config().spawn_mode, SpawnMode::SingleSession);
    }
    
    #[tokio::test]
    async fn test_register_environment() {
        let config = Config::default();
        let state = crate::state::new_app_state();
        
        let manager = BridgeManager::new(config, state);
        
        let (env_id, env_secret) = manager.register_environment().await.unwrap();
        
        assert!(!env_id.is_empty());
        assert!(!env_secret.is_empty());
    }
}
