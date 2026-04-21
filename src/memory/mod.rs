//! Memory Module - Complete memory management system
//!
//! Features:
//! - Session management
//! - History tracking
//! - Context maintenance
//! - Persistent storage

pub mod session;
pub mod history;
pub mod context;
pub mod storage;
pub mod consolidation;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

pub use session::{SessionManager, Session, SessionInfo};
pub use history::{HistoryManager, HistoryEntry, HistoryFilter};
pub use context::{ContextManager, ContextWindow, ContextEntry};
pub use storage::{Storage, StorageBackend};
pub use consolidation::{ConsolidationEngine, ConsolidationConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub memory_type: MemoryType,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub importance: f32,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub embedding: Option<Vec<f32>>,
}

impl MemoryEntry {
    pub fn new(memory_type: MemoryType, content: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            memory_type,
            content: content.to_string(),
            timestamp: Utc::now(),
            importance: 0.5,
            tags: Vec::new(),
            metadata: HashMap::new(),
            embedding: None,
        }
    }
    
    pub fn with_importance(mut self, importance: f32) -> Self {
        self.importance = importance.clamp(0.0, 1.0);
        self
    }
    
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
    
    pub fn with_metadata(mut self, key: &str, value: serde_json::Value) -> Self {
        self.metadata.insert(key.to_string(), value);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MemoryType {
    Session,
    Conversation,
    Knowledge,
    Preference,
    Task,
    Error,
    Insight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatus {
    pub total_memories: usize,
    pub session_count: usize,
    pub conversation_count: usize,
    pub knowledge_count: usize,
    pub last_consolidation: Option<DateTime<Utc>>,
    pub storage_size_bytes: u64,
}

pub struct MemoryManager {
    sessions: Arc<SessionManager>,
    history: Arc<HistoryManager>,
    context: Arc<ContextManager>,
    storage: Arc<Storage>,
    consolidation: Arc<ConsolidationEngine>,
    memories: Arc<RwLock<Vec<MemoryEntry>>>,
}

impl MemoryManager {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let memory_path = home.join(".claude-code").join("memory");
        
        std::fs::create_dir_all(&memory_path).ok();
        
        Self {
            sessions: Arc::new(SessionManager::new()),
            history: Arc::new(HistoryManager::new()),
            context: Arc::new(ContextManager::new()),
            storage: Arc::new(Storage::new(memory_path)),
            consolidation: Arc::new(ConsolidationEngine::new(Default::default())),
            memories: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub async fn status(&self) -> anyhow::Result<MemoryStatus> {
        let memories = self.memories.read().await;
        
        let storage_size = self.storage.size().await.unwrap_or(0);
        
        Ok(MemoryStatus {
            total_memories: memories.len(),
            session_count: memories.iter().filter(|m| m.memory_type == MemoryType::Session).count(),
            conversation_count: memories.iter().filter(|m| m.memory_type == MemoryType::Conversation).count(),
            knowledge_count: memories.iter().filter(|m| m.memory_type == MemoryType::Knowledge).count(),
            last_consolidation: None,
            storage_size_bytes: storage_size,
        })
    }
    
    pub async fn add_memory(&self, entry: MemoryEntry) -> anyhow::Result<()> {
        let mut memories = self.memories.write().await;
        memories.push(entry.clone());
        
        self.storage.save_memory(&entry).await?;
        
        Ok(())
    }
    
    pub async fn get_memory(&self, id: &str) -> Option<MemoryEntry> {
        let memories = self.memories.read().await;
        memories.iter().find(|m| m.id == id).cloned()
    }
    
    pub async fn search_memories(&self, query: &str) -> Vec<MemoryEntry> {
        let query_lower = query.to_lowercase();
        let memories = self.memories.read().await;
        
        memories.iter()
            .filter(|m| {
                m.content.to_lowercase().contains(&query_lower) ||
                m.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }
    
    pub async fn get_memories_by_type(&self, memory_type: MemoryType) -> Vec<MemoryEntry> {
        let memories = self.memories.read().await;
        memories.iter()
            .filter(|m| m.memory_type == memory_type)
            .cloned()
            .collect()
    }
    
    pub async fn get_important_memories(&self, threshold: f32) -> Vec<MemoryEntry> {
        let memories = self.memories.read().await;
        memories.iter()
            .filter(|m| m.importance >= threshold)
            .cloned()
            .collect()
    }
    
    pub async fn clear(&self) -> anyhow::Result<()> {
        let mut memories = self.memories.write().await;
        memories.clear();
        
        self.storage.clear().await?;
        
        Ok(())
    }
    
    pub async fn export(&self, output: &PathBuf) -> anyhow::Result<()> {
        let memories = self.memories.read().await;
        let content = serde_json::to_string_pretty(&*memories)?;
        tokio::fs::write(output, content).await?;
        Ok(())
    }
    
    pub async fn import(&self, input: &PathBuf) -> anyhow::Result<()> {
        let content = tokio::fs::read_to_string(input).await?;
        let imported: Vec<MemoryEntry> = serde_json::from_str(&content)?;
        
        let mut memories = self.memories.write().await;
        memories.extend(imported);
        
        Ok(())
    }
    
    pub async fn consolidate(&self) -> anyhow::Result<()> {
        let memories = self.memories.read().await;
        let consolidated = self.consolidation.consolidate(&memories).await?;
        
        drop(memories);
        
        let mut memories = self.memories.write().await;
        *memories = consolidated;
        
        Ok(())
    }
    
    pub async fn load(&self) -> anyhow::Result<()> {
        let memories = self.storage.load_all().await?;
        let mut mem = self.memories.write().await;
        *mem = memories;
        Ok(())
    }
    
    pub async fn save(&self) -> anyhow::Result<()> {
        let memories = self.memories.read().await;
        self.storage.save_all(&memories).await
    }
    
    pub fn sessions(&self) -> Arc<SessionManager> {
        self.sessions.clone()
    }
    
    pub fn history(&self) -> Arc<HistoryManager> {
        self.history.clone()
    }
    
    pub fn context(&self) -> Arc<ContextManager> {
        self.context.clone()
    }
    
    pub fn storage(&self) -> Arc<Storage> {
        self.storage.clone()
    }
    
    pub fn consolidation(&self) -> Arc<ConsolidationEngine> {
        self.consolidation.clone()
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}
