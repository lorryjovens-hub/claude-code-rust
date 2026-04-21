 //! Application State Module

use crate::config::Settings;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Application state shared across the application
pub struct AppState {
    /// Configuration settings
    pub settings: Settings,
    /// Session history
    pub session_history: Arc<RwLock<Vec<SessionEntry>>>,
    /// Current conversation
    pub current_conversation: Arc<RwLock<Conversation>>,
    /// Tool registry
    pub tool_registry: Arc<RwLock<ToolRegistryState>>,
    /// Memory state
    pub memory_state: Arc<RwLock<MemoryState>>,
    /// Running flag
    pub running: Arc<RwLock<bool>>,
}

/// A session entry in history
#[derive(Debug, Clone)]
pub struct SessionEntry {
    /// Session ID
    pub id: String,
    /// Session start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Session end time
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Number of messages
    pub message_count: usize,
    /// Session summary
    pub summary: Option<String>,
}

/// Current conversation state
#[derive(Debug, Clone)]
pub struct Conversation {
    /// Conversation ID
    pub id: String,
    /// Messages in the conversation
    pub messages: Vec<Message>,
    /// Current model
    pub model: String,
    /// Total tokens used
    pub total_tokens: usize,
    /// Total cost
    pub total_cost: f64,
}

/// A message in the conversation
#[derive(Debug, Clone)]
pub struct Message {
    /// Message role
    pub role: MessageRole,
    /// Message content
    pub content: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Tool calls (if any)
    pub tool_calls: Vec<ToolCall>,
}

#[derive(Debug, Clone)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    /// Tool name
    pub name: String,
    /// Tool input
    pub input: serde_json::Value,
    /// Tool output
    pub output: Option<serde_json::Value>,
    /// Status
    pub status: ToolCallStatus,
}

#[derive(Debug, Clone)]
pub enum ToolCallStatus {
    Pending,
    Running,
    Success,
    Error,
}

/// Tool registry state
#[derive(Debug, Clone)]
pub struct ToolRegistryState {
    /// Registered tools
    pub tools: Vec<ToolInfo>,
}

#[derive(Debug, Clone)]
pub struct ToolInfo {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Tool status
    pub enabled: bool,
}

/// Memory state
#[derive(Debug, Clone, Default)]
pub struct MemoryState {
    /// Number of memories stored
    pub memory_count: usize,
    /// Last consolidation time
    pub last_consolidation: Option<chrono::DateTime<chrono::Utc>>,
    /// Session count since last consolidation
    pub sessions_since_consolidation: usize,
}

impl AppState {
    /// Create a new application state
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            session_history: Arc::new(RwLock::new(Vec::new())),
            current_conversation: Arc::new(RwLock::new(Conversation::new())),
            tool_registry: Arc::new(RwLock::new(ToolRegistryState::default())),
            memory_state: Arc::new(RwLock::new(MemoryState::default())),
            running: Arc::new(RwLock::new(true)),
        }
    }

    /// Add a message to the current conversation
    pub async fn add_message(&self, role: MessageRole, content: String) {
        let mut conversation = self.current_conversation.write().await;
        conversation.messages.push(Message {
            role,
            content,
            timestamp: chrono::Utc::now(),
            tool_calls: Vec::new(),
        });
    }

    /// Get the current conversation messages
    pub async fn get_messages(&self) -> Vec<Message> {
        let conversation = self.current_conversation.read().await;
        conversation.messages.clone()
    }

    /// Clear the current conversation
    pub async fn clear_conversation(&self) {
        let mut conversation = self.current_conversation.write().await;
        conversation.messages.clear();
        conversation.total_tokens = 0;
        conversation.total_cost = 0.0;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(Settings::default())
    }
}

impl Conversation {
    /// Create a new conversation
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            messages: Vec::new(),
            model: "sonnet".to_string(),
            total_tokens: 0,
            total_cost: 0.0,
        }
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
}

impl Default for Conversation {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ToolRegistryState {
    fn default() -> Self {
        Self {
            tools: vec![
                ToolInfo {
                    name: "file_read".to_string(),
                    description: "Read file contents".to_string(),
                    enabled: true,
                },
                ToolInfo {
                    name: "file_edit".to_string(),
                    description: "Edit file contents".to_string(),
                    enabled: true,
                },
                ToolInfo {
                    name: "file_write".to_string(),
                    description: "Write new file".to_string(),
                    enabled: true,
                },
                ToolInfo {
                    name: "execute_command".to_string(),
                    description: "Execute shell command".to_string(),
                    enabled: true,
                },
                ToolInfo {
                    name: "search".to_string(),
                    description: "Search for patterns in files".to_string(),
                    enabled: true,
                },
                ToolInfo {
                    name: "list_files".to_string(),
                    description: "List directory contents".to_string(),
                    enabled: true,
                },
            ],
        }
    }
}