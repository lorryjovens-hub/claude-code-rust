//! Storage - Persistent storage backend

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::MemoryEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageBackend {
    File,
    Sqlite,
    Memory,
}

pub struct Storage {
    backend: StorageBackend,
    path: PathBuf,
    cache: Arc<RwLock<Vec<MemoryEntry>>>,
}

impl Storage {
    pub fn new(path: PathBuf) -> Self {
        Self {
            backend: StorageBackend::File,
            path,
            cache: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub fn with_backend(mut self, backend: StorageBackend) -> Self {
        self.backend = backend;
        self
    }
    
    pub async fn save_memory(&self, entry: &MemoryEntry) -> anyhow::Result<()> {
        match self.backend {
            StorageBackend::File => {
                let file_path = self.path.join(format!("{}.json", entry.id));
                let content = serde_json::to_string_pretty(entry)?;
                tokio::fs::write(&file_path, content).await?;
            }
            StorageBackend::Sqlite => {
                todo!("SQLite backend not implemented")
            }
            StorageBackend::Memory => {
                let mut cache = self.cache.write().await;
                cache.push(entry.clone());
            }
        }
        
        Ok(())
    }
    
    pub async fn load_memory(&self, id: &str) -> anyhow::Result<Option<MemoryEntry>> {
        match self.backend {
            StorageBackend::File => {
                let file_path = self.path.join(format!("{}.json", id));
                if !file_path.exists() {
                    return Ok(None);
                }
                
                let content = tokio::fs::read_to_string(&file_path).await?;
                let entry: MemoryEntry = serde_json::from_str(&content)?;
                Ok(Some(entry))
            }
            StorageBackend::Sqlite => {
                todo!("SQLite backend not implemented")
            }
            StorageBackend::Memory => {
                let cache = self.cache.read().await;
                Ok(cache.iter().find(|e| e.id == id).cloned())
            }
        }
    }
    
    pub async fn delete_memory(&self, id: &str) -> anyhow::Result<()> {
        match self.backend {
            StorageBackend::File => {
                let file_path = self.path.join(format!("{}.json", id));
                if file_path.exists() {
                    tokio::fs::remove_file(&file_path).await?;
                }
            }
            StorageBackend::Sqlite => {
                todo!("SQLite backend not implemented")
            }
            StorageBackend::Memory => {
                let mut cache = self.cache.write().await;
                cache.retain(|e| e.id != id);
            }
        }
        
        Ok(())
    }
    
    pub async fn load_all(&self) -> anyhow::Result<Vec<MemoryEntry>> {
        match self.backend {
            StorageBackend::File => {
                if !self.path.exists() {
                    return Ok(Vec::new());
                }
                
                let mut entries = Vec::new();
                let mut dir = tokio::fs::read_dir(&self.path).await?;
                
                while let Some(entry) = dir.next_entry().await? {
                    let path = entry.path();
                    if path.extension().map(|e| e == "json").unwrap_or(false) {
                        if let Ok(content) = tokio::fs::read_to_string(&path).await {
                            if let Ok(memory) = serde_json::from_str::<MemoryEntry>(&content) {
                                entries.push(memory);
                            }
                        }
                    }
                }
                
                Ok(entries)
            }
            StorageBackend::Sqlite => {
                todo!("SQLite backend not implemented")
            }
            StorageBackend::Memory => {
                let cache = self.cache.read().await;
                Ok(cache.clone())
            }
        }
    }
    
    pub async fn save_all(&self, entries: &[MemoryEntry]) -> anyhow::Result<()> {
        tokio::fs::create_dir_all(&self.path).await?;
        
        for entry in entries {
            self.save_memory(entry).await?;
        }
        
        Ok(())
    }
    
    pub async fn clear(&self) -> anyhow::Result<()> {
        match self.backend {
            StorageBackend::File => {
                if self.path.exists() {
                    tokio::fs::remove_dir_all(&self.path).await?;
                    tokio::fs::create_dir_all(&self.path).await?;
                }
            }
            StorageBackend::Sqlite => {
                todo!("SQLite backend not implemented")
            }
            StorageBackend::Memory => {
                let mut cache = self.cache.write().await;
                cache.clear();
            }
        }
        
        Ok(())
    }
    
    pub async fn size(&self) -> anyhow::Result<u64> {
        match self.backend {
            StorageBackend::File => {
                if !self.path.exists() {
                    return Ok(0);
                }
                
                let mut total_size = 0u64;
                let mut dir = tokio::fs::read_dir(&self.path).await?;
                
                while let Some(entry) = dir.next_entry().await? {
                    let metadata = entry.metadata().await?;
                    total_size += metadata.len();
                }
                
                Ok(total_size)
            }
            StorageBackend::Sqlite => {
                todo!("SQLite backend not implemented")
            }
            StorageBackend::Memory => {
                let cache = self.cache.read().await;
                Ok(std::mem::size_of_val(&*cache) as u64)
            }
        }
    }
    
    pub async fn count(&self) -> anyhow::Result<usize> {
        match self.backend {
            StorageBackend::File => {
                if !self.path.exists() {
                    return Ok(0);
                }
                
                let mut count = 0;
                let mut dir = tokio::fs::read_dir(&self.path).await?;
                
                while let Some(entry) = dir.next_entry().await? {
                    if entry.path().extension().map(|e| e == "json").unwrap_or(false) {
                        count += 1;
                    }
                }
                
                Ok(count)
            }
            StorageBackend::Sqlite => {
                todo!("SQLite backend not implemented")
            }
            StorageBackend::Memory => {
                let cache = self.cache.read().await;
                Ok(cache.len())
            }
        }
    }
}

impl Default for Storage {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self::new(home.join(".claude-code").join("memory"))
    }
}
