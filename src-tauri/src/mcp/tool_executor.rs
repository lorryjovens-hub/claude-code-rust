use crate::mcp::{McpConnector, McpServerManager};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub server_name: String,
}

pub struct McpToolRegistry {
    manager: Arc<Mutex<McpServerManager>>,
    #[allow(dead_code)]
    active_connectors: Arc<Mutex<HashMap<String, Arc<Mutex<McpConnector>>>>>,
}

impl McpToolRegistry {
    pub fn new(manager: Arc<Mutex<McpServerManager>>) -> Self {
        Self {
            manager,
            active_connectors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_all_tools(&self) -> Result<Vec<McpToolDefinition>> {
        let manager = self.manager.lock().await;
        let servers = manager.list_servers_full().await;
        let mut all_tools = Vec::new();

        for server in servers {
            if !server.config.enabled || server.status.pid.is_none() {
                continue;
            }

            if let Some(connector_arc) = &server.connector {
                let connector = connector_arc.lock().await;
                let tools = connector.list_tools().await.unwrap_or_default();
                
                for tool in tools {
                    all_tools.push(McpToolDefinition {
                        name: tool.name.clone(),
                        description: tool.description.clone(),
                        input_schema: tool.input_schema.clone(),
                        server_name: server.config.name.clone(),
                    });
                }
            }
        }

        Ok(all_tools)
    }

    pub async fn execute_tool(&self, name: &str, arguments: serde_json::Value) -> Result<serde_json::Value> {
        let manager = self.manager.lock().await;
        let servers = manager.list_servers_full().await;

        for server in servers {
            if !server.config.enabled || server.status.pid.is_none() {
                continue;
            }

            if let Some(connector_arc) = &server.connector {
                let connector = connector_arc.lock().await;
                let tools = connector.list_tools().await.unwrap_or_default();
                
                if tools.iter().any(|t| t.name == name) {
                    return connector.call_tool(name, arguments).await;
                }
            }
        }

        Err(anyhow::anyhow!("MCP tool '{}' not found in any running server", name))
    }

    pub async fn is_mcp_tool(&self, name: &str) -> bool {
        match self.get_all_tools().await {
            Ok(tools) => tools.iter().any(|t| t.name == name),
            Err(_) => false,
        }
    }
}
