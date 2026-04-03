//! 代理记忆系统
//! 
//! 这个模块实现了代理记忆存储与检索功能

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 记忆条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// 记忆 ID
    pub id: String,
    
    /// 记忆内容
    pub content: String,
    
    /// 创建时间
    pub created_at: i64,
    
    /// 最后访问时间
    pub last_accessed: i64,
    
    /// 访问次数
    pub access_count: u32,
    
    /// 标签
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// 元数据
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl MemoryEntry {
    /// 创建新的记忆条目
    pub fn new(content: String) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            content,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// 访问记忆
    pub fn access(&mut self) {
        self.last_accessed = chrono::Utc::now().timestamp_millis();
        self.access_count += 1;
    }
    
    /// 添加标签
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
    
    /// 设置元数据
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

/// 代理记忆
pub struct AgentMemory {
    /// 记忆存储
    memories: Arc<RwLock<HashMap<String, MemoryEntry>>>,
    
    /// 记忆文件路径
    memory_file: Option<PathBuf>,
    
    /// 最大记忆数
    max_memories: usize,
}

impl AgentMemory {
    /// 创建新的代理记忆
    pub fn new() -> Self {
        Self {
            memories: Arc::new(RwLock::new(HashMap::new())),
            memory_file: None,
            max_memories: 1000,
        }
    }
    
    /// 使用文件存储创建
    pub fn with_file(memory_file: PathBuf) -> Self {
        Self {
            memories: Arc::new(RwLock::new(HashMap::new())),
            memory_file: Some(memory_file),
            max_memories: 1000,
        }
    }
    
    /// 设置最大记忆数
    pub fn with_max_memories(mut self, max: usize) -> Self {
        self.max_memories = max;
        self
    }
    
    /// 添加记忆
    pub async fn add(&self, content: String) -> Result<String> {
        let entry = MemoryEntry::new(content);
        let id = entry.id.clone();
        
        self.memories.write().await.insert(id.clone(), entry);
        
        // 检查是否超过最大记忆数
        if self.memories.read().await.len() > self.max_memories {
            self.evict_oldest().await?;
        }
        
        // 保存到文件
        if let Some(ref file) = self.memory_file {
            self.save_to_file(file).await?;
        }
        
        Ok(id)
    }
    
    /// 获取记忆
    pub async fn get(&self, id: &str) -> Option<MemoryEntry> {
        let mut memories = self.memories.write().await;
        
        if let Some(entry) = memories.get_mut(id) {
            entry.access();
            return Some(entry.clone());
        }
        
        None
    }
    
    /// 搜索记忆
    pub async fn search(&self, query: &str) -> Vec<MemoryEntry> {
        let memories = self.memories.read().await;
        
        memories.values()
            .filter(|entry| entry.content.contains(query))
            .cloned()
            .collect()
    }
    
    /// 按标签搜索
    pub async fn search_by_tag(&self, tag: &str) -> Vec<MemoryEntry> {
        let memories = self.memories.read().await;
        
        memories.values()
            .filter(|entry| entry.tags.contains(&tag.to_string()))
            .cloned()
            .collect()
    }
    
    /// 删除记忆
    pub async fn remove(&self, id: &str) -> Option<MemoryEntry> {
        let entry = self.memories.write().await.remove(id);
        
        // 保存到文件
        if entry.is_some() {
            if let Some(ref file) = self.memory_file {
                let _ = self.save_to_file(file).await;
            }
        }
        
        entry
    }
    
    /// 清空记忆
    pub async fn clear(&self) {
        self.memories.write().await.clear();
        
        // 保存到文件
        if let Some(ref file) = self.memory_file {
            let _ = self.save_to_file(file).await;
        }
    }
    
    /// 获取记忆数量
    pub async fn len(&self) -> usize {
        self.memories.read().await.len()
    }
    
    /// 检查是否为空
    pub async fn is_empty(&self) -> bool {
        self.memories.read().await.is_empty()
    }
    
    /// 淘汰最旧的记忆
    async fn evict_oldest(&self) -> Result<()> {
        let mut memories = self.memories.write().await;
        
        if memories.is_empty() {
            return Ok(());
        }
        
        // 找到最旧的记忆
        let oldest = memories.values()
            .min_by_key(|entry| entry.last_accessed)
            .map(|entry| entry.id.clone());
        
        if let Some(id) = oldest {
            memories.remove(&id);
        }
        
        Ok(())
    }
    
    /// 保存到文件
    async fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let memories = self.memories.read().await;
        let json = serde_json::to_string_pretty(&*memories)?;
        
        tokio::fs::write(path, json).await?;
        
        Ok(())
    }
    
    /// 从文件加载
    pub async fn load_from_file(&self, path: &PathBuf) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }
        
        let json = tokio::fs::read_to_string(path).await?;
        let memories: HashMap<String, MemoryEntry> = serde_json::from_str(&json)?;
        
        *self.memories.write().await = memories;
        
        Ok(())
    }
}

impl Default for AgentMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_memory_entry() {
        let mut entry = MemoryEntry::new("Test memory".to_string());
        
        entry.add_tag("test".to_string());
        entry.set_metadata("key".to_string(), "value".to_string());
        entry.access();
        
        assert!(entry.tags.contains(&"test".to_string()));
        assert_eq!(entry.metadata.get("key"), Some(&"value".to_string()));
        assert_eq!(entry.access_count, 1);
    }
    
    #[tokio::test]
    async fn test_agent_memory() {
        let memory = AgentMemory::new();
        
        let id = memory.add("Test memory".to_string()).await.unwrap();
        
        let entry = memory.get(&id).await;
        assert!(entry.is_some());
        
        let results = memory.search("Test").await;
        assert_eq!(results.len(), 1);
        
        assert_eq!(memory.len().await, 1);
    }
    
    #[tokio::test]
    async fn test_memory_eviction() {
        let memory = AgentMemory::new().with_max_memories(2);
        
        memory.add("Memory 1".to_string()).await.unwrap();
        memory.add("Memory 2".to_string()).await.unwrap();
        memory.add("Memory 3".to_string()).await.unwrap();
        
        // 应该淘汰最旧的记忆
        assert_eq!(memory.len().await, 2);
    }
}
