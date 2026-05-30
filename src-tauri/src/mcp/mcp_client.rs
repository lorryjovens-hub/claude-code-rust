use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

use super::{McpServerConfig, McpTool, McpResource};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub config: McpServerConfig,
    pub available_tools: Vec<McpTool>,
    pub available_resources: Vec<McpResource>,
    pub connected: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpDiscoveryResult {
    pub discovered: Vec<McpServerInfo>,
    pub known_servers: Vec<McpServerConfig>,
    pub already_connected: Vec<String>,
}

pub struct McpClientManager {
    connected_servers: HashMap<String, McpServerInfo>,
}

impl McpClientManager {
    pub fn new() -> Self {
        Self {
            connected_servers: HashMap::new(),
        }
    }

    /// Scan known locations for MCP server configurations
    pub async fn discover_servers(&self) -> Result<McpDiscoveryResult> {
        let discovered = vec![];
        let mut known_servers = vec![];

        // 1. Check Claude desktop config
        if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
            let claude_config = PathBuf::from(&home)
                .join("AppData")
                .join("Roaming")
                .join("Claude")
                .join("claude_desktop_config.json");

            if claude_config.exists() {
                if let Ok(content) = tokio::fs::read_to_string(&claude_config).await {
                    if let Ok(config) = serde_json::from_str::<Value>(&content) {
                        if let Some(servers) = config.get("mcpServers").and_then(|s| s.as_object()) {
                            for (id, value) in servers {
                                let server = McpServerConfig {
                                    id: id.clone(),
                                    name: value.get("name")
                                        .and_then(|c| c.as_str())
                                        .unwrap_or(id)
                                        .to_string(),
                                    command: value.get("command")
                                        .and_then(|c| c.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                    args: value.get("args")
                                        .and_then(|a| a.as_array())
                                        .map(|arr| arr.iter()
                                            .filter_map(|s| s.as_str().map(String::from))
                                            .collect())
                                        .unwrap_or_default(),
                                    env: value.get("env")
                                        .and_then(|e| e.as_object())
                                        .map(|obj| obj.iter()
                                            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                                            .collect())
                                        .unwrap_or_default(),
                                    enabled: value.get("enabled")
                                        .and_then(|e| e.as_bool())
                                        .unwrap_or(true),
                                };
                                known_servers.push(server);
                            }
                        }
                    }
                }
            }

            // 2. Check local .mcp directory
            let mcp_dir = PathBuf::from(&home).join(".mcp");
            if mcp_dir.exists() {
                if let Ok(mut entries) = tokio::fs::read_dir(&mcp_dir).await {
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        let path = entry.path();
                        if path.extension().and_then(|e| e.to_str()) == Some("json") {
                            if let Ok(content) = tokio::fs::read_to_string(&path).await {
                                if let Ok(server) = serde_json::from_str::<McpServerConfig>(&content) {
                                    known_servers.push(server);
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Check common MCP server registry paths
        let search_paths = vec![
            PathBuf::from(".mcp.json"),
            PathBuf::from("mcp.json"),
            PathBuf::from("claude_desktop_config.json"),
        ];

        for path in &search_paths {
            if path.exists() {
                if let Ok(content) = tokio::fs::read_to_string(path).await {
                    if let Ok(config) = serde_json::from_str::<Value>(&content) {
                        if let Some(servers) = config.get("mcpServers").and_then(|s| s.as_object()) {
                            for (id, value) in servers {
                                let cmd = value.get("command")
                                    .and_then(|c| c.as_str())
                                    .unwrap_or("");
                                known_servers.push(McpServerConfig {
                                    id: id.clone(),
                                    name: id.clone(),
                                    command: cmd.to_string(),
                                    args: value.get("args")
                                        .and_then(|a| a.as_array())
                                        .map(|arr| arr.iter()
                                            .filter_map(|s| s.as_str().map(String::from))
                                            .collect())
                                        .unwrap_or_default(),
                                    env: HashMap::new(),
                                    enabled: !value.get("disabled")
                                        .and_then(|d| d.as_bool())
                                        .unwrap_or(false),
                                });
                            }
                        }
                    }
                }
            }
        }

        let already_connected: Vec<String> = self.connected_servers.keys().cloned().collect();

        Ok(McpDiscoveryResult {
            discovered,
            known_servers,
            already_connected,
        })
    }

    /// Get the list of all connected MCP servers
    pub fn get_connected_servers(&self) -> Vec<&McpServerInfo> {
        self.connected_servers.values().collect()
    }

    /// Check if a server is connected
    pub fn is_connected(&self, server_id: &str) -> bool {
        self.connected_servers.contains_key(server_id)
    }

    /// Disconnect from an MCP server
    pub fn disconnect(&mut self, server_id: &str) {
        self.connected_servers.remove(server_id);
    }

    /// Auto-connect to all enabled servers from discovery
    pub async fn auto_connect(
        &mut self,
        servers: Vec<McpServerConfig>,
    ) -> Result<Vec<McpServerInfo>> {
        let mut results = vec![];

        for server in servers {
            if !server.enabled {
                continue;
            }
            if self.is_connected(&server.id) {
                if let Some(info) = self.connected_servers.get(&server.id) {
                    results.push(info.clone());
                }
                continue;
            }
            let info = self.connect_server(&server).await;
            match info {
                Ok(info) => {
                    self.connected_servers.insert(server.id.clone(), info.clone());
                    results.push(info);
                }
                Err(e) => {
                    tracing::warn!(
                        module = "McpClient",
                        "Failed to connect to MCP server '{}': {}",
                        server.id, e
                    );
                }
            }
        }

        Ok(results)
    }

    /// Simulate connection to an MCP server and enumerate its tools
    /// In production, this would use actual stdio/HTTP transport
    async fn connect_server(&self, config: &McpServerConfig) -> Result<McpServerInfo> {
        tracing::info!(
            module = "McpClient",
            "Connecting to MCP server '{}': {} {}",
            config.id,
            config.command,
            config.args.join(" ")
        );

        let server_tools = self.enumerate_tools(config).await?;
        let server_resources = self.enumerate_resources(config).await?;

        Ok(McpServerInfo {
            config: config.clone(),
            available_tools: server_tools,
            available_resources: server_resources,
            connected: true,
            error: None,
        })
    }

    /// Enumerate tools from an MCP server via stdio
    async fn enumerate_tools(&self, config: &McpServerConfig) -> Result<Vec<McpTool>> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        });

        match self.send_mcp_request(config, &request).await {
            Ok(response) => {
                if let Some(tools) = response.get("result")
                    .and_then(|r| r.get("tools"))
                    .and_then(|t| t.as_array())
                {
                    Ok(tools.iter().filter_map(|tool| {
                        Some(McpTool {
                            name: tool.get("name")?.as_str()?.to_string(),
                            description: tool.get("description")
                                .and_then(|d| d.as_str())
                                .unwrap_or("")
                                .to_string(),
                            input_schema: tool.get("inputSchema").cloned().unwrap_or(Value::Null),
                            server_name: config.name.clone(),
                        })
                    }).collect())
                } else {
                    Ok(vec![])
                }
            }
            Err(_) => Ok(vec![]),
        }
    }

    /// Enumerate resources from an MCP server via stdio
    async fn enumerate_resources(&self, config: &McpServerConfig) -> Result<Vec<McpResource>> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "resources/list",
            "params": {}
        });

        match self.send_mcp_request(config, &request).await {
            Ok(response) => {
                if let Some(resources) = response.get("result")
                    .and_then(|r| r.get("resources"))
                    .and_then(|r| r.as_array())
                {
                    Ok(resources.iter().filter_map(|res| {
                        Some(McpResource {
                            uri: res.get("uri")?.as_str()?.to_string(),
                            name: res.get("name")?.as_str()?.to_string(),
                            mime_type: res.get("mimeType").and_then(|m| m.as_str()).map(String::from),
                        })
                    }).collect())
                } else {
                    Ok(vec![])
                }
            }
            Err(_) => Ok(vec![]),
        }
    }

