use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub transport: McpTransport,
    pub capabilities: ServerCapabilities,
    pub metadata: HashMap<String, String>,
    pub health: McpHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpTransport {
    Stdio {
        command: String,
        args: Vec<String>,
        env: HashMap<String, String>,
    },
    Sse {
        url: String,
        headers: HashMap<String, String>,
    },
    WebSocket {
        url: String,
        headers: HashMap<String, String>,
    },
    Embedded {
        module: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServerCapabilities {
    pub tools: ToolCapabilities,
    pub resources: ResourceCapabilities,
    pub prompts: PromptCapabilities,
    pub logging: LoggingCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolCapabilities {
    pub supported: bool,
    pub max_tools: Option<usize>,
    pub tool_list: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceCapabilities {
    pub supported: bool,
    pub subscribe: bool,
    pub list_changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptCapabilities {
    pub supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoggingCapabilities {
    pub supported: bool,
    pub levels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpHealth {
    Connected,
    Disconnected,
    Error { message: String, retry_at: Option<u64> },
    Connecting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCallRequest {
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub server_id: String,
    pub timeout: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCallResult {
    pub content: Vec<ContentItem>,
    pub is_error: bool,
    pub duration: Duration,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentItem {
    Text { text: String },
    Image { data: String, mime_type: String },
    Resource { resource: ResourceContent },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    pub mime_type: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServersConfig {
    pub servers: Vec<McpServerConfigEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfigEntry {
    pub name: String,
    pub transport: McpTransport,
    pub auto_connect: bool,
    pub max_retries: u32,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone)]
pub enum McpEvent {
    Connected { server_id: String },
    Disconnected { server_id: String, reason: String },
    ToolListChanged { server_id: String, tools: Vec<String> },
    ResourceListChanged { server_id: String },
    Error { server_id: String, error: String },
    Log { server_id: String, level: String, message: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct McpHealthSummary {
    pub total: usize,
    pub connected: usize,
    pub disconnected: usize,
    pub errored: usize,
    pub details: Vec<ServerHealth>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerHealth {
    pub id: String,
    pub status: String,
    pub latency_ms: u64,
}