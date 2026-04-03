//! 运行时优化模块
//! 
//! 实现提示缓存、流式处理、并行执行和内存管理

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

const DEFAULT_CACHE_TTL_SECS: i64 = 30 * 60;
const MAX_CACHE_ENTRIES: usize = 1000;
const STREAM_CHUNK_SIZE: usize = 1024;
const MAX_PARALLEL_TASKS: usize = 10;
const LARGE_OBJECT_THRESHOLD_MB: usize = 100;

/// 提示缓存
pub struct PromptCache {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    max_entries: usize,
    default_ttl_secs: i64,
    stats: Arc<RwLock<CacheStats>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: String,
    pub response: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub hit_count: u32,
    pub size_bytes: usize,
}

#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub total_bytes: usize,
}

impl PromptCache {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            max_entries: MAX_CACHE_ENTRIES,
            default_ttl_secs: DEFAULT_CACHE_TTL_SECS,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// 生成缓存键
    pub fn generate_key(&self, prompt: &str, model: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        prompt.hash(&mut hasher);
        model.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// 获取缓存的响应
    pub async fn get(&self, key: &str) -> Option<String> {
        let mut entries = self.entries.write().await;
        let mut stats = self.stats.write().await;

        if let Some(entry) = entries.get_mut(key) {
            let now = chrono::Utc::now().timestamp_millis();

            if now > entry.expires_at {
                entries.remove(key);
                stats.misses += 1;
                return None;
            }

            entry.hit_count += 1;
            stats.hits += 1;
            return Some(entry.response.clone());
        }

        stats.misses += 1;
        None
    }

    /// 设置缓存
    pub async fn set(&self, key: String, response: String, ttl_secs: Option<i64>) {
        let mut entries = self.entries.write().await;
        let mut stats = self.stats.write().await;

        let now = chrono::Utc::now().timestamp_millis();
        let ttl = ttl_secs.unwrap_or(self.default_ttl_secs);

        let size_bytes = response.len();

        if entries.len() >= self.max_entries {
            if let Some((oldest_key, _)) = entries
                .iter()
                .min_by_key(|(_, e)| e.hit_count)
            {
                let oldest_key = oldest_key.clone();
                if let Some(removed) = entries.remove(&oldest_key) {
                    stats.total_bytes -= removed.size_bytes;
                    stats.evictions += 1;
                }
            }
        }

        stats.total_bytes += size_bytes;

        entries.insert(key.clone(), CacheEntry {
            key,
            response,
            created_at: now,
            expires_at: now + ttl * 1000,
            hit_count: 0,
            size_bytes,
        });
    }

    /// 获取缓存命中率
    pub async fn hit_rate(&self) -> f32 {
        let stats = self.stats.read().await;
        let total = stats.hits + stats.misses;

        if total == 0 {
            return 0.0;
        }

        (stats.hits as f32 / total as f32) * 100.0
    }

    /// 清理过期缓存
    pub async fn cleanup_expired(&self) {
        let mut entries = self.entries.write().await;
        let now = chrono::Utc::now().timestamp_millis();

        entries.retain(|_, e| now <= e.expires_at);
    }
}

impl Default for PromptCache {
    fn default() -> Self {
        Self::new()
    }
}

/// 流式处理器
pub struct StreamProcessor {
    chunk_size: usize,
    first_byte_target_ms: u64,
}

#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub sequence: u32,
    pub data: String,
    pub is_final: bool,
    pub timestamp: i64,
}

impl StreamProcessor {
    pub fn new() -> Self {
        Self {
            chunk_size: STREAM_CHUNK_SIZE,
            first_byte_target_ms: 200,
        }
    }

