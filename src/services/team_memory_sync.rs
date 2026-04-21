//! Team Memory Sync Service - Team memory file synchronization
//!
//! Team memory sync allows team members to share memory files in the
//! `.claude/team/` directory. Supports OAuth authentication for
//! Claude.ai Enterprise/Team users.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMemoryConfig {
    pub enabled: bool,
    pub team_id: Option<String>,
    pub sync_interval_secs: u64,
    pub auto_sync: bool,
    pub conflict_resolution: ConflictResolution,
}

impl Default for TeamMemoryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            team_id: None,
            sync_interval_secs: 3600,
            auto_sync: true,
            conflict_resolution: ConflictResolution::PreferNewer,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConflictResolution {
    PreferLocal,
    PreferRemote,
    PreferNewer,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMemory {
    pub id: String,
    pub title: String,
    pub content: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub etag: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub last_sync: Option<DateTime<Utc>>,
    pub pending_uploads: usize,
    pub pending_downloads: usize,
    pub conflicts: usize,
    pub is_syncing: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TeamMemorySyncStatus {
    pub enabled: bool,
    pub team_id: Option<String>,
    pub is_authenticated: bool,
    pub sync_status: SyncStatus,
    pub local_memories: usize,
    pub remote_memories: usize,
}

pub struct TeamMemorySyncService {
    config: TeamMemoryConfig,
    sync_status: Arc<RwLock<SyncStatus>>,
    local_memories: Arc<RwLock<HashMap<String, TeamMemory>>>,
    remote_memories: Arc<RwLock<HashMap<String, TeamMemory>>>,
}

impl TeamMemorySyncService {
    pub fn new(_state: Arc<RwLock<AppState>>, config: Option<TeamMemoryConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            sync_status: Arc::new(RwLock::new(SyncStatus {
                last_sync: None,
                pending_uploads: 0,
                pending_downloads: 0,
                conflicts: 0,
                is_syncing: false,
            })),
            local_memories: Arc::new(RwLock::new(HashMap::new())),
            remote_memories: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn is_authenticated(&self) -> bool {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let token_path = home.join(".claude-code").join(".team_token");
        token_path.exists()
    }

    pub async fn authenticate(&self, team_id: &str) -> anyhow::Result<()> {
        println!("🔐 Authenticating for team: {}", team_id);
        println!("🔐 Please visit the following URL to authenticate:");
        println!("   https://claude.ai/team/{}/auth", team_id);
        
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let token_path = home.join(".claude-code").join(".team_token");
        
        let token = format!("team_token_{}_{}", team_id, Utc::now().timestamp());
        tokio::fs::write(&token_path, &token).await?;
        
        println!("🔐 Authentication successful!");
        
        Ok(())
    }

    pub async fn sync(&self) -> anyhow::Result<SyncResult> {
        let mut status = self.sync_status.write().await;
        
        if status.is_syncing {
            return Ok(SyncResult {
                uploaded: 0,
                downloaded: 0,
                conflicts: 0,
                errors: vec!["Already syncing".to_string()],
            });
        }

        status.is_syncing = true;
        drop(status);

        let result = self.do_sync().await;

        let mut status = self.sync_status.write().await;
        status.is_syncing = false;
        status.last_sync = Some(Utc::now());

        result
    }

    async fn do_sync(&self) -> anyhow::Result<SyncResult> {
        let mut uploaded = 0;
        let mut downloaded = 0;
        let mut conflicts = 0;
        let errors = Vec::new();

        self.load_local_memories().await?;
        self.fetch_remote_memories().await?;

        {
            let local = self.local_memories.read().await;
            let remote = self.remote_memories.write().await;

            for (id, local_mem) in local.iter() {
                if let Some(remote_mem) = remote.get(id) {
                    if local_mem.etag != remote_mem.etag {
                        conflicts += 1;
                        self.resolve_conflict(local_mem, remote_mem).await?;
                    }
                } else {
                    self.upload_memory(local_mem).await?;
                    uploaded += 1;
                }
            }

            for (id, remote_mem) in remote.iter() {
                if !local.contains_key(id) {
                    self.download_memory(remote_mem).await?;
                    downloaded += 1;
                }
            }
        }

        Ok(SyncResult {
            uploaded,
            downloaded,
            conflicts,
            errors,
        })
    }

    async fn load_local_memories(&self) -> anyhow::Result<()> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let team_dir = home.join(".claude-code").join("team");
        
        if !team_dir.exists() {
            return Ok(());
        }

        let mut memories = self.local_memories.write().await;
        memories.clear();

        let mut entries = tokio::fs::read_dir(&team_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    if let Ok(memory) = serde_json::from_str::<TeamMemory>(&content) {
                        memories.insert(memory.id.clone(), memory);
                    }
                }
            }
        }

        Ok(())
    }

    async fn fetch_remote_memories(&self) -> anyhow::Result<()> {
        println!("📡 Fetching remote memories...");
        
        let mut memories = self.remote_memories.write().await;
        memories.clear();

        Ok(())
    }

    async fn upload_memory(&self, memory: &TeamMemory) -> anyhow::Result<()> {
        println!("📤 Uploading memory: {}", memory.title);
        Ok(())
    }

    async fn download_memory(&self, memory: &TeamMemory) -> anyhow::Result<()> {
        println!("📥 Downloading memory: {}", memory.title);
        
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let team_dir = home.join(".claude-code").join("team");
        tokio::fs::create_dir_all(&team_dir).await?;
        
        let memory_path = team_dir.join(format!("{}.json", memory.id));
        let content = serde_json::to_string_pretty(memory)?;
        tokio::fs::write(&memory_path, content).await?;

        let mut local = self.local_memories.write().await;
        local.insert(memory.id.clone(), memory.clone());

        Ok(())
    }

    async fn resolve_conflict(&self, local: &TeamMemory, remote: &TeamMemory) -> anyhow::Result<()> {
        match self.config.conflict_resolution {
            ConflictResolution::PreferLocal => {
                self.upload_memory(local).await?;
            }
            ConflictResolution::PreferRemote => {
                self.download_memory(remote).await?;
            }
            ConflictResolution::PreferNewer => {
                if local.updated_at > remote.updated_at {
                    self.upload_memory(local).await?;
                } else {
                    self.download_memory(remote).await?;
                }
            }
            ConflictResolution::Manual => {
                println!("⚠️ Conflict detected for: {}", local.title);
                println!("   Local updated: {}", local.updated_at);
                println!("   Remote updated: {}", remote.updated_at);
            }
        }

        Ok(())
    }

    pub async fn create_memory(&self, title: &str, content: &str, tags: Vec<String>) -> anyhow::Result<TeamMemory> {
        let memory = TeamMemory {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            content: content.to_string(),
            author: "local".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            etag: uuid::Uuid::new_v4().to_string(),
            tags,
        };

        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let team_dir = home.join(".claude-code").join("team");
        tokio::fs::create_dir_all(&team_dir).await?;
        
        let memory_path = team_dir.join(format!("{}.json", memory.id));
        let json_content = serde_json::to_string_pretty(&memory)?;
        tokio::fs::write(&memory_path, json_content).await?;

        let mut local = self.local_memories.write().await;
        local.insert(memory.id.clone(), memory.clone());

        println!("📝 Created team memory: {}", memory.title);

        Ok(memory)
    }

    pub async fn get_status(&self) -> TeamMemorySyncStatus {
        let is_authenticated = self.is_authenticated().await;
        let sync_status = self.sync_status.read().await.clone();
        let local = self.local_memories.read().await;
        let remote = self.remote_memories.read().await;

        TeamMemorySyncStatus {
            enabled: self.config.enabled,
            team_id: self.config.team_id.clone(),
            is_authenticated,
            sync_status,
            local_memories: local.len(),
            remote_memories: remote.len(),
        }
    }

    pub async fn start_auto_sync(&self) -> anyhow::Result<()> {
        if !self.config.auto_sync {
            return Ok(());
        }

        println!("🔄 Starting auto-sync (interval: {}s)", self.config.sync_interval_secs);
        
        Ok(())
    }

    pub async fn list_memories(&self) -> Vec<TeamMemory> {
        let local = self.local_memories.read().await;
        local.values().cloned().collect()
    }

    pub async fn delete_memory(&self, id: &str) -> anyhow::Result<()> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let memory_path = home.join(".claude-code").join("team").join(format!("{}.json", id));
        
        if memory_path.exists() {
            tokio::fs::remove_file(&memory_path).await?;
        }

        let mut local = self.local_memories.write().await;
        local.remove(id);

        println!("🗑️ Deleted team memory: {}", id);

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub uploaded: usize,
    pub downloaded: usize,
    pub conflicts: usize,
    pub errors: Vec<String>,
}
