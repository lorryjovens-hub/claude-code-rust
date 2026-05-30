use crate::execution::types::{ToolCallRequest, ToolCallResult};
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ResultCache {
    cache: Arc<Mutex<LruCache<u64, ToolCallResult>>>,
    stats: Arc<Mutex<CacheStats>>,
}

#[derive(Debug, Default)]
struct CacheStats {
    hits: u64,
    misses: u64,
    evictions: u64,
}

impl ResultCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(
                LruCache::new(NonZeroUsize::new(capacity.max(1)).unwrap()),
            )),
            stats: Arc::new(Mutex::new(CacheStats::default())),
        }
    }

    fn compute_key(tool: &ToolCallRequest) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        tool.tool_name.hash(&mut hasher);
        tool.arguments.to_string().hash(&mut hasher);
        hasher.finish()
    }

    pub async fn get(&self, tool: &ToolCallRequest) -> Option<ToolCallResult> {
        let key = Self::compute_key(tool);
        let mut cache = self.cache.lock().await;

        match cache.get(&key) {
            Some(result) => {
                let mut stats = self.stats.lock().await;
                stats.hits += 1;
                Some(result.clone())
            }
            None => {
                let mut stats = self.stats.lock().await;
                stats.misses += 1;
                None
            }
        }
    }

    pub async fn put(&self, tool: ToolCallRequest, result: ToolCallResult) {
        let key = Self::compute_key(&tool);
        let mut cache = self.cache.lock().await;

        let old = cache.put(key, result);
        if old.is_some() {
            let mut stats = self.stats.lock().await;
            stats.evictions += 1;
        }
    }

    pub async fn size(&self) -> usize {
        self.cache.lock().await.len()
    }

    pub async fn hit_rate(&self) -> f64 {
        let stats = self.stats.lock().await;
        let total = stats.hits + stats.misses;
        if total == 0 {
            0.0
        } else {
            stats.hits as f64 / total as f64
        }
    }

    pub async fn clear(&self) {
        self.cache.lock().await.clear();
        *self.stats.lock().await = CacheStats::default();
    }
}