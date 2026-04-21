//! Session Module - Session management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Session manager
pub struct SessionManager {
    sessions_dir: PathBuf,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let sessions_dir = home.join(".claude-code").join("sessions");
        
        Self { sessions_dir }
    }
    
    /// List all sessions
    pub fn list(&self) -> anyhow::Result<Vec<SessionInfo>> {
        if !self.sessions_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut sessions = Vec::new();
        for entry in std::fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<SessionInfo>(&content) {
                        sessions.push(session);
                    }
                }
            }
        }
        
        // Sort by created_at descending
        sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(sessions)
    }
    
    /// Create a new session
    pub fn create(&self, name: Option<&str>) -> anyhow::Result<Session> {
        std::fs::create_dir_all(&self.sessions_dir)?;
        
        let id = uuid::Uuid::new_v4().to_string();
        let session_name = name.unwrap_or(&id).to_string();
        
        let session = Session {
            id: id.clone(),
            name: session_name,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            messages: Vec::new(),
        };
        
        self.save(&session)?;
        
        Ok(session)
    }
    
    /// Load a session by ID
    pub fn load(&self, id: &str) -> anyhow::Result<Option<Session>> {
        let path = self.sessions_dir.join(format!("{}.json", id));
        
        if !path.exists() {
            return Ok(None);
        }
        
        let content = std::fs::read_to_string(&path)?;
        let session = serde_json::from_str(&content)?;
        
        Ok(Some(session))
    }
    
    /// Save a session
    pub fn save(&self, session: &Session) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.sessions_dir)?;
        
        let path = self.sessions_dir.join(format!("{}.json", session.id));
        let content = serde_json::to_string_pretty(session)?;
        std::fs::write(&path, content)?;
        
        Ok(())
    }
    
    /// Delete a session
    pub fn delete(&self, id: &str) -> anyhow::Result<()> {
        let path = self.sessions_dir.join(format!("{}.json", id));
        
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        
        Ok(())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
}