use crate::mcp::registry::McpClientConnection;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::debug;

pub struct McpConnectionPool {
    connections: Arc<Mutex<Vec<PooledConnection>>>,
    max_idle: usize,
    idle_timeout: Duration,
}

struct PooledConnection {
    connection: Arc<McpClientConnection>,
    server_id: String,
    last_used: std::time::Instant,
    in_use: bool,
}

impl McpConnectionPool {
    pub fn new(max_idle: usize, idle_timeout_secs: u64) -> Self {
        Self {
            connections: Arc::new(Mutex::new(Vec::new())),
            max_idle,
            idle_timeout: Duration::from_secs(idle_timeout_secs),
        }
    }

    pub async fn acquire(
        &self,
        server_id: &str,
    ) -> Option<Arc<McpClientConnection>> {
        let mut pool = self.connections.lock().await;

        if let Some(entry) = pool.iter_mut().find(|e| e.server_id == server_id && !e.in_use) {
            entry.in_use = true;
            entry.last_used = std::time::Instant::now();
            debug!(target: "mcp_pool", "Reusing pooled connection for: {}", server_id);
            return Some(entry.connection.clone());
        }

        None
    }

    pub async fn release(&self, connection: Arc<McpClientConnection>, server_id: &str) {
        let mut pool = self.connections.lock().await;

        if let Some(entry) = pool.iter_mut().find(|e| e.server_id == server_id && e.in_use) {
            entry.in_use = false;
            entry.last_used = std::time::Instant::now();
            return;
        }

        if pool.len() < self.max_idle {
            pool.push(PooledConnection {
                connection,
                server_id: server_id.to_string(),
                last_used: std::time::Instant::now(),
                in_use: false,
            });
        }
    }

    pub async fn add(&self, connection: Arc<McpClientConnection>, server_id: &str) {
        let mut pool = self.connections.lock().await;

        if pool.iter_mut().any(|e| e.server_id == server_id) {
            return;
        }

        if pool.len() >= self.max_idle {
            if let Some(pos) = pool.iter().position(|e| !e.in_use) {
                pool.remove(pos);
            }
        }

        pool.push(PooledConnection {
            connection,
            server_id: server_id.to_string(),
            last_used: std::time::Instant::now(),
            in_use: false,
        });
    }

    pub async fn cleanup_expired(&self) {
        let mut pool = self.connections.lock().await;
        let timeout = self.idle_timeout;

        pool.retain(|entry| {
            if entry.in_use {
                return true;
            }
            entry.last_used.elapsed() < timeout
        });

        debug!(target: "mcp_pool", "Pool cleanup: {} connections remaining", pool.len());
    }

    pub async fn size(&self) -> usize {
        self.connections.lock().await.len()
    }

    pub async fn active_count(&self) -> usize {
        self.connections.lock().await.iter().filter(|e| e.in_use).count()
    }
}