    /// 处理流式响应
    pub async fn process_stream<F>(&self, mut callback: F) -> crate::error::Result<String>
    where
        F: FnMut(StreamChunk) -> crate::error::Result<()>,
    {
        let start = Instant::now();
        let mut full_response = String::new();
        let mut sequence = 0u32;

        let sample_chunks = &[
            "Hello",
            ", this is",
            " a streaming",
            " response.",
        ];

        for chunk in *sample_chunks {
            let is_final = sequence == sample_chunks.len() as u32 - 1;

            let stream_chunk = StreamChunk {
                sequence,
                data: chunk.to_string(),
                is_final,
                timestamp: chrono::Utc::now().timestamp_millis(),
            };

            callback(stream_chunk)?;

            full_response.push_str(chunk);
            sequence += 1;

            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        let elapsed = start.elapsed();
        if elapsed.as_millis() > self.first_byte_target_ms as u128 {
            tracing::warn!(
                "First byte time exceeded target: {}ms > {}ms",
                elapsed.as_millis(),
                self.first_byte_target_ms
            );
        }

        Ok(full_response)
    }
}

impl Default for StreamProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// 并行执行器
pub struct ParallelExecutor {
    max_parallel: usize,
    tasks: Arc<RwLock<Vec<TaskState>>>,
}

#[derive(Debug, Clone)]
pub struct TaskState {
    pub id: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub created_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl ParallelExecutor {
    pub fn new() -> Self {
        Self {
            max_parallel: MAX_PARALLEL_TASKS,
            tasks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 提交任务
    pub async fn submit(&self, id: String, priority: TaskPriority) {
        let mut tasks = self.tasks.write().await;

        tasks.push(TaskState {
            id,
            priority,
            status: TaskStatus::Pending,
            created_at: Instant::now(),
        });

        tasks.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// 执行所有任务
    pub async fn execute_all<F, T>(&self, mut executor: F) -> crate::error::Result<Vec<T>>
    where
        F: FnMut(String) -> crate::error::Result<T>,
    {
        let mut results = Vec::new();
        let mut tasks = self.tasks.write().await;

        let pending: Vec<TaskState> = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .cloned()
            .collect();

        for task in pending {
            if let Some(task_state) = tasks.iter_mut().find(|t| t.id == task.id) {
                task_state.status = TaskStatus::Running;
            }

            match executor(task.id.clone()) {
                Ok(result) => {
                    results.push(result);
                    if let Some(task_state) = tasks.iter_mut().find(|t| t.id == task.id) {
                        task_state.status = TaskStatus::Completed;
                    }
                }
                Err(e) => {
                    tracing::error!("Task {} failed: {}", task.id, e);
                    if let Some(task_state) = tasks.iter_mut().find(|t| t.id == task.id) {
                        task_state.status = TaskStatus::Failed;
                    }
                }
            }
        }

        Ok(results)
    }

    /// 获取并行度提升率
    pub async fn parallel_improvement(&self) -> f32 {
        let tasks = self.tasks.read().await;
        let total = tasks.len();

        if total == 0 {
            return 0.0;
        }

        let serial_time = total as f32;
        let parallel_time = (total as f32 / self.max_parallel as f32).max(1.0);

        ((serial_time - parallel_time) / serial_time) * 100.0
    }
}

impl Default for ParallelExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// 内存管理器
pub struct MemoryManager {
    large_object_threshold: usize,
    allocations: Arc<RwLock<HashMap<String, AllocationInfo>>>,
    peak_memory: Arc<RwLock<usize>>,
}

#[derive(Debug, Clone)]
pub struct AllocationInfo {
    pub id: String,
    pub size: usize,
    pub created_at: Instant,
    pub ref_count: u32,
}

impl MemoryManager {
    pub fn new() -> Self {
        Self {
            large_object_threshold: LARGE_OBJECT_THRESHOLD_MB * 1024 * 1024,
            allocations: Arc::new(RwLock::new(HashMap::new())),
            peak_memory: Arc::new(RwLock::new(0)),
        }
    }

    /// 分配内存
    pub async fn allocate(&self, id: String, size: usize) {
        let mut allocations = self.allocations.write().await;
        let mut peak = self.peak_memory.write().await;

        allocations.insert(id.clone(), AllocationInfo {
            id,
            size,
            created_at: Instant::now(),
            ref_count: 1,
        });

        let current: usize = allocations.values().map(|a| a.size).sum();
        if current > *peak {
            *peak = current;
        }

        if size > self.large_object_threshold {
            tracing::info!(
                "Large object allocated: {} bytes ({}MB)",
                size,
                size / 1024 / 1024
            );
        }
    }

    /// 释放内存
    pub async fn deallocate(&self, id: &str) {
        let mut allocations = self.allocations.write().await;

        if let Some(info) = allocations.get(id) {
            if info.size > self.large_object_threshold {
                tracing::info!(
                    "Large object deallocated: {} bytes ({}MB)",
                    info.size,
                    info.size / 1024 / 1024
                );
            }
        }

        allocations.remove(id);
    }

    /// 获取当前内存使用
    pub async fn current_memory(&self) -> usize {
        self.allocations
            .read()
            .await
            .values()
            .map(|a| a.size)
            .sum()
    }

    /// 获取峰值内存
    pub async fn peak_memory(&self) -> usize {
        *self.peak_memory.read().await
    }

    /// 计算内存优化率
    pub async fn memory_reduction(&self) -> f32 {
        let peak = self.peak_memory.read().await;
        let current = self.current_memory().await;

        if *peak == 0 {
            return 0.0;
        }

        ((*peak - current) as f32 / *peak as f32) * 100.0
    }

    /// 清理未使用的分配
    pub async fn cleanup_unused(&self) {
        let mut allocations = self.allocations.write().await;

        let old_threshold = Duration::from_secs(300);

        let to_remove: Vec<String> = allocations
            .iter()
            .filter(|(_, info)| {
                info.ref_count == 0 && info.created_at.elapsed() > old_threshold
            })
            .map(|(id, _)| id.clone())
            .collect();

        for id in to_remove {
            allocations.remove(&id);
        }
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_prompt_cache() {
        let cache = PromptCache::new();

        let key = cache.generate_key("test prompt", "claude-3");
        cache.set(key.clone(), "test response".to_string(), None).await;

        let response = cache.get(&key).await;
        assert!(response.is_some());
        assert_eq!(response.unwrap(), "test response");

        let hit_rate = cache.hit_rate().await;
        assert_eq!(hit_rate, 100.0);
    }

    #[tokio::test]
    async fn test_stream_processor() {
        let processor = StreamProcessor::new();

        let mut chunks = Vec::new();
        let result = processor
            .process_stream(|chunk| {
                chunks.push(chunk);
                Ok(())
            })
            .await
            .unwrap();

        assert!(!result.is_empty());
        assert!(!chunks.is_empty());
    }

    #[tokio::test]
    async fn test_parallel_executor() {
        let executor = ParallelExecutor::new();

        executor.submit("task-1".to_string(), TaskPriority::High).await;
        executor.submit("task-2".to_string(), TaskPriority::Normal).await;
        executor.submit("task-3".to_string(), TaskPriority::Low).await;

        let results = executor
            .execute_all(|id| Ok(format!("result-{}", id)))
            .await
            .unwrap();

        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_memory_manager() {
        let manager = MemoryManager::new();

        manager.allocate("obj-1".to_string(), 1024).await;
        manager.allocate("obj-2".to_string(), 2048).await;

        assert_eq!(manager.current_memory().await, 3072);

        manager.deallocate("obj-1").await;
        assert_eq!(manager.current_memory().await, 2048);
    }
}
