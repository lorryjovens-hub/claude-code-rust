use crate::mcp::registry::{McpRegistry, McpRegistryEvent};
use std::sync::Arc;
use tracing::{debug, info, warn};

pub struct McpToolBridge {
    registry: Arc<McpRegistry>,
    registered_tools: Vec<String>,
}

impl McpToolBridge {
    pub fn new(registry: Arc<McpRegistry>) -> Self {
        Self {
            registry,
            registered_tools: Vec::new(),
        }
    }

    pub async fn sync_all(&mut self) -> Result<usize, String> {
        self.cleanup_old_tools().await;

        let mcp_tools = self.registry.get_all_mcp_tools().await;
        let count = mcp_tools.len();

        for tool in &mcp_tools {
            self.registered_tools.push(tool.name.clone());
        }

        info!(target: "mcp_bridge", "MCP tool bridge sync complete: {} tools registered", count);

        Ok(count)
    }

    async fn cleanup_old_tools(&mut self) {
        self.registered_tools.clear();
    }

    pub fn get_registered_tools(&self) -> &[String] {
        &self.registered_tools
    }

    pub async fn auto_sync_loop(&mut self) {
        let mut rx = self.registry.subscribe();

        loop {
            match rx.recv().await {
                Ok(event) => {
                    match event {
                        McpRegistryEvent::ToolRegistered { server_id, tool_count } => {
                            debug!(target: "mcp_bridge", "MCP server {} tools updated: {} tools", server_id, tool_count);
                        }
                        McpRegistryEvent::ServerConnected(id) => {
                            info!(target: "mcp_bridge", "MCP server connected, syncing tools: {}", id);
                        }
                        McpRegistryEvent::ServerDisconnected(id) => {
                            info!(target: "mcp_bridge", "MCP server disconnected, cleaning up tools: {}", id);
                            let prefix = format!("mcp_{}:", id);
                            self.registered_tools.retain(|t| !t.starts_with(&prefix));
                        }
                        _ => {}
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    warn!(target: "mcp_bridge", "MCP events lagged: {} skipped", n);
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    }
}