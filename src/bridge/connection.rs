//! 连接管理模块

use crate::error::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

/// P2P 连接
#[derive(Debug, Clone)]
pub struct P2PConnection {
    /// 连接ID
    pub id: String,
    /// 远程地址
    pub remote_addr: String,
    /// 是否已连接
    pub connected: bool,
    /// 最后活动时间
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

impl P2PConnection {
    /// 创建新的 P2P 连接
    pub fn new(id: String, remote_addr: String) -> Self {
        Self {
            id,
            remote_addr,
            connected: false,
            last_activity: chrono::Utc::now(),
        }
    }
    
    /// 连接
    pub async fn connect(&mut self) -> Result<()> {
        self.connected = true;
        self.last_activity = chrono::Utc::now();
        Ok(())
    }
    
    /// 断开连接
    pub async fn disconnect(&mut self) {
        self.connected = false;
    }
    
    /// 更新活动时间
    pub fn update_activity(&mut self) {
        self.last_activity = chrono::Utc::now();
    }
    
    /// 检查连接是否超时
    pub fn is_timeout(&self, timeout_secs: i64) -> bool {
        let now = chrono::Utc::now();
        let elapsed = (now - self.last_activity).num_seconds();
        elapsed > timeout_secs
    }
}

/// 连接管理器
#[derive(Debug)]
pub struct ConnectionManager {
    /// 活动连接
    connections: Arc<RwLock<HashMap<String, P2PConnection>>>,
    /// 连接超时时间（秒）
    connection_timeout_secs: i64,
    /// 心跳间隔（秒）
    heartbeat_interval_secs: u64,
}

impl ConnectionManager {
    /// 创建新的连接管理器
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            connection_timeout_secs: 300, // 5 分钟
            heartbeat_interval_secs: 30,  // 30 秒
        }
    }
    
    /// 创建新连接
    pub async fn create_connection(&self, id: String, remote_addr: String) -> Result<Arc<String>> {
        let mut connections = self.connections.write().await;
        
        let connection = P2PConnection::new(id.clone(), remote_addr);
        connections.insert(id.clone(), connection);
        
        Ok(Arc::new(id))
    }
    
    /// 获取连接
    pub async fn get_connection(&self, id: &str) -> Option<P2PConnection> {
        let connections = self.connections.read().await;
        connections.get(id).cloned()
    }
    
    /// 连接
    pub async fn connect(&self, id: &str) -> Result<()> {
        let mut connections = self.connections.write().await;
        
        let connection = connections.get_mut(id).ok_or_else(|| {
            crate::error::ClaudeError::Bridge("Connection not found".to_string())
        })?;
        
        connection.connect().await
    }
    
    /// 断开连接
    pub async fn disconnect(&self, id: &str) {
        let mut connections = self.connections.write().await;
        
        if let Some(connection) = connections.get_mut(id) {
            connection.disconnect().await;
        }
    }
    
    /// 移除连接
    pub async fn remove_connection(&self, id: &str) {
        let mut connections = self.connections.write().await;
        connections.remove(id);
    }
    
    /// 更新活动时间
    pub async fn update_activity(&self, id: &str) -> Result<()> {
        let mut connections = self.connections.write().await;
        
        let connection = connections.get_mut(id).ok_or_else(|| {
            crate::error::ClaudeError::Bridge("Connection not found".to_string())
        })?;
        
        connection.update_activity();
        
        Ok(())
    }
    
    /// 获取活动连接数
    pub async fn active_connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.values().filter(|c| c.connected).count()
    }
    
    /// 清理超时连接
    pub async fn cleanup_timeout_connections(&self) {
        let mut connections = self.connections.write().await;
        connections.retain(|_, conn| !conn.is_timeout(self.connection_timeout_secs));
    }
    
    /// 启动心跳任务
    pub async fn start_heartbeat(&self) {
        let connections = Arc::clone(&self.connections);
        let interval_secs = self.heartbeat_interval_secs;
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(interval_secs));
            
            loop {
                interval.tick().await;
                
                let mut conns = connections.write().await;
                for (_, conn) in conns.iter_mut() {
                    if conn.connected {
                        tracing::debug!("Heartbeat for connection {}", conn.id);
                    }
                }
            }
        });
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_p2p_connection() {
        let mut conn = P2PConnection::new(
            "test-conn".to_string(),
            "127.0.0.1:8080".to_string(),
        );
        
        assert!(!conn.connected);
        
        conn.connect().await.unwrap();
        assert!(conn.connected);
        
        conn.disconnect().await;
        assert!(!conn.connected);
    }
    
    #[tokio::test]
    async fn test_connection_manager() {
        let manager = ConnectionManager::new();
        
        let conn_id = manager.create_connection(
            "test-conn".to_string(),
            "127.0.0.1:8080".to_string(),
        ).await.unwrap();
        
        assert_eq!(manager.active_connection_count().await, 0);
        
        manager.connect(&conn_id).await.unwrap();
        assert_eq!(manager.active_connection_count().await, 1);
    }
}
