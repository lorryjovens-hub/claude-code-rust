use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    PermissionChecked,
    PermissionGranted,
    PermissionDenied,
    PermissionConfirmed,
    ToolExecuted,
    ToolCancelled,
    ToolStarted,
    ToolCompleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: String,
    pub action: AuditAction,
    pub tool_name: String,
    pub conversation_id: String,
    pub message_id: Option<String>,
    pub user_id: Option<String>,
    pub result: String,
    pub details: Option<serde_json::Value>,
    pub duration_ms: Option<u64>,
}

impl AuditEntry {
    pub fn new(action: AuditAction, tool_name: &str, conversation_id: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            action,
            tool_name: tool_name.to_string(),
            conversation_id: conversation_id.to_string(),
            message_id: None,
            user_id: None,
            result: String::new(),
            details: None,
            duration_ms: None,
        }
    }

    pub fn with_message_id(mut self, message_id: &str) -> Self {
        self.message_id = Some(message_id.to_string());
        self
    }

    pub fn with_user_id(mut self, user_id: Option<&str>) -> Self {
        self.user_id = user_id.map(|s| s.to_string());
        self
    }

    pub fn with_result(mut self, result: &str) -> Self {
        self.result = result.to_string();
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_duration_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }
}

pub struct AuditLogger {
    entries: Mutex<VecDeque<AuditEntry>>,
    max_entries: usize,
}

impl AuditLogger {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Mutex::new(VecDeque::with_capacity(max_entries)),
            max_entries,
        }
    }

    pub fn log(&self, entry: AuditEntry) {
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        if entries.len() >= self.max_entries {
            entries.pop_front();
        }
        entries.push_back(entry);
    }

    pub fn log_tool_execution(
        &self,
        tool_name: &str,
        conversation_id: &str,
        message_id: Option<&str>,
        tool_input: &serde_json::Value,
        output: &str,
        is_error: bool,
        duration_ms: u64,
    ) {
        let entry = AuditEntry::new(
            if is_error { AuditAction::ToolCancelled } else { AuditAction::ToolExecuted },
            tool_name,
            conversation_id,
        )
        .with_result(if is_error { "Error" } else { "Success" })
        .with_details(serde_json::json!({
            "input": tool_input,
            "output": output,
            "is_error": is_error,
        }))
        .with_duration_ms(duration_ms);

        let entry = if let Some(msg_id) = message_id {
            entry.with_message_id(msg_id)
        } else {
            entry
        };

        self.log(entry);

        tracing::info!(
            module = "Audit",
            "Tool: {} | Conv: {} | Msg: {} | Error: {} | Duration: {}ms",
            tool_name,
            conversation_id,
            message_id.unwrap_or("N/A"),
            is_error,
            duration_ms
        );
    }

    pub fn get_entries(&self) -> Vec<AuditEntry> {
        let entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        entries.clone().into()
    }

    pub fn get_entries_by_conversation(&self, conversation_id: &str) -> Vec<AuditEntry> {
        let entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        entries
            .iter()
            .filter(|e| e.conversation_id == conversation_id)
            .cloned()
            .collect()
    }

    pub fn get_entries_by_tool(&self, tool_name: &str) -> Vec<AuditEntry> {
        let entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        entries
            .iter()
            .filter(|e| e.tool_name == tool_name)
            .cloned()
            .collect()
    }

    pub fn clear(&self) {
        let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        entries.clear();
    }
}

pub type AuditLoggerRef = Arc<AuditLogger>;
