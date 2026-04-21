//! MCP (Model Context Protocol) Module
//!
//! Complete implementation of MCP server with:
//! - Tool registration and execution
//! - Resource management
//! - Prompt system
//! - Sampling support

pub mod tools;
pub mod resources;
pub mod prompts;
pub mod sampling;
pub mod server;
pub mod transport;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

pub use tools::{McpTool, ToolRegistry, ToolExecutor};
pub use resources::{Resource, ResourceManager};
pub use prompts::{Prompt, PromptManager};
pub use sampling::{SamplingRequest, SamplingManager};
pub use server::McpServer;
pub use crate::config::mcp_config::{McpConfig, McpServerStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    pub status: McpServerStatus,
    pub tools_count: usize,
    pub resources_count: usize,
    pub prompts_count: usize,
    pub started_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpMessage {
    pub jsonrpc: String,
    pub id: Option<i64>,
    pub method: Option<String>,
    pub params: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error: Option<McpError>,
}

impl McpMessage {
    pub fn request(id: i64, method: &str, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            method: Some(method.to_string()),
            params,
            result: None,
            error: None,
        }
    }
    
    pub fn response(id: i64, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            method: None,
            params: None,
            result: Some(result),
            error: None,
        }
    }
    
    pub fn error_response(id: i64, code: i32, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            method: None,
            params: None,
            result: None,
            error: Some(McpError { code, message: message.to_string() }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
}

pub struct McpManager {
    servers: Arc<RwLock<HashMap<String, McpServerConnection>>>,
    tool_registry: Arc<ToolRegistry>,
    resource_manager: Arc<ResourceManager>,
    prompt_manager: Arc<PromptManager>,
    sampling_manager: Arc<SamplingManager>,
}

struct McpServerConnection {
    config: McpConfig,
    process: Option<tokio::process::Child>,
    started_at: Option<DateTime<Utc>>,
    last_error: Option<String>,
}

impl McpManager {
    pub fn new() -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            tool_registry: Arc::new(ToolRegistry::new()),
            resource_manager: Arc::new(ResourceManager::new()),
            prompt_manager: Arc::new(PromptManager::new()),
            sampling_manager: Arc::new(SamplingManager::new()),
        }
    }
    
    pub async fn list_servers(&self) -> anyhow::Result<Vec<McpServerInfo>> {
        let settings = crate::config::Settings::load()?;
        let servers = self.servers.read().await;
        
        Ok(settings.mcp_servers.iter().map(|config| {
            let conn = servers.get(&config.name);
            McpServerInfo {
                name: config.name.clone(),
                status: conn.map_or(config.status.clone(), |c| c.config.status.clone()),
                tools_count: 0,
                resources_count: 0,
                prompts_count: 0,
                started_at: conn.and_then(|c| c.started_at),
                last_error: conn.and_then(|c| c.last_error.clone()),
            }
        }).collect())
    }
    
    pub async fn add_server(&self, config: McpConfig) -> anyhow::Result<()> {
        let mut settings = crate::config::Settings::load()?;
        settings.mcp_servers.push(config);
        settings.save()?;
        Ok(())
    }
    
    pub async fn remove_server(&self, name: &str) -> anyhow::Result<()> {
        self.stop_server(name).await?;
        
        let mut settings = crate::config::Settings::load()?;
        settings.mcp_servers.retain(|s| s.name != name);
        settings.save()?;
        Ok(())
    }
    
    pub async fn start_server(&self, name: &str) -> anyhow::Result<()> {
        let settings = crate::config::Settings::load()?;
        let config = settings.mcp_servers.iter()
            .find(|s| s.name == name)
            .ok_or_else(|| anyhow::anyhow!("Server not found: {}", name))?
            .clone();
        
        let mut cmd = tokio::process::Command::new(&config.command);
        cmd.args(&config.args);
        
        for (key, value) in &config.env {
            cmd.env(key, value);
        }
        
        let mut config = config;
        config.status = McpServerStatus::Starting;
        
        match cmd.spawn() {
            Ok(process) => {
                let mut servers = self.servers.write().await;
                servers.insert(name.to_string(), McpServerConnection {
                    config: McpConfig {
                        status: McpServerStatus::Running,
                        ..config
                    },
                    process: Some(process),
                    started_at: Some(Utc::now()),
                    last_error: None,
                });
                println!("✅ MCP server started: {}", name);
            }
            Err(e) => {
                let mut servers = self.servers.write().await;
                config.status = McpServerStatus::Error;
                servers.insert(name.to_string(), McpServerConnection {
                    config,
                    process: None,
                    started_at: None,
                    last_error: Some(e.to_string()),
                });
                println!("❌ Failed to start MCP server {}: {}", name, e);
            }
        }
        
        Ok(())
    }
    
    pub async fn stop_server(&self, name: &str) -> anyhow::Result<()> {
        let mut servers = self.servers.write().await;
        if let Some(conn) = servers.get_mut(name) {
            if let Some(mut process) = conn.process.take() {
                let _ = process.kill().await;
            }
            conn.config.status = McpServerStatus::Stopped;
            println!("🛑 MCP server stopped: {}", name);
        }
        Ok(())
    }
    
    pub async fn restart_server(&self, name: &str) -> anyhow::Result<()> {
        self.stop_server(name).await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        self.start_server(name).await
    }
    
    pub async fn start_all(&self) -> anyhow::Result<()> {
        let settings = crate::config::Settings::load()?;
        for server in &settings.mcp_servers {
            if server.auto_start {
                let _ = self.start_server(&server.name).await;
            }
        }
        Ok(())
    }
    
    pub async fn stop_all(&self) -> anyhow::Result<()> {
        let servers = self.servers.read().await;
        let names: Vec<String> = servers.keys().cloned().collect();
        drop(servers);
        
        for name in names {
            self.stop_server(&name).await?;
        }
        Ok(())
    }
    
    pub fn tool_registry(&self) -> Arc<ToolRegistry> {
        self.tool_registry.clone()
    }
    
    pub fn resource_manager(&self) -> Arc<ResourceManager> {
        self.resource_manager.clone()
    }
    
    pub fn prompt_manager(&self) -> Arc<PromptManager> {
        self.prompt_manager.clone()
    }
    
    pub fn sampling_manager(&self) -> Arc<SamplingManager> {
        self.sampling_manager.clone()
    }
}

impl Default for McpManager {
    fn default() -> Self {
        Self::new()
    }
}
