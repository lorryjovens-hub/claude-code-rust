use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db::DbManager;

use super::message_router::UnifiedMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub platform: String,
    pub user_id: String,
    pub chat_id: String,
    pub thread_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: u64,
    pub last_message_content: Option<String>,
    pub metadata: serde_json::Value,
}

impl SessionContext {
    pub fn session_key(&self) -> String {
        format!("{}:{}:{}", self.platform, self.user_id, self.chat_id)
    }

    pub fn thread_key(&self) -> String {
        match &self.thread_id {
            Some(thread_id) => {
                format!("{}:{}:{}:{}", self.platform, self.user_id, self.chat_id, thread_id)
            }
            None => self.session_key(),
        }
    }
}

impl From<&UnifiedMessage> for SessionContext {
    fn from(msg: &UnifiedMessage) -> Self {
        Self {
            platform: msg.platform.clone(),
            user_id: msg.user_id.clone(),
            chat_id: msg.chat_id.clone(),
            thread_id: msg.thread_id.clone(),
            created_at: msg.timestamp,
            updated_at: msg.timestamp,
            message_count: 1,
            last_message_content: Some(msg.content.clone()),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub id: String,
    pub session_key: String,
    pub role: String,
    pub content: String,
    pub message_type: String,
    pub timestamp: DateTime<Utc>,
    pub raw_data: Option<serde_json::Value>,
}

pub struct SessionManager {
    db: Arc<DbManager>,
    sessions: RwLock<HashMap<String, SessionContext>>,
}

impl SessionManager {
    pub fn new(db: Arc<DbManager>) -> Self {
        Self {
            db,
            sessions: RwLock::new(HashMap::new()),
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        self.ensure_tables().await?;
        self.load_sessions().await?;
        Ok(())
    }

    pub async fn get_or_create_session(&self, msg: &UnifiedMessage) -> Result<SessionContext> {
        let key = msg.thread_key();
        {
            let sessions = self.sessions.read().await;
            if let Some(session) = sessions.get(&key) {
                return Ok(session.clone());
            }
        }

        let session = SessionContext::from(msg);
        self.persist_session(&session).await?;

        let mut sessions = self.sessions.write().await;
        sessions.insert(key, session.clone());

        Ok(session)
    }

    pub async fn get_session(&self, platform: &str, user_id: &str, chat_id: &str, thread_id: Option<&str>) -> Option<SessionContext> {
        let key = match thread_id {
            Some(tid) => format!("{}:{}:{}:{}", platform, user_id, chat_id, tid),
            None => format!("{}:{}:{}", platform, user_id, chat_id),
        };
        let sessions = self.sessions.read().await;
        sessions.get(&key).cloned()
    }

    pub async fn update_session(&self, msg: &UnifiedMessage) -> Result<SessionContext> {
        let key = msg.thread_key();
        let mut sessions = self.sessions.write().await;

        let session = sessions.entry(key.clone()).or_insert_with(|| SessionContext::from(msg));
        session.updated_at = Utc::now();
        session.message_count += 1;
        session.last_message_content = Some(msg.content.clone());

        let session_clone = session.clone();
        drop(sessions);

        self.persist_session(&session_clone).await?;
        self.persist_message(&session_clone, msg).await?;

        Ok(session_clone)
    }

    pub async fn list_sessions(&self) -> Vec<SessionContext> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    pub async fn list_sessions_by_platform(&self, platform: &str) -> Vec<SessionContext> {
        let sessions = self.sessions.read().await;
        sessions
            .values()
            .filter(|s| s.platform == platform)
            .cloned()
            .collect()
    }

    pub async fn delete_session(&self, platform: &str, user_id: &str, chat_id: &str, thread_id: Option<&str>) -> Result<()> {
        let key = match thread_id {
            Some(tid) => format!("{}:{}:{}:{}", platform, user_id, chat_id, tid),
            None => format!("{}:{}:{}", platform, user_id, chat_id),
        };

        {
            let mut sessions = self.sessions.write().await;
            sessions.remove(&key);
        }

        let db = self.db.clone();
        let key_clone = key.clone();
        tokio::task::spawn_blocking(move || -> Result<()> {
            db.with_conn(|conn| {
                conn.execute(
                    "DELETE FROM im_sessions WHERE session_key = ?1",
                    rusqlite::params![key_clone],
                )?;
                conn.execute(
                    "DELETE FROM im_session_messages WHERE session_key = ?1",
                    rusqlite::params![key_clone],
                )?;
                Ok::<(), anyhow::Error>(())
            })??;
            Ok(())
        })
        .await??;

        Ok(())
    }

    pub async fn get_session_messages(
        &self,
        platform: &str,
        user_id: &str,
        chat_id: &str,
        thread_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SessionMessage>> {
        let key = match thread_id {
            Some(tid) => format!("{}:{}:{}:{}", platform, user_id, chat_id, tid),
            None => format!("{}:{}:{}", platform, user_id, chat_id),
        };

        let db = self.db.clone();
        let key_clone = key.clone();
        let messages = tokio::task::spawn_blocking(move || -> Result<Vec<SessionMessage>> {
            db.with_conn(|conn| {
                let mut stmt = conn.prepare_cached(
                    "SELECT id, session_key, role, content, message_type, timestamp, raw_data 
                     FROM im_session_messages 
                     WHERE session_key = ?1 
                     ORDER BY timestamp DESC 
                     LIMIT ?2"
                )?;
                let rows = stmt.query_map(
                    rusqlite::params![key_clone, limit as i64],
                    |row| {
                        let raw_data_str: Option<String> = row.get(6)?;
                        let raw_data = raw_data_str
                            .and_then(|s| serde_json::from_str(&s).ok());
                        let ts_str: String = row.get(5)?;
                        let timestamp = DateTime::parse_from_rfc3339(&ts_str)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now());
                        Ok(SessionMessage {
                            id: row.get(0)?,
                            session_key: row.get(1)?,
                            role: row.get(2)?,
                            content: row.get(3)?,
                            message_type: row.get(4)?,
                            timestamp,
                            raw_data,
                        })
                    },
                )?;
                let mut result = Vec::new();
                for row in rows {
                    result.push(row?);
                }
                Ok(result)
            })?
        })
        .await??;

        Ok(messages)
    }

    pub async fn add_message_to_session(
        &self,
        platform: &str,
        user_id: &str,
        chat_id: &str,
        thread_id: Option<&str>,
        role: &str,
        content: &str,
        message_type: &str,
        raw_data: Option<serde_json::Value>,
    ) -> Result<SessionMessage> {
        let key = match thread_id {
            Some(tid) => format!("{}:{}:{}:{}", platform, user_id, chat_id, tid),
            None => format!("{}:{}:{}", platform, user_id, chat_id),
        };

        let msg = SessionMessage {
            id: uuid::Uuid::new_v4().to_string(),
            session_key: key.clone(),
            role: role.to_string(),
            content: content.to_string(),
            message_type: message_type.to_string(),
            timestamp: Utc::now(),
            raw_data: raw_data.clone(),
        };

        let db = self.db.clone();
        let msg_clone = msg.clone();
        tokio::task::spawn_blocking(move || -> Result<()> {
            let raw_data_str = msg_clone.raw_data.as_ref().map(|v| v.to_string());
            db.with_conn(|conn| {
                let mut stmt = conn.prepare_cached(
                    "INSERT INTO im_session_messages 
                     (id, session_key, role, content, message_type, timestamp, raw_data) 
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
                )?;
                stmt.execute(rusqlite::params![
                    msg_clone.id,
                    msg_clone.session_key,
                    msg_clone.role,
                    msg_clone.content,
                    msg_clone.message_type,
                    msg_clone.timestamp.to_rfc3339(),
                    raw_data_str,
                ])?;
                Ok::<(), anyhow::Error>(())
            })??;
            Ok(())
        })
        .await??;

        Ok(msg)
    }

    pub async fn set_session_metadata(
        &self,
        platform: &str,
        user_id: &str,
        chat_id: &str,
        thread_id: Option<&str>,
        metadata: serde_json::Value,
    ) -> Result<()> {
        let key = match thread_id {
            Some(tid) => format!("{}:{}:{}:{}", platform, user_id, chat_id, tid),
            None => format!("{}:{}:{}", platform, user_id, chat_id),
        };

        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(&key) {
                session.metadata = metadata.clone();
            }
        }

        let metadata_str = serde_json::to_string(&metadata)?;
        let db = self.db.clone();
        let key_clone = key.clone();
        tokio::task::spawn_blocking(move || -> Result<()> {
            db.with_conn(|conn| {
                conn.execute(
                    "UPDATE im_sessions SET metadata = ?1, updated_at = ?2 WHERE session_key = ?3",
                    rusqlite::params![metadata_str, Utc::now().to_rfc3339(), key_clone],
                )?;
                Ok::<(), anyhow::Error>(())
            })??;
            Ok(())
        })
        .await??;

        Ok(())
    }

    async fn ensure_tables(&self) -> Result<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || -> Result<()> {
            db.with_conn(|conn| {
                conn.execute_batch(
                    r#"
                    CREATE TABLE IF NOT EXISTS im_sessions (
                        session_key TEXT PRIMARY KEY,
                        platform TEXT NOT NULL,
                        user_id TEXT NOT NULL,
                        chat_id TEXT NOT NULL,
                        thread_id TEXT,
                        created_at TEXT NOT NULL,
                        updated_at TEXT NOT NULL,
                        message_count INTEGER DEFAULT 0,
                        last_message_content TEXT,
                        metadata TEXT
                    );

                    CREATE TABLE IF NOT EXISTS im_session_messages (
                        id TEXT PRIMARY KEY,
                        session_key TEXT NOT NULL,
                        role TEXT NOT NULL,
                        content TEXT NOT NULL,
                        message_type TEXT NOT NULL,
                        timestamp TEXT NOT NULL,
                        raw_data TEXT,
                        FOREIGN KEY (session_key) REFERENCES im_sessions(session_key) ON DELETE CASCADE
                    );

                    CREATE INDEX IF NOT EXISTS idx_im_sessions_platform ON im_sessions(platform);
                    CREATE INDEX IF NOT EXISTS idx_im_sessions_user ON im_sessions(user_id);
                    CREATE INDEX IF NOT EXISTS idx_im_sessions_chat ON im_sessions(chat_id);
                    CREATE INDEX IF NOT EXISTS idx_im_session_messages_session ON im_session_messages(session_key);
                    CREATE INDEX IF NOT EXISTS idx_im_session_messages_timestamp ON im_session_messages(timestamp);
                    "#
                )?;
                Ok::<(), anyhow::Error>(())
            })??;
            Ok(())
        })
        .await??;

        Ok(())
    }