    /// Send a JSON-RPC request to an MCP server via stdio
    async fn send_mcp_request(
        &self,
        config: &McpServerConfig,
        request: &Value,
    ) -> Result<Value> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::process::Command;
        use tokio::time::{timeout, Duration};

        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args);
        for (key, value) in &config.env {
            cmd.env(key, value);
        }
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| anyhow!("Failed to spawn MCP server process: {}", e))?;

        let stdin = child.stdin.take().ok_or_else(|| anyhow!("Failed to open stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow!("Failed to open stdout"))?;

        let mut stdin_writer = tokio::io::BufWriter::new(stdin);
        let mut request_str = serde_json::to_string(request)?;
        request_str.push('\n');

        stdin_writer.write_all(request_str.as_bytes()).await?;
        stdin_writer.flush().await?;

        let mut reader = BufReader::new(stdout);
        let mut response_line = String::new();

        match timeout(Duration::from_secs(10), reader.read_line(&mut response_line)).await {
            Ok(Ok(_)) => {},
            Ok(Err(e)) => {
                let _ = child.kill().await;
                return Err(anyhow!("Failed to read MCP response: {}", e));
            }
            Err(_) => {
                let _ = child.kill().await;
                return Err(anyhow!("MCP server timed out"));
            }
        }

        let _ = child.kill().await;

        serde_json::from_str(&response_line)
            .map_err(|e| anyhow!("Failed to parse MCP response: {}", e))
    }
}