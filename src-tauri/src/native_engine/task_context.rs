use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionEntry {
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub tool_output: String,
    pub is_error: bool,
    pub timestamp: String,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub conversation_id: String,
    pub current_task_id: String,
    pub workspace_path: String,
    pub environment_variables: HashMap<String, String>,
    pub execution_history: Vec<TaskExecutionEntry>,
    pub created_at: String,
    pub updated_at: String,
}

impl TaskContext {
    pub fn new(conversation_id: &str, task_id: &str, workspace_path: &str) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            conversation_id: conversation_id.to_string(),
            current_task_id: task_id.to_string(),
            workspace_path: workspace_path.to_string(),
            environment_variables: HashMap::new(),
            execution_history: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn with_env_vars(mut self, vars: HashMap<String, String>) -> Self {
        self.environment_variables = vars;
        self
    }

    pub fn push_execution(&mut self, entry: TaskExecutionEntry) {
        self.execution_history.push(entry);
        self.updated_at = Utc::now().to_rfc3339();
    }

    pub fn update_task_id(&mut self, new_task_id: &str) {
        self.current_task_id = new_task_id.to_string();
        self.updated_at = Utc::now().to_rfc3339();
    }

    pub fn update_workspace(&mut self, new_workspace: &str) {
        self.workspace_path = new_workspace.to_string();
        self.updated_at = Utc::now().to_rfc3339();
    }

    pub fn set_env_var(&mut self, key: &str, value: &str) {
        self.environment_variables.insert(key.to_string(), value.to_string());
        self.updated_at = Utc::now().to_rfc3339();
    }

    pub fn get_env_var(&self, key: &str) -> Option<&String> {
        self.environment_variables.get(key)
    }

    pub fn last_execution(&self) -> Option<&TaskExecutionEntry> {
        self.execution_history.last()
    }
}

pub struct TaskContextManager {
    contexts: Arc<Mutex<HashMap<String, TaskContext>>>,
}

impl TaskContextManager {
    pub fn new() -> Self {
        Self {
            contexts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create_context(
        &self,
        conversation_id: &str,
        task_id: &str,
        workspace_path: &str,
    ) -> Result<String> {
        let context = TaskContext::new(conversation_id, task_id, workspace_path);
        let context_id = format!("{}-{}", conversation_id, task_id);
        
        let mut contexts = self.contexts.lock().await;
        contexts.insert(context_id.clone(), context);
        
        tracing::info!(
            module = "TaskContext",
            "Created context: {} (Conv: {}, Task: {})",
            context_id, conversation_id, task_id
        );
        
        Ok(context_id)
    }

    pub async fn get_context(&self, context_id: &str) -> Option<TaskContext> {
        let contexts = self.contexts.lock().await;
        contexts.get(context_id).cloned()
    }

    pub async fn get_context_by_conversation(&self, conversation_id: &str) -> Option<TaskContext> {
        let contexts = self.contexts.lock().await;
        contexts.values()
            .find(|c| c.conversation_id == conversation_id)
            .cloned()
    }

    pub async fn update_context<F>(&self, context_id: &str, updater: F) -> Result<()>
    where
        F: FnOnce(&mut TaskContext),
    {
        let mut contexts = self.contexts.lock().await;
        let context = contexts.get_mut(context_id)
            .ok_or_else(|| anyhow::anyhow!("Context not found: {}", context_id))?;
        
        updater(context);
        context.updated_at = Utc::now().to_rfc3339();
        
        Ok(())
    }

    pub async fn push_execution(
        &self,
        context_id: &str,
        entry: TaskExecutionEntry,
    ) -> Result<()> {
        self.update_context(context_id, |ctx| {
            ctx.push_execution(entry);
        }).await
    }

    pub async fn remove_context(&self, context_id: &str) -> bool {
        let mut contexts = self.contexts.lock().await;
        contexts.remove(context_id).is_some()
    }

    pub async fn list_contexts(&self) -> Vec<TaskContext> {
        let contexts = self.contexts.lock().await;
        contexts.values().cloned().collect()
    }

    pub async fn clear_inactive_contexts(&self, max_age_minutes: u64) -> usize {
        let now = Utc::now();
        let mut contexts = self.contexts.lock().await;
        let initial_count = contexts.len();
        
        contexts.retain(|_, ctx| {
            if let Ok(updated) = chrono::DateTime::parse_from_rfc3339(&ctx.updated_at) {
                let duration = now.signed_duration_since(updated);
                duration.num_minutes() <= max_age_minutes as i64
            } else {
                false
            }
        });
        
        initial_count - contexts.len()
    }
}

pub type TaskContextManagerRef = Arc<TaskContextManager>;
