//! MCP 连接管理器
//! 
//! 这个模块实现了 MCP 连接管理功能

use crate::error::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use super::types::{McpServerConfig, McpServerConnection, McpConnectionStatus};

/// MCP 连接管理器
pub struct McpConnectionManager {
    /// 服务器连接映射
    connections: Arc<RwLock<HashMap<String, McpServerConnection>>>,
    
    /// 配置映射
    configs: Arc<RwLock<HashMap<String, McpServerConfig>>>,
}

impl McpConnectionManager {
    /// 创建新的 MCP 连接管理器
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 注册服务器配置
    pub async fn register_server(&self, name: String, config: McpServerConfig) {
        self.configs.write().await.insert(name.clone(), config);
        
        // 创建连接对象
        let connection = McpServerConnection {
            name: name.clone(),
            config: self.configs.read().await.get(&name).unwrap().clone(),
            status: McpConnectionStatus::Disconnected,
            error: None,
        };
        
        self.connections.write().await.insert(name, connection);
    }
    
    /// 注销服务器
    pub async fn unregister_server(&self, name: &str) -> Option<McpServerConnection> {
        self.configs.write().await.remove(name);
        self.connections.write().await.remove(name)
    }
    
    /// 连接服务器
    pub async fn connect_server(&self, name: &str) -> Result<()> {
        let mut connections = self.connections.write().await;
        
        if let Some(connection) = connections.get_mut(name) {
            connection.status = McpConnectionStatus::Connecting;
            
            // TODO: 实现实际的连接逻辑
            // 根据配置类型建立连接
            
            connection.status = McpConnectionStatus::Connected;
            connection.error = None;
        }
        
        Ok(())
    }
    
    /// 断开服务器
    pub async fn disconnect_server(&self, name: &str) -> Result<()> {
        let mut connections = self.connections.write().await;
        
        if let Some(connection) = connections.get_mut(name) {
            // TODO: 实现实际的断开逻辑
            
            connection.status = McpConnectionStatus::Disconnected;
            connection.error = None;
        }
        
        Ok(())
    }
    
    /// 重连服务器
    pub async fn reconnect_server(&self, name: &str) -> Result<()> {
        self.disconnect_server(name).await?;
        self.connect_server(name).await
    }
    
    /// 切换服务器状态
    pub async fn toggle_server(&self, name: &str) -> Result<()> {
        let connections = self.connections.read().await;
        
        if let Some(connection) = connections.get(name) {
            let status = connection.status;
            drop(connections);
            
            match status {
                McpConnectionStatus::Connected => {
                    self.disconnect_server(name).await?;
                }
                McpConnectionStatus::Disconnected => {
                    self.connect_server(name).await?;
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// 获取服务器连接
    pub async fn get_connection(&self, name: &str) -> Option<McpServerConnection> {
        self.connections.read().await.get(name).cloned()
    }
    
    /// 获取所有连接
    pub async fn list_connections(&self) -> Vec<McpServerConnection> {
        self.connections.read().await.values().cloned().collect()
    }
    
    /// 获取连接状态
    pub async fn get_status(&self, name: &str) -> Option<McpConnectionStatus> {
        self.connections.read().await.get(name).map(|c| c.status)
    }
    
    /// 检查服务器是否已连接
    pub async fn is_connected(&self, name: &str) -> bool {
        self.get_status(name).await == Some(McpConnectionStatus::Connected)
    }
}

impl Default for McpConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for McpConnectionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpConnectionManager")
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::McpStdioConfig;
    
    #[tokio::test]
    async fn test_register_server() {
        let manager = McpConnectionManager::new();
        
        let config = McpServerConfig::Stdio(McpStdioConfig {
            command: "node".to_string(),
            args: vec![],
            env: None,
        });
        
        manager.register_server("test".to_string(), config).await;
        
        let connection = manager.get_connection("test").await;
        assert!(connection.is_some());
    }
    
    #[tokio::test]
    async fn test_connect_disconnect() {
        let manager = McpConnectionManager::new();
        
        let config = McpServerConfig::Stdio(McpStdioConfig {
            command: "node".to_string(),
            args: vec![],
            env: None,
        });
        
        manager.register_server("test".to_string(), config).await;
        
        // 连接
        manager.connect_server("test").await.unwrap();
        assert_eq!(manager.get_status("test").await, Some(McpConnectionStatus::Connected));
        
        // 断开
        manager.disconnect_server("test").await.unwrap();
        assert_eq!(manager.get_status("test").await, Some(McpConnectionStatus::Disconnected));
    }
    
    #[tokio::test]
    async fn test_toggle_server() {
        let manager = McpConnectionManager::new();
        
        let config = McpServerConfig::Stdio(McpStdioConfig {
            command: "node".to_string(),
            args: vec![],
            env: None,
        });
        
        manager.register_server("test".to_string(), config).await;
        
        // 切换为连接
        manager.toggle_server("test").await.unwrap();
        assert_eq!(manager.get_status("test").await, Some(McpConnectionStatus::Connected));
        
        // 切换为断开
        manager.toggle_server("test").await.unwrap();
        assert_eq!(manager.get_status("test").await, Some(McpConnectionStatus::Disconnected));
    }
}
