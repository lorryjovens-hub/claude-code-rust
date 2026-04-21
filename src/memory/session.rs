//! Session Management - Session lifecycle management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub project_path: Option<PathBuf>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<SessionMessage>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub status: SessionStatus,
}

impl Session {
    pub fn new(name: Option<&str>) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id: id.clone(),
            name: name.unwrap_or(&id).to_string(),
            project_path: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            messages: Vec::new(),
            metadata: HashMap::new(),
            status: SessionStatus::Active,
        }
    }
    
    pub fn with_project(mut self, path: PathBuf) -> Self {
        self.project_path = Some(path);
        self
    }
    
    pub fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push(SessionMessage {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        });
        self.updated_at = Utc::now();
    }
    
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionStatus {
    Active,
    Paused,
    Archived,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub project_path: Option<PathBuf>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: usize,
    pub status: SessionStatus,
}

pub struct SessionManager {
    sessions_dir: PathBuf,
    active_session: Arc<RwLock<Option<Session>>>,
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let sessions_dir = home.join(".claude-code").join("sessions");
        
        Self {
            sessions_dir,
            active_session: Arc::new(RwLock::new(None)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn create(&self, name: Option<&str>) -> anyhow::Result<Session> {
        let session = Session::new(name);
        self.save(&session).await?;
        
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session.clone());
        
        Ok(session)
    }
    
    pub async fn load(&self, id: &str) -> anyhow::Result<Option<Session>> {
        let path = self.sessions_dir.join(format!("{}.json", id));
        
        if !path.exists() {
            return Ok(None);
        }
        
        let content = tokio::fs::read_to_string(&path).await?;
        let session: Session = serde_json::from_str(&content)?;
        
        let mut sessions = self.sessions.write().await;
        sessions.insert(id.to_string(), session.clone());
        
        Ok(Some(session))
    }
    
    pub async fn save(&self, session: &Session) -> anyhow::Result<()> {
        tokio::fs::create_dir_all(&self.sessions_dir).await?;
        
        let path = self.sessions_dir.join(format!("{}.json", session.id));
        let content = serde_json::to_string_pretty(session)?;
        tokio::fs::write(&path, content).await?;
        
        Ok(())
    }
    
    pub async fn delete(&self, id: &str) -> anyhow::Result<()> {
        let path = self.sessions_dir.join(format!("{}.json", id));
        
        if path.exists() {
            tokio::fs::remove_file(&path).await?;
        }
        
        let mut sessions = self.sessions.write().await;
        sessions.remove(id);
        
        Ok(())
    }
    
    pub async fn list(&self) -> anyhow::Result<Vec<SessionInfo>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.values().map(|s| SessionInfo {
            id: s.id.clone(),
            name: s.name.clone(),
            project_path: s.project_path.clone(),
            created_at: s.created_at,
            updated_at: s.updated_at,
            message_count: s.messages.len(),
            status: s.status.clone(),
        }).collect())
    }
    
    pub async fn get(&self, id: &str) -> Option<Session> {
        let sessions = self.sessions.read().await;
        sessions.get(id).cloned()
    }
    
    pub async fn set_active(&self, session: Session) {
        let mut active = self.active_session.write().await;
        *active = Some(session);
    }
    
    pub async fn get_active(&self) -> Option<Session> {
        let active = self.active_session.read().await;
        active.clone()
    }
    
    pub async fn clear_active(&self) {
        let mut active = self.active_session.write().await;
        *active = None;
    }
    
    pub async fn add_message(&self, id: &str, role: &str, content: &str) -> anyhow::Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(id) {
            session.add_message(role, content);
            self.save(session).await?;
        }
        Ok(())
    }
    
    pub async fn archive(&self, id: &str) -> anyhow::Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(id) {
            session.status = SessionStatus::Archived;
            self.save(session).await?;
        }
        Ok(())
    }
    
    pub async fn search(&self, query: &str) -> Vec<SessionInfo> {
        let query_lower = query.to_lowercase();
        let sessions = self.sessions.read().await;
        
        sessions.values()
            .filter(|s| {
                s.name.to_lowercase().contains(&query_lower) ||
                s.messages.iter().any(|m| m.content.to_lowercase().contains(&query_lower))
            })
            .map(|s| SessionInfo {
                id: s.id.clone(),
                name: s.name.clone(),
                project_path: s.project_path.clone(),
                created_at: s.created_at,
                updated_at: s.updated_at,
                message_count: s.messages.len(),
                status: s.status.clone(),
            })
            .collect()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
