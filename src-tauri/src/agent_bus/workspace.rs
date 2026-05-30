use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceEntry {
    pub key: String,
    pub value: serde_json::Value,
    pub last_writer: String,
    pub version: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictStrategy {
    LastWriteWins,
    Merge,
    Reject,
}

pub struct SharedWorkspace {
    data: Arc<RwLock<HashMap<String, WorkspaceEntry>>>,
    strategy: ConflictStrategy,
}

impl SharedWorkspace {
    pub fn new(strategy: ConflictStrategy) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            strategy,
        }
    }

    pub async fn read(&self, key: &str) -> Option<WorkspaceEntry> {
        let data = self.data.read().await;
        data.get(key).cloned()
    }

    pub async fn write(&self, key: &str, value: serde_json::Value, writer_id: &str) -> Result<(), String> {
        let mut data = self.data.write().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        match self.strategy {
            ConflictStrategy::LastWriteWins => {
                let version = data.get(key).map(|e| e.version + 1).unwrap_or(1);
                data.insert(key.to_string(), WorkspaceEntry {
                    key: key.to_string(),
                    value,
                    last_writer: writer_id.to_string(),
                    version,
                    updated_at: now,
                });
                Ok(())
            }
            ConflictStrategy::Merge => {
                if let Some(existing) = data.get(key) {
                    let existing_version = existing.version;
                    if let (serde_json::Value::Array(mut arr), serde_json::Value::Array(new_arr)) =
                        (existing.value.clone(), value.clone()) {
                        arr.extend(new_arr);
                        data.insert(key.to_string(), WorkspaceEntry {
                            key: key.to_string(),
                            value: serde_json::Value::Array(arr),
                            last_writer: writer_id.to_string(),
                            version: existing_version + 1,
                            updated_at: now,
                        });
                        return Ok(());
                    }
                }
                let version = data.get(key).map(|e| e.version + 1).unwrap_or(1);
                data.insert(key.to_string(), WorkspaceEntry {
                    key: key.to_string(),
                    value,
                    last_writer: writer_id.to_string(),
                    version,
                    updated_at: now,
                });
                Ok(())
            }
            ConflictStrategy::Reject => {
                if data.contains_key(key) {
                    Err(format!("Key '{}' already exists and conflict strategy is Reject", key))
                } else {
                    data.insert(key.to_string(), WorkspaceEntry {
                        key: key.to_string(),
                        value,
                        last_writer: writer_id.to_string(),
                        version: 1,
                        updated_at: now,
                    });
                    Ok(())
                }
            }
        }
    }

    pub async fn list_keys(&self) -> Vec<String> {
        let data = self.data.read().await;
        data.keys().cloned().collect()
    }

    pub async fn delete(&self, key: &str) {
        let mut data = self.data.write().await;
        data.remove(key);
    }
}
