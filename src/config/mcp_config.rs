//! MCP Server Configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// MCP Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Server name
    pub name: String,
    /// Command to run the server
    pub command: String,
    /// Arguments for the command
    pub args: Vec<String>,
    /// Environment variables
    pub env: std::collections::HashMap<String, String>,
    /// Working directory
    pub cwd: Option<PathBuf>,
    /// Server status
    pub status: McpServerStatus,
    /// Capabilities
    pub capabilities: Vec<String>,
    /// Auto start on launch
    pub auto_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum McpServerStatus {
    Running,
    Stopped,
    Error,
    Unknown,
    Starting,
}

impl std::fmt::Display for McpServerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpServerStatus::Running => write!(f, "running"),
            McpServerStatus::Stopped => write!(f, "stopped"),
            McpServerStatus::Error => write!(f, "error"),
            McpServerStatus::Unknown => write!(f, "unknown"),
            McpServerStatus::Starting => write!(f, "starting"),
        }
    }
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            command: String::new(),
            args: Vec::new(),
            env: std::collections::HashMap::new(),
            cwd: None,
            status: McpServerStatus::Unknown,
            capabilities: Vec::new(),
            auto_start: true,
        }
    }
}

impl McpConfig {
    /// Create a new MCP server configuration
    pub fn new(name: &str, command: &str) -> Self {
        Self {
            name: name.to_string(),
            command: command.to_string(),
            args: Vec::new(),
            env: std::collections::HashMap::new(),
            cwd: None,
            status: McpServerStatus::Unknown,
            capabilities: Vec::new(),
            auto_start: true,
        }
    }

    /// Add an argument
    pub fn with_arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    /// Add an environment variable
    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.env.insert(key.to_string(), value.to_string());
        self
    }
}