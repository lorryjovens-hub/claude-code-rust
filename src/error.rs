//! Error types for Claude Code

use std::fmt;

/// Result type for Claude Code operations
pub type Result<T> = std::result::Result<T, ClaudeError>;

/// Main error type for Claude Code
#[derive(Debug)]
pub enum ClaudeError {
    /// Configuration error
    Config(String),
    
    /// IO error
    Io(std::io::Error),
    
    /// File error
    File(String),
    
    /// Network error
    Network(reqwest::Error),
    
    /// Serialization/deserialization error
    Serialization(serde_json::Error),
    
    /// Tool execution error
    Tool(String),
    
    /// Command error
    Command(String),
    
    /// Authentication error
    Auth(String),
    
    /// Permission error
    Permission(String),
    
    /// Bridge error
    Bridge(String),
    
    /// MCP (Model Context Protocol) error
    Mcp(String),
    
    /// State error
    State(String),
    
    /// Agent error
    Agent(String),
    
    /// Not implemented
    NotImplemented(String),
    
    /// Any other error
    Other(String),
}

impl fmt::Display for ClaudeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClaudeError::Config(msg) => write!(f, "Configuration error: {}", msg),
            ClaudeError::Io(err) => write!(f, "IO error: {}", err),
            ClaudeError::File(msg) => write!(f, "File error: {}", msg),
            ClaudeError::Network(err) => write!(f, "Network error: {}", err),
            ClaudeError::Serialization(err) => write!(f, "Serialization error: {}", err),
            ClaudeError::Tool(msg) => write!(f, "Tool error: {}", msg),
            ClaudeError::Command(msg) => write!(f, "Command error: {}", msg),
            ClaudeError::Auth(msg) => write!(f, "Authentication error: {}", msg),
            ClaudeError::Permission(msg) => write!(f, "Permission error: {}", msg),
            ClaudeError::Bridge(msg) => write!(f, "Bridge error: {}", msg),
            ClaudeError::Mcp(msg) => write!(f, "MCP error: {}", msg),
            ClaudeError::State(msg) => write!(f, "State error: {}", msg),
            ClaudeError::Agent(msg) => write!(f, "Agent error: {}", msg),
            ClaudeError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            ClaudeError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for ClaudeError {}

impl From<std::io::Error> for ClaudeError {
    fn from(err: std::io::Error) -> Self {
        ClaudeError::Io(err)
    }
}

impl From<reqwest::Error> for ClaudeError {
    fn from(err: reqwest::Error) -> Self {
        ClaudeError::Network(err)
    }
}

impl From<serde_json::Error> for ClaudeError {
    fn from(err: serde_json::Error) -> Self {
        ClaudeError::Serialization(err)
    }
}

impl From<anyhow::Error> for ClaudeError {
    fn from(err: anyhow::Error) -> Self {
        ClaudeError::Other(err.to_string())
    }
}

impl From<walkdir::Error> for ClaudeError {
    fn from(err: walkdir::Error) -> Self {
        ClaudeError::Io(err.into())
    }
}

impl From<regex::Error> for ClaudeError {
    fn from(err: regex::Error) -> Self {
        ClaudeError::Other(err.to_string())
    }
}

impl From<&str> for ClaudeError {
    fn from(msg: &str) -> Self {
        ClaudeError::Other(msg.to_string())
    }
}

impl From<String> for ClaudeError {
    fn from(msg: String) -> Self {
        ClaudeError::Other(msg)
    }
}
