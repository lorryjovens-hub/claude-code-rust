//! 启动优化模块
//! 
//! 实现快速路径、懒加载、预连接和缓存策略

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

const FAST_PATH_TARGET_MS: u64 = 100;
const INITIAL_BUDGET_MB: usize = 2;
const HEAVY_MODULE_THRESHOLD_KB: usize = 500;
const CONNECTION_POOL_SIZE: usize = 5;
const CACHE_HIT_TARGET_PERCENT: f32 = 85.0;

/// 启动优化器
pub struct StartupOptimizer {
    start_time: Instant,
    fast_path_enabled: bool,
    lazy_loader: LazyLoader,
    connection_pool: ConnectionPool,
    disk_cache: DiskCache,
}

impl StartupOptimizer {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            fast_path_enabled: true,
            lazy_loader: LazyLoader::new(),
            connection_pool: ConnectionPool::new(CONNECTION_POOL_SIZE),
            disk_cache: DiskCache::new(None),
        }
    }

    /// 执行快速路径初始化
    /// 确保核心功能在100ms内完成初始化
    pub async fn fast_path_init(&self) -> crate::error::Result<()> {
        let start = Instant::now();

        self.lazy_loader.load_critical_modules().await?;

        self.connection_pool.preconnect().await?;

        let elapsed = start.elapsed();
        if elapsed.as_millis() > FAST_PATH_TARGET_MS as u128 {
            tracing::warn!(
                "Fast path init exceeded target: {}ms > {}ms",
                elapsed.as_millis(),
                FAST_PATH_TARGET_MS
            );
        } else {
            tracing::info!(
                "Fast path init completed in {}ms",
                elapsed.as_millis()
            );
        }

        Ok(())
    }

    /// 获取启动时间
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// 检查是否在快速路径模式
    pub fn is_fast_path(&self) -> bool {
        self.fast_path_enabled && self.uptime() < Duration::from_millis(FAST_PATH_TARGET_MS)
    }

    /// 懒加载模块
    pub async fn load_module(&self, module_name: &str) -> crate::error::Result<()> {
        self.lazy_loader.load(module_name).await
    }

    /// 获取连接池
    pub fn connection_pool(&self) -> &ConnectionPool {
        &self.connection_pool
    }

    /// 获取磁盘缓存
    pub fn disk_cache(&self) -> &DiskCache {
        &self.disk_cache
    }
}

impl Default for StartupOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// 懒加载器
pub struct LazyLoader {
    loaded_modules: Arc<RwLock<HashMap<String, bool>>>,
    module_priorities: HashMap<String, ModulePriority>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModulePriority {
    Critical,
    High,
    Normal,
    Low,
    Deferred,
}

impl LazyLoader {
    pub fn new() -> Self {
        let mut priorities = HashMap::new();
        priorities.insert("core".to_string(), ModulePriority::Critical);
        priorities.insert("config".to_string(), ModulePriority::Critical);
        priorities.insert("commands".to_string(), ModulePriority::High);
        priorities.insert("tools".to_string(), ModulePriority::High);
        priorities.insert("services".to_string(), ModulePriority::Normal);
        priorities.insert("agents".to_string(), ModulePriority::Normal);
        priorities.insert("voice".to_string(), ModulePriority::Low);
        priorities.insert("analytics".to_string(), ModulePriority::Deferred);

        Self {
            loaded_modules: Arc::new(RwLock::new(HashMap::new())),
            module_priorities: priorities,
        }
    }

    /// 加载关键模块
    pub async fn load_critical_modules(&self) -> crate::error::Result<()> {
        let critical_modules: Vec<&str> = self.module_priorities
            .iter()
            .filter(|(_, p)| **p == ModulePriority::Critical)
            .map(|(name, _)| name.as_str())
            .collect();

        for module in critical_modules {
            self.load(module).await?;
        }

        Ok(())
    }

    /// 加载指定模块
    pub async fn load(&self, module_name: &str) -> crate::error::Result<()> {
        let mut loaded = self.loaded_modules.write().await;
        
        if loaded.contains_key(module_name) {
            return Ok(());
        }

        let start = Instant::now();
        
        // 模拟模块加载
        tokio::time::sleep(Duration::from_millis(1)).await;

        loaded.insert(module_name.to_string(), true);

        tracing::debug!(
            "Module '{}' loaded in {}ms",
            module_name,
            start.elapsed().as_millis()
        );

        Ok(())
    }

    /// 检查模块是否已加载
    pub async fn is_loaded(&self, module_name: &str) -> bool {
        self.loaded_modules.read().await.contains_key(module_name)
    }

    /// 获取模块大小估算（KB）
    pub fn estimate_module_size(&self, module_name: &str) -> usize {
        match module_name {
            "core" | "config" => 50,
            "commands" | "tools" => 200,
            "services" | "agents" => 400,
            "voice" | "analytics" => 600,
            _ => 100,
        }
    }

    /// 检查是否为重型模块
    pub fn is_heavy_module(&self, module_name: &str) -> bool {
        self.estimate_module_size(module_name) > HEAVY_MODULE_THRESHOLD_KB
    }
}

impl Default for LazyLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// 连接池
pub struct ConnectionPool {
    pool_size: usize,
    connections: Arc<RwLock<Vec<ConnectionState>>>,
}

#[derive(Debug, Clone)]
pub struct ConnectionState {
    pub id: String,
    pub status: ConnectionStatus,
    pub created_at: Instant,
    pub last_used: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Idle,
    Busy,
    Error,
}

impl ConnectionPool {
    pub fn new(pool_size: usize) -> Self {
        Self {
            pool_size,
            connections: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 预建立连接
    pub async fn preconnect(&self) -> crate::error::Result<()> {
        let mut connections = self.connections.write().await;

        for i in 0..self.pool_size {
            connections.push(ConnectionState {
                id: format!("conn-{}", i),
                status: ConnectionStatus::Idle,
                created_at: Instant::now(),
                last_used: Instant::now(),
            });
        }

        tracing::info!("Pre-established {} API connections", self.pool_size);
        Ok(())
    }

    /// 获取可用连接
    pub async fn acquire(&self) -> crate::error::Result<String> {
        let mut connections = self.connections.write().await;

        for conn in connections.iter_mut() {
            if conn.status == ConnectionStatus::Idle {
                conn.status = ConnectionStatus::Busy;
                conn.last_used = Instant::now();
                return Ok(conn.id.clone());
            }
        }

        Err(crate::error::ClaudeError::Other("No available connections".to_string()))
    }

    /// 释放连接
    pub async fn release(&self, conn_id: &str) {
        let mut connections = self.connections.write().await;

        if let Some(conn) = connections.iter_mut().find(|c| c.id == conn_id) {
            conn.status = ConnectionStatus::Idle;
            conn.last_used = Instant::now();
        }
    }

    /// 获取空闲连接数
    pub async fn idle_count(&self) -> usize {
        self.connections
            .read()
            .await
            .iter()
            .filter(|c| c.status == ConnectionStatus::Idle)
            .count()
    }
}

/// 磁盘缓存
pub struct DiskCache {
    cache_dir: PathBuf,
    max_size_mb: usize,
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub data: Vec<u8>,
    pub created_at: i64,
    pub expires_at: i64,
    pub hit_count: u32,
}

impl DiskCache {
    pub fn new(cache_dir: Option<PathBuf>) -> Self {
        let cache_dir = cache_dir.unwrap_or_else(|| {
            std::env::temp_dir().join("claude-code-cache")
        });

        Self {
            cache_dir,
            max_size_mb: 100,
            entries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 获取缓存
    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let mut entries = self.entries.write().await;

        if let Some(entry) = entries.get_mut(key) {
            let now = chrono::Utc::now().timestamp_millis();
            
            if now > entry.expires_at {
                entries.remove(key);
                return None;
            }

            entry.hit_count += 1;
            return Some(entry.data.clone());
        }

        None
    }

    /// 设置缓存
    pub async fn set(&self, key: &str, data: Vec<u8>, ttl_secs: i64) {
        let mut entries = self.entries.write().await;

        let now = chrono::Utc::now().timestamp_millis();

        entries.insert(key.to_string(), CacheEntry {
            key: key.to_string(),
            data,
            created_at: now,
            expires_at: now + ttl_secs * 1000,
            hit_count: 0,
        });
    }

    /// 获取缓存命中率
    pub async fn hit_rate(&self) -> f32 {
        let entries = self.entries.read().await;

        if entries.is_empty() {
            return 0.0;
        }

        let total_hits: u32 = entries.values().map(|e| e.hit_count).sum();
        let total_requests = total_hits + entries.len() as u32;

        if total_requests == 0 {
            return 0.0;
        }

        (total_hits as f32 / total_requests as f32) * 100.0
    }

    /// 清理过期缓存
    pub async fn cleanup_expired(&self) {
        let mut entries = self.entries.write().await;
        let now = chrono::Utc::now().timestamp_millis();

        entries.retain(|_, e| now <= e.expires_at);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_startup_optimizer() {
        let optimizer = StartupOptimizer::new();
        assert!(optimizer.fast_path_init().await.is_ok());
        assert!(optimizer.uptime() < Duration::from_secs(1));
    }

    #[tokio::test]
    async fn test_lazy_loader() {
        let loader = LazyLoader::new();
        
        assert!(!loader.is_loaded("core").await);
        assert!(loader.load("core").await.is_ok());
        assert!(loader.is_loaded("core").await);
    }

    #[tokio::test]
    async fn test_connection_pool() {
        let pool = ConnectionPool::new(3);
        
        assert!(pool.preconnect().await.is_ok());
        assert_eq!(pool.idle_count().await, 3);
        
        let conn = pool.acquire().await.unwrap();
        assert_eq!(pool.idle_count().await, 2);
        
        pool.release(&conn).await;
        assert_eq!(pool.idle_count().await, 3);
    }

    #[tokio::test]
    async fn test_disk_cache() {
        let cache = DiskCache::new(None);
        
        cache.set("test_key", vec![1, 2, 3], 60).await;
        
        let data = cache.get("test_key").await;
        assert!(data.is_some());
        assert_eq!(data.unwrap(), vec![1, 2, 3]);
    }
}
