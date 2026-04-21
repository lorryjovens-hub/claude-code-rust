//! Magic Docs Service - Automatic documentation maintenance
//!
//! Magic Docs automatically maintains markdown documentation files marked with special headers.
//! When a file with "# MAGIC DOC: [title]" is read, it runs periodically in the background
//! using a forked subagent to update the document with new learnings from the conversation.

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::AppState;

const MAGIC_DOC_HEADER_PATTERN: &str = r"#\s*MAGIC\s+DOC:\s*(.+?)(?:\n|$)";
const ITALICS_PATTERN: &str = r"[*_](.+?)[*_]";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicDocHeader {
    pub title: String,
    pub instructions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicDocInfo {
    pub path: String,
    pub title: String,
    pub instructions: Option<String>,
    pub last_updated: DateTime<Utc>,
    pub update_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicDocsConfig {
    pub enabled: bool,
    pub auto_update: bool,
    pub update_interval_hours: i64,
    pub max_docs: usize,
}

impl Default for MagicDocsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_update: true,
            update_interval_hours: 1,
            max_docs: 100,
        }
    }
}

pub struct MagicDocsService {
    state: Arc<RwLock<AppState>>,
    config: MagicDocsConfig,
    tracked_docs: Arc<RwLock<HashMap<String, MagicDocInfo>>>,
}

impl MagicDocsService {
    pub fn new(state: Arc<RwLock<AppState>>, config: Option<MagicDocsConfig>) -> Self {
        Self {
            state,
            config: config.unwrap_or_default(),
            tracked_docs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn detect_magic_doc_header(&self, content: &str) -> Option<MagicDocHeader> {
        let header_re = Regex::new(MAGIC_DOC_HEADER_PATTERN).ok()?;
        let italics_re = Regex::new(ITALICS_PATTERN).ok()?;

        let header_match = header_re.captures(content)?;
        let title = header_match.get(1)?.as_str().trim().to_string();

        let lines: Vec<&str> = content.lines().collect();
        for line in lines.iter().skip(1) {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(caps) = italics_re.captures(trimmed) {
                return Some(MagicDocHeader {
                    title,
                    instructions: Some(caps.get(1)?.as_str().trim().to_string()),
                });
            }
            break;
        }

        Some(MagicDocHeader {
            title,
            instructions: None,
        })
    }

    pub async fn register_magic_doc(&self, file_path: &str, header: MagicDocHeader) {
        let mut docs = self.tracked_docs.write().await;
        
        if !docs.contains_key(file_path) {
            docs.insert(
                file_path.to_string(),
                MagicDocInfo {
                    path: file_path.to_string(),
                    title: header.title,
                    instructions: header.instructions,
                    last_updated: Utc::now(),
                    update_count: 0,
                },
            );
            println!("📚 Magic Doc registered: {}", file_path);
        }
    }

    pub async fn check_file(&self, file_path: &str) -> Option<MagicDocHeader> {
        let path = PathBuf::from(file_path);
        
        if !path.exists() {
            return None;
        }

        if let Ok(content) = tokio::fs::read_to_string(&path).await {
            return self.detect_magic_doc_header(&content);
        }

        None
    }

    pub async fn update_magic_doc(&self, file_path: &str, context: &str) -> anyhow::Result<()> {
        let mut docs = self.tracked_docs.write().await;
        
        if let Some(doc_info) = docs.get_mut(file_path) {
            let path = PathBuf::from(file_path);
            
            if !path.exists() {
                docs.remove(file_path);
                return Ok(());
            }

            let current_content = tokio::fs::read_to_string(&path).await?;
            
            let updated_content = self.generate_update(&doc_info, &current_content, context).await?;
            
            tokio::fs::write(&path, &updated_content).await?;
            
            doc_info.last_updated = Utc::now();
            doc_info.update_count += 1;
            
            println!("📚 Magic Doc updated: {} (update #{})", file_path, doc_info.update_count);
        }

        Ok(())
    }

    async fn generate_update(
        &self,
        doc_info: &MagicDocInfo,
        current_content: &str,
        context: &str,
    ) -> anyhow::Result<String> {
        let state = self.state.read().await;
        let api_client = crate::api::ApiClient::new(state.settings.clone());

        let instructions = doc_info.instructions.as_deref().unwrap_or(
            "Update this document with new learnings and insights from the conversation."
        );

        let prompt = format!(
            r#"You are a documentation maintainer. Update the following Magic Doc with new information.

# Current Document: {}

## Instructions
{}

## Current Content
```
{}
```

## New Context to Incorporate
```
{}
```

Please update the document while:
1. Preserving the MAGIC DOC header
2. Maintaining the existing structure
3. Adding new insights and learnings
4. Removing outdated information
5. Keeping the document concise and useful

Output only the updated document content, nothing else."#,
            doc_info.title,
            instructions,
            current_content,
            context
        );

        let messages = vec![
            crate::api::ChatMessage {
                role: "user".to_string(),
                content: Some(prompt),
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        let response = api_client.chat(messages, None).await?;

        if let Some(choice) = response.choices.first() {
            return Ok(choice.message.content.clone().unwrap_or_default());
        }

        Ok(current_content.to_string())
    }

    pub async fn get_tracked_docs(&self) -> Vec<MagicDocInfo> {
        let docs = self.tracked_docs.read().await;
        docs.values().cloned().collect()
    }

    pub async fn get_status(&self) -> MagicDocsStatus {
        let docs = self.tracked_docs.read().await;
        
        MagicDocsStatus {
            enabled: self.config.enabled,
            auto_update: self.config.auto_update,
            tracked_count: docs.len(),
            docs: docs.values().cloned().collect(),
        }
    }

    pub async fn remove_doc(&self, file_path: &str) {
        let mut docs = self.tracked_docs.write().await;
        docs.remove(file_path);
    }

    pub async fn clear_all(&self) {
        let mut docs = self.tracked_docs.write().await;
        docs.clear();
        println!("📚 All Magic Docs cleared");
    }

    pub async fn save_state(&self) -> anyhow::Result<()> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let state_path = home.join(".claude-code").join("magic_docs_state.json");
        
        let docs = self.tracked_docs.read().await;
        let docs_vec: Vec<MagicDocInfo> = docs.values().cloned().collect();
        
        let content = serde_json::to_string_pretty(&docs_vec)?;
        tokio::fs::write(&state_path, content).await?;

        Ok(())
    }

    pub async fn load_state(&self) -> anyhow::Result<()> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let state_path = home.join(".claude-code").join("magic_docs_state.json");
        
        if !state_path.exists() {
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&state_path).await?;
        let docs_vec: Vec<MagicDocInfo> = serde_json::from_str(&content)?;
        
        let mut docs = self.tracked_docs.write().await;
        for doc in docs_vec {
            docs.insert(doc.path.clone(), doc);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MagicDocsStatus {
    pub enabled: bool,
    pub auto_update: bool,
    pub tracked_count: usize,
    pub docs: Vec<MagicDocInfo>,
}