    async fn load_sessions(&self) -> Result<()> {
        let db = self.db.clone();
        let rows = tokio::task::spawn_blocking(move || -> Result<Vec<SessionContext>> {
            db.with_conn(|conn| {
                let mut stmt = conn.prepare_cached(
                    "SELECT session_key, platform, user_id, chat_id, thread_id, created_at, updated_at, message_count, last_message_content, metadata 
                     FROM im_sessions"
                )?;
                let rows = stmt.query_map([], |row| {
                    let metadata_str: Option<String> = row.get(9)?;
                    let metadata = metadata_str
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
                    let created_at_str: String = row.get(5)?;
                    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now());
                    let updated_at_str: String = row.get(6)?;
                    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now());
                    Ok(SessionContext {
                        platform: row.get(1)?,
                        user_id: row.get(2)?,
                        chat_id: row.get(3)?,
                        thread_id: row.get(4)?,
                        created_at,
                        updated_at,
                        message_count: row.get::<_, i64>(7)? as u64,
                        last_message_content: row.get(8)?,
                        metadata,
                    })
                })?;
                let mut result = Vec::new();
                for row in rows {
                    result.push(row?);
                }
                Ok(result)
            })?
        })
        .await??;

        let mut sessions = self.sessions.write().await;
        for session in rows {
            sessions.insert(session.session_key(), session);
        }

        Ok(())
    }

    async fn persist_session(&self, session: &SessionContext) -> Result<()> {
        let db = self.db.clone();
        let session_clone = session.clone();
        tokio::task::spawn_blocking(move || -> Result<()> {
            let metadata_str = serde_json::to_string(&session_clone.metadata).unwrap_or_default();
            db.with_conn(|conn| {
                let mut stmt = conn.prepare_cached(
                    "INSERT INTO im_sessions 
                     (session_key, platform, user_id, chat_id, thread_id, created_at, updated_at, message_count, last_message_content, metadata) 
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                     ON CONFLICT(session_key) DO UPDATE SET
                     updated_at = excluded.updated_at,
                     message_count = excluded.message_count,
                     last_message_content = excluded.last_message_content,
                     metadata = excluded.metadata"
                )?;
                stmt.execute(rusqlite::params![
                    session_clone.session_key(),
                    session_clone.platform,
                    session_clone.user_id,
                    session_clone.chat_id,
                    session_clone.thread_id,
                    session_clone.created_at.to_rfc3339(),
                    session_clone.updated_at.to_rfc3339(),
                    session_clone.message_count as i64,
                    session_clone.last_message_content,
                    metadata_str,
                ])?;
                Ok::<(), anyhow::Error>(())
            })??;
            Ok(())
        })
        .await??;

        Ok(())
    }

    async fn persist_message(&self, session: &SessionContext, msg: &UnifiedMessage) -> Result<()> {
        let db = self.db.clone();
        let session_key = session.session_key();
        let raw_data = msg.raw_data.clone();
        let content = msg.content.clone();
        let msg_type = msg.message_type.as_str().to_string();
        let timestamp = msg.timestamp;

        tokio::task::spawn_blocking(move || -> Result<()> {
            let raw_data_str = Some(raw_data.to_string());
            db.with_conn(|conn| {
                let mut stmt = conn.prepare_cached(
                    "INSERT INTO im_session_messages 
                     (id, session_key, role, content, message_type, timestamp, raw_data) 
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
                )?;
                stmt.execute(rusqlite::params![
                    uuid::Uuid::new_v4().to_string(),
                    session_key,
                    "user",
                    content,
                    msg_type,
                    timestamp.to_rfc3339(),
                    raw_data_str,
                ])?;
                Ok::<(), anyhow::Error>(())
            })??;
            Ok(())
        })
        .await??;

        Ok(())
    }
}
