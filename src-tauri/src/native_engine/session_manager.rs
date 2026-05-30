use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: Option<String>,
    pub model: String,
    pub workspace_path: PathBuf,
    pub created_at: String,
    pub updated_at: String,
    pub project_id: Option<String>,
    pub research_mode: bool,
    pub claude_session_id: Option<String>,
    pub pending_resume_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
    pub tool_calls: Option<Vec<ToolCallRecord>>,
    pub is_compact_boundary: bool,
    pub engine_uuid_synced: bool,
    pub attachments: Option<Vec<Attachment>>,
    pub research: Option<ResearchData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
    pub output: Option<String>,
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub file_id: String,
    pub file_name: String,
    pub file_type: String,
    pub mime_type: String,
    pub size: u64,
    pub source: Option<String>,
    pub gh_repo: Option<String>,
    pub gh_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchData {
    pub plan: Option<String>,
    pub sub_results: Vec<SubResearchResult>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubResearchResult {
    pub sub_question: String,
    pub findings: String,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub workspace_path: String,
    pub is_archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFile {
    pub id: String,
    pub project_id: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: u64,
    pub mime_type: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub conversations: Vec<Conversation>,
    pub messages: Vec<Message>,
    pub projects: Vec<Project>,
    pub project_files: Vec<ProjectFile>,
}

pub struct SessionManager {
    db: Database,
    db_path: PathBuf,
    workspaces_dir: PathBuf,
}

impl SessionManager {
    pub fn new(db_path: PathBuf, workspaces_dir: PathBuf) -> Self {
        let mut manager = Self {
            db: Database {
                conversations: Vec::new(),
                messages: Vec::new(),
                projects: Vec::new(),
                project_files: Vec::new(),
            },
            db_path,
            workspaces_dir,
        };
        manager.load();
        manager
    }

    pub fn load(&mut self) {
        if self.db_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&self.db_path) {
                if let Ok(db) = serde_json::from_str::<Database>(&content) {
                    self.db = db;
                    tracing::info!(module = "SessionManager", "Loaded database from {}", self.db_path.display());
                }
            }
        }
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        if let Some(parent) = self.db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.db)?;
        std::fs::write(&self.db_path, content)?;
        Ok(())
    }

    pub fn get_workspaces_dir(&self) -> &PathBuf {
        &self.workspaces_dir
    }

    pub fn create_conversation(&mut self, model: String, title: Option<String>, research_mode: bool) -> Conversation {
        let id = Uuid::new_v4().to_string();
        let workspace_path = self.workspaces_dir.join(&id);
        
        if !workspace_path.exists() {
            let _ = std::fs::create_dir_all(&workspace_path);
        }

        let now = chrono::Utc::now().to_rfc3339();
        let conv = Conversation {
            id: id.clone(),
            title,
            model,
            workspace_path,
            created_at: now.clone(),
            updated_at: now,
            project_id: None,
            research_mode,
            claude_session_id: None,
            pending_resume_at: None,
        };

        self.db.conversations.push(conv.clone());
        let _ = self.save();
        conv
    }

    pub fn get_conversation(&self, id: &str) -> Option<&Conversation> {
        self.db.conversations.iter().find(|c| c.id == id)
    }

    pub fn get_conversation_mut(&mut self, id: &str) -> Option<&mut Conversation> {
        self.db.conversations.iter_mut().find(|c| c.id == id)
    }

    pub fn list_conversations(&self) -> &[Conversation] {
        &self.db.conversations
    }

    pub fn delete_conversation(&mut self, id: &str) {
        self.db.messages.retain(|m| m.conversation_id != id);
        self.db.conversations.retain(|c| c.id != id);

        let workspace_path = self.workspaces_dir.join(id);
        if workspace_path.exists() {
            let _ = std::fs::remove_dir_all(&workspace_path);
        }

        let _ = self.save();
    }

    pub fn update_conversation(&mut self, id: &str, updates: ConversationUpdates) {
        if let Some(conv) = self.db.conversations.iter_mut().find(|c| c.id == id) {
            if let Some(title) = updates.title {
                conv.title = Some(title);
            }
            if let Some(model) = updates.model {
                conv.model = model;
            }
            if let Some(project_id) = updates.project_id {
                conv.project_id = Some(project_id);
            }
            if let Some(research_mode) = updates.research_mode {
                conv.research_mode = research_mode;
            }
            conv.updated_at = chrono::Utc::now().to_rfc3339();
        }
        let _ = self.save();
    }

    pub fn add_message(&mut self, conversation_id: &str, role: &str, content: &str) -> String {
        let id = Uuid::new_v4().to_string();
        let message = Message {
            id: id.clone(),
            conversation_id: conversation_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            tool_calls: None,
            is_compact_boundary: false,
            engine_uuid_synced: true,
            attachments: None,
            research: None,
        };

        self.db.messages.push(message);
        
        if let Some(conv) = self.db.conversations.iter_mut().find(|c| c.id == conversation_id) {
            conv.updated_at = chrono::Utc::now().to_rfc3339();
        }

        let _ = self.save();
        id
    }

    pub fn add_message_with_tool_calls(
        &mut self,
        conversation_id: &str,
        role: &str,
        content: &str,
        tool_calls: Vec<ToolCallRecord>,
    ) -> String {
        let id = Uuid::new_v4().to_string();
        let message = Message {
            id: id.clone(),
            conversation_id: conversation_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            tool_calls: Some(tool_calls),
            is_compact_boundary: false,
            engine_uuid_synced: true,
            attachments: None,
            research: None,
        };

        self.db.messages.push(message);
        
        if let Some(conv) = self.db.conversations.iter_mut().find(|c| c.id == conversation_id) {
            conv.updated_at = chrono::Utc::now().to_rfc3339();
        }

        let _ = self.save();
        id
    }

    pub fn get_messages(&self, conversation_id: &str) -> Vec<&Message> {
        let mut msgs: Vec<&Message> = self.db.messages
            .iter()
            .filter(|m| m.conversation_id == conversation_id)
            .collect();
        
        msgs.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        msgs
    }

    pub fn delete_message(&mut self, conversation_id: &str, message_id: &str) {
        let messages = self.db.messages
            .iter()
            .filter(|m| m.conversation_id == conversation_id)
            .collect::<Vec<_>>();
        
        let target_idx = messages.iter().position(|m| m.id == message_id);
        
        if let Some(idx) = target_idx {
            let target_created_at = messages[idx].created_at.clone();
            self.db.messages.retain(|m| {
                m.conversation_id != conversation_id || m.created_at < target_created_at
            });
        }

        let _ = self.save();
    }

    pub fn delete_messages_tail(&mut self, conversation_id: &str, count: usize) {
        let mut msgs: Vec<&Message> = self.db.messages
            .iter()
            .filter(|m| m.conversation_id == conversation_id)
            .collect();
        
        msgs.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        
        if msgs.len() <= count {
            self.db.messages.retain(|m| m.conversation_id != conversation_id);
        } else {
            let cutoff_idx = msgs.len() - count;
            let cutoff_time = msgs[cutoff_idx].created_at.clone();
            self.db.messages.retain(|m| {
                m.conversation_id != conversation_id || m.created_at < cutoff_time
            });
        }

        let _ = self.save();
    }

    pub fn create_project(&mut self, name: &str, description: &str) -> Project {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        
        let project = Project {
            id: id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            instructions: String::new(),
            workspace_path: String::new(),
            is_archived: false,
            created_at: now.clone(),
            updated_at: now,
        };

        self.db.projects.push(project.clone());
        let _ = self.save();
        project
    }

    pub fn list_projects(&self) -> &[Project] {
        &self.db.projects
    }

    pub fn delete_project(&mut self, id: &str) {
        self.db.projects.retain(|p| p.id != id);
        self.db.project_files.retain(|f| f.project_id != id);
        let _ = self.save();
    }

    pub fn add_project_file(&mut self, _project_id: &str, file: ProjectFile) {
        self.db.project_files.push(file);
        let _ = self.save();
    }

    pub fn get_project_files(&self, project_id: &str) -> Vec<&ProjectFile> {
        self.db.project_files
            .iter()
            .filter(|f| f.project_id == project_id)
            .collect()
    }

    pub fn delete_project_file(&mut self, project_id: &str, file_id: &str) {
        self.db.project_files.retain(|f| !(f.project_id == project_id && f.id == file_id));
        let _ = self.save();
    }
}

pub struct ConversationUpdates {
    pub title: Option<String>,
    pub model: Option<String>,
    pub project_id: Option<String>,
    pub research_mode: Option<bool>,
}
