use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, RwLock};

pub mod tool_executor;
pub mod composio;
pub mod mcp_client;
pub mod types;
pub mod discovery;
pub mod registry;
pub mod bridge;
pub mod pool;

pub use tool_executor::McpToolRegistry;
pub use mcp_client::McpClientManager;
pub use registry::{McpRegistry, McpRegistryEvent, McpClientConnection};
pub use bridge::McpToolBridge;
pub use pool::McpConnectionPool;
pub use discovery::McpDiscovery;
pub use types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub server_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourceContent {
    pub uri: String,
    pub content: String,
    pub content_type: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourceChange {
    pub uri: String,
    pub change_type: String,
    pub timestamp: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerStatus {
    pub id: String,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
    pub enabled: bool,
    pub running: bool,
    pub pid: Option<u32>,
    pub tools_count: usize,
    pub resources_count: usize,
    pub error: Option<String>,
    pub transport_type: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct McpServerState {
    #[serde(skip)]
    pub connector: Option<Arc<Mutex<McpConnector>>>,
    pub config: McpServerConfig,
    pub status: McpServerStatus,
}

impl std::fmt::Debug for McpServerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpServerState")
            .field("config", &self.config)
            .field("status", &self.status)
            .finish()
    }
}

pub struct McpServerManager {
    servers: Arc<RwLock<HashMap<String, McpServerState>>>,
    config_path: PathBuf,
}

impl McpServerManager {
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            config_path,
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        if self.config_path.exists() {
            self.load_config().await?;
        } else {
            self.create_default_config().await?;
        }
        Ok(())
    }

    async fn load_config(&self) -> Result<()> {
        let content = tokio::fs::read_to_string(&self.config_path).await?;
        let config: serde_json::Value = serde_json::from_str(&content)?;

        if let Some(servers) = config.get("mcpServers").and_then(|s| s.as_object()) {
            for (id, value) in servers {
                let server_config = McpServerConfig {
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
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|s| s.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default(),
                    env: value.get("env")
                        .and_then(|e| e.as_object())
                        .map(|obj| {
                            obj.iter()
                                .filter_map(|(k, v)| {
                                    v.as_str().map(|s| (k.clone(), s.to_string()))
                                })
                                .collect()
                        })
                        .unwrap_or_default(),
                    enabled: value.get("enabled")
                        .and_then(|e| e.as_bool())
                        .unwrap_or(true),
                };

                let status = McpServerStatus {
                    id: server_config.id.clone(),
                    name: server_config.name.clone(),
                    command: server_config.command.clone(),
                    args: server_config.args.clone(),
                    env: if server_config.env.is_empty() { None } else { Some(server_config.env.clone()) },
                    enabled: server_config.enabled,
                    running: false,
                    pid: None,
                    tools_count: 0,
                    resources_count: 0,
                    error: None,
                    transport_type: "stdio".to_string(),
                };

                let state = McpServerState {
                    config: server_config,
                    connector: None,
                    status,
                };

                self.servers.write().await.insert(id.clone(), state);
            }
        }

        Ok(())
    }

    async fn create_default_config(&self) -> Result<()> {
        let default_servers = vec![
            McpServerConfig {
                id: "filesystem".to_string(),
                name: "Filesystem (Anthropic)".to_string(),
                command: "npx".to_string(),
                args: vec![
                    "-y".to_string(),
                    "@anthropic/mcp-server-filesystem".to_string(),
                    ".".to_string(),
                ],
                env: HashMap::new(),
                enabled: false,
            },
            McpServerConfig {
                id: "git".to_string(),
                name: "Git".to_string(),
                command: "npx".to_string(),
                args: vec![
                    "-y".to_string(),
                    "@anthropic/mcp-server-git".to_string(),
                ],
                env: HashMap::new(),
                enabled: false,
            },
            McpServerConfig {
                id: "brave-search".to_string(),
                name: "Brave Search".to_string(),
                command: "npx".to_string(),
                args: vec![
                    "-y".to_string(),
                    "@anthropic/mcp-server-brave-search".to_string(),
                ],
                env: HashMap::new(),
                enabled: false,
            },
            McpServerConfig {
                id: "mcp-filesystem".to_string(),
                name: "Filesystem (MCP)".to_string(),
                command: "npx".to_string(),
                args: vec![
                    "-y".to_string(),
                    "@modelcontextprotocol/server-filesystem".to_string(),
                    ".".to_string(),
                ],
                env: HashMap::new(),
                enabled: false,
            },
            McpServerConfig {
                id: "github".to_string(),
                name: "GitHub".to_string(),
                command: "npx".to_string(),
                args: vec![
                    "-y".to_string(),
                    "@modelcontextprotocol/server-github".to_string(),
                ],
                env: HashMap::new(),
                enabled: false,
            },
            McpServerConfig {
                id: "slack".to_string(),
                name: "Slack".to_string(),
                command: "npx".to_string(),
                args: vec![
                    "-y".to_string(),
                    "@modelcontextprotocol/server-slack".to_string(),
                ],
                env: HashMap::new(),
                enabled: false,
            },
        ];

        let mut servers = HashMap::new();
        for server in default_servers {
            let status = McpServerStatus {
                id: server.id.clone(),
                name: server.name.clone(),
                command: server.command.clone(),
                args: server.args.clone(),
                env: if server.env.is_empty() { None } else { Some(server.env.clone()) },
                enabled: server.enabled,
                running: false,
                pid: None,
                tools_count: 0,
                resources_count: 0,
                error: None,
                transport_type: "stdio".to_string(),
            };

            servers.insert(server.id.clone(), McpServerState {
                config: server,
                connector: None,
                status,
            });
        }

        *self.servers.write().await = servers;
        self.save_config().await?;

        Ok(())
    }

    async fn save_config(&self) -> Result<()> {
        let servers_map = self.servers.read().await;
        let mut mcp_servers = serde_json::Map::new();

        for (id, state) in servers_map.iter() {
            let mut server_obj = serde_json::Map::new();
            server_obj.insert("name".to_string(), serde_json::json!(state.config.name));
            server_obj.insert("command".to_string(), serde_json::json!(state.config.command));
            server_obj.insert("args".to_string(), serde_json::json!(state.config.args));
            server_obj.insert("env".to_string(), serde_json::json!(state.config.env));
            server_obj.insert("enabled".to_string(), serde_json::json!(state.config.enabled));
            mcp_servers.insert(id.clone(), serde_json::json!(server_obj));
        }

        let config = serde_json::json!({
            "mcpServers": mcp_servers
        });

        if let Some(parent) = self.config_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&self.config_path, serde_json::to_string_pretty(&config)?).await?;
        Ok(())
    }

    pub async fn add_server(&self, config: McpServerConfig) -> Result<()> {
        let status = McpServerStatus {
            id: config.id.clone(),
            name: config.name.clone(),
            command: config.command.clone(),
            args: config.args.clone(),
            env: if config.env.is_empty() { None } else { Some(config.env.clone()) },
            enabled: config.enabled,
            running: false,
            pid: None,
            tools_count: 0,
            resources_count: 0,
            error: None,
            transport_type: "stdio".to_string(),
        };

        let state = McpServerState {
            config,
            connector: None,
            status,
        };

        self.servers.write().await.insert(state.config.id.clone(), state);
        self.save_config().await?;
        Ok(())
    }

    pub async fn update_server(&self, id: &str, config: McpServerConfig) -> Result<()> {
        let mut servers = self.servers.write().await;
        if let Some(state) = servers.get_mut(id) {
            let was_running = state.status.running;
            state.config = config;
            state.status.name = state.config.name.clone();
            state.status.command = state.config.command.clone();
            state.status.args = state.config.args.clone();
            state.status.env = if state.config.env.is_empty() { None } else { Some(state.config.env.clone()) };
            state.status.enabled = state.config.enabled;

            if was_running {
                if let Some(connector) = state.connector.take() {
                    let mut conn = connector.lock().await;
                    let _ = conn.stop().await;
                }
                state.status.running = false;
                state.status.pid = None;
            }
        } else {
            return Err(anyhow!("Server not found: {}", id));
        }

        self.save_config().await?;
        Ok(())
    }

    pub async fn remove_server(&self, id: &str) -> Result<()> {
        self.stop_server(id).await?;
        self.servers.write().await.remove(id);
        self.save_config().await?;
        Ok(())
    }

    pub async fn list_servers(&self) -> Vec<McpServerStatus> {
        let servers = self.servers.read().await;
        servers.values().map(|s| s.status.clone()).collect()
    }

    pub async fn list_servers_full(&self) -> Vec<McpServerState> {
        let servers = self.servers.read().await;
        servers.values().cloned().collect()
    }

    pub async fn get_server(&self, id: &str) -> Option<McpServerStatus> {
        self.servers.read().await.get(id).map(|s| s.status.clone())
    }

    pub async fn start_server(&self, id: &str) -> Result<()> {
        let config = {
            let servers = self.servers.read().await;
            servers.get(id).map(|s| s.config.clone())
        };

        let config = match config {
            Some(c) => c,
            None => return Err(anyhow!("Server not found: {}", id)),
        };

        if !config.enabled {
            return Err(anyhow!("Server is disabled: {}", id));
        }

        let mut connector = McpConnector::new(config.clone());
        connector.start().await?;

        let tools = connector.list_tools().await.unwrap_or_default();
        let resources = connector.list_resources().await.unwrap_or_default();

        let connector = Arc::new(Mutex::new(connector));

        let mut servers = self.servers.write().await;
        if let Some(state) = servers.get_mut(id) {
            state.connector = Some(connector);
            state.status.running = true;
            state.status.pid = None;
            state.status.tools_count = tools.len();
            state.status.resources_count = resources.len();
            state.status.error = None;
        }

        Ok(())
    }

    pub async fn stop_server(&self, id: &str) -> Result<()> {
        let connector = {
            let mut servers = self.servers.write().await;
            if let Some(state) = servers.get_mut(id) {
                state.connector.take()
            } else {
                return Err(anyhow!("Server not found: {}", id));
            }
        };

        if let Some(connector) = connector {
            let mut conn = connector.lock().await;
            conn.stop().await?;
        }

        let mut servers = self.servers.write().await;
        if let Some(state) = servers.get_mut(id) {
            state.status.running = false;
            state.status.pid = None;
            state.status.tools_count = 0;
            state.status.resources_count = 0;
        }

        Ok(())
    }

    pub async fn restart_server(&self, id: &str) -> Result<()> {
        self.stop_server(id).await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        self.start_server(id).await?;
        Ok(())
    }

    pub async fn get_all_tools(&self) -> Vec<McpTool> {
        let servers = self.servers.read().await;
        let mut all_tools = Vec::new();

        for state in servers.values() {
            if state.status.running {
                if let Some(connector) = &state.connector {
                    let conn = connector.lock().await;
                    if let Ok(tools) = conn.list_tools().await {
                        for tool in tools {
                            all_tools.push(McpTool {
                                server_name: state.config.name.clone(),
                                ..tool
                            });
                        }
                    }
                }
            }
        }

        all_tools
    }

    pub async fn get_all_resources(&self) -> Vec<McpResource> {
        let servers = self.servers.read().await;
        let mut all_resources = Vec::new();

        for state in servers.values() {
            if state.status.running {
                if let Some(connector) = &state.connector {
                    let conn = connector.lock().await;
                    if let Ok(resources) = conn.list_resources().await {
                        all_resources.extend(resources);
                    }
                }
            }
        }

        all_resources
    }

    pub async fn read_resource(&self, server_id: &str, uri: &str, options: Option<serde_json::Value>) -> Result<McpResourceContent> {
        let connector = {
            let servers = self.servers.read().await;
            if let Some(state) = servers.get(server_id) {
                if let Some(connector) = &state.connector {
                    Some(connector.clone())
                } else {
                    return Err(anyhow!("Server connector not available: {}", server_id));
                }
            } else {
                return Err(anyhow!("Server not found: {}", server_id));
            }
        };

        if let Some(connector) = connector {
            let conn = connector.lock().await;
            conn.read_resource(uri, options).await
        } else {
            Err(anyhow!("Server not running: {}", server_id))
        }
    }

    pub async fn monitor_resource(&self, server_id: &str, uri: &str, enabled: bool) -> Result<bool> {
        let connector = {
            let servers = self.servers.read().await;
            if let Some(state) = servers.get(server_id) {
                if let Some(connector) = &state.connector {
                    Some(connector.clone())
                } else {
                    return Err(anyhow!("Server connector not available: {}", server_id));
                }
            } else {
                return Err(anyhow!("Server not found: {}", server_id));
            }
        };

        if let Some(connector) = connector {
            let conn = connector.lock().await;
            conn.monitor_resource(uri, enabled).await
        } else {
            Err(anyhow!("Server not running: {}", server_id))
        }
    }

    pub async fn call_tool(&self, server_id: &str, tool_name: &str, arguments: serde_json::Value) -> Result<serde_json::Value> {
        let connector = {
            let servers = self.servers.read().await;
            if let Some(state) = servers.get(server_id) {
                if let Some(connector) = &state.connector {
                    Some(connector.clone())
                } else {
                    return Err(anyhow!("Server connector not available: {}", server_id));
                }
            } else {
                return Err(anyhow!("Server not found: {}", server_id));
            }
        };

        if let Some(connector) = connector {
            let conn = connector.lock().await;
            conn.call_tool(tool_name, arguments).await
        } else {
            Err(anyhow!("Server not running: {}", server_id))
        }
    }

    pub async fn toggle_server(&self, id: &str) -> Result<()> {
        let enabled = {
            let servers = self.servers.read().await;
            if let Some(state) = servers.get(id) {
                state.config.enabled
            } else {
                return Err(anyhow!("Server not found: {}", id));
            }
        };

        let mut servers = self.servers.write().await;
        if let Some(state) = servers.get_mut(id) {
            state.config.enabled = !enabled;
            state.status.enabled = !enabled;
        }

        self.save_config().await?;
        Ok(())
    }

    pub async fn set_server_enabled(&self, id: &str, enabled: bool) -> Result<()> {
        let mut servers = self.servers.write().await;
        if let Some(state) = servers.get_mut(id) {
            state.config.enabled = enabled;
            state.status.enabled = enabled;
        } else {
            return Err(anyhow!("Server not found: {}", id));
        }

        self.save_config().await?;
        Ok(())
    }
}

impl Default for McpServerManager {
    fn default() -> Self {
        Self::new(PathBuf::from("mcp-servers.json"))
    }
}

impl McpServerManager {
    pub async fn shutdown_all(&self) -> Result<()> {
        let server_ids: Vec<String> = {
            let servers = self.servers.read().await;
            servers.keys()
                .filter(|id| servers.get(*id).map(|s| s.status.running).unwrap_or(false))
                .cloned()
                .collect()
        };

        for id in server_ids {
            tracing::info!(module = "MCP", "Shutting down server: {}", id);
            let _ = self.stop_server(&id).await;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 10000,
            backoff_factor: 2.0,
        }
    }
}

pub struct McpConnector {
    config: McpServerConfig,
    process: Option<Child>,
    request_id: Mutex<u64>,
    stdin: Mutex<Option<tokio::process::ChildStdin>>,
    stdout: Mutex<Option<tokio::process::ChildStdout>>,
    retry_config: RetryConfig,
    last_heartbeat: Mutex<std::time::Instant>,
    is_healthy: Mutex<bool>,
}

impl McpConnector {
    pub fn new(config: McpServerConfig) -> Self {
        Self {
            config,
            process: None,
            request_id: Mutex::new(0),
            stdin: Mutex::new(None),
            stdout: Mutex::new(None),
            retry_config: RetryConfig::default(),
            last_heartbeat: Mutex::new(std::time::Instant::now()),
            is_healthy: Mutex::new(false),
        }
    }

    pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = retry_config;
        self
    }

    pub async fn start(&mut self) -> Result<()> {
        let mut cmd = Command::new(&self.config.command);
        cmd.args(&self.config.args)
            .envs(&self.config.env)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd.spawn()?;

        let stdin = child.stdin.take().ok_or_else(|| anyhow!("Failed to take stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow!("Failed to take stdout"))?;

        self.stdin = Mutex::new(Some(stdin));
        self.stdout = Mutex::new(Some(stdout));
        self.process = Some(child);

        self.initialize().await?;

        Ok(())
    }

    async fn initialize(&self) -> Result<()> {
        let request = McpJsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "roots": {"listChanged": true},
                    "sampling": {}
                },
                "clientInfo": {
                    "name": "claude-desktop-tauri",
                    "version": "1.6.12"
                }
            })),
        };

        self.send_json_rpc(request).await?;
        Ok(())
    }

    pub async fn list_tools(&self) -> Result<Vec<McpTool>> {
        let request = McpJsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_id().await,
            method: "tools/list".to_string(),
            params: None,
        };

        let response = self.send_json_rpc(request).await?;

        let tools = response
            .result
            .as_ref()
            .and_then(|r| r.get("tools"))
            .and_then(|t| t.as_array())
            .map(|arr| {
                arr.iter().filter_map(|t| {
                    Some(McpTool {
                        name: t.get("name")?.as_str()?.to_string(),
                        description: t.get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("")
                            .to_string(),
                        input_schema: t.get("inputSchema").cloned().unwrap_or(serde_json::json!({})),
                        server_name: String::new(),
                    })
                }).collect()
            })
            .unwrap_or_default();

        Ok(tools)
    }

    pub async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> Result<serde_json::Value> {
        let request = McpJsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_id().await,
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": name,
                "arguments": arguments
            })),
        };

        let response = self.send_json_rpc(request).await?;

        response.result.ok_or_else(|| anyhow!("No result in tool call response"))
    }

    pub async fn list_resources(&self) -> Result<Vec<McpResource>> {
        let request = McpJsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_id().await,
            method: "resources/list".to_string(),
            params: None,
        };

        let response = self.send_json_rpc(request).await?;

        let resources = response
            .result
            .as_ref()
            .and_then(|r| r.get("resources"))
            .and_then(|t| t.as_array())
            .map(|arr| {
                arr.iter().filter_map(|r| {
                    Some(McpResource {
                        uri: r.get("uri")?.as_str()?.to_string(),
                        name: r.get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("")
                            .to_string(),
                        mime_type: r.get("mimeType").and_then(|m| m.as_str()).map(String::from),
                    })
                }).collect()
            })
            .unwrap_or_default();

        Ok(resources)
    }

    pub async fn read_resource(&self, uri: &str, options: Option<serde_json::Value>) -> Result<McpResourceContent> {
        let params = match options {
            Some(opts) => serde_json::json!({
                "uri": uri,
                "options": opts
            }),
            None => serde_json::json!({
                "uri": uri
            }),
        };

        let request = McpJsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_id().await,
            method: "resources/read".to_string(),
            params: Some(params),
        };

        let response = self.send_json_rpc(request).await?;

        let result = response.result.ok_or_else(|| anyhow!("No result in read resource response"))?;

        let content = McpResourceContent {
            uri: result.get("uri").and_then(|u| u.as_str()).unwrap_or(uri).to_string(),
            content: result.get("content").and_then(|c| c.as_str()).unwrap_or("").to_string(),
            content_type: result.get("contentType").and_then(|ct| ct.as_str()).unwrap_or("text/plain").to_string(),
            metadata: result.get("metadata").cloned(),
        };

        Ok(content)
    }

    pub async fn monitor_resource(&self, uri: &str, enabled: bool) -> Result<bool> {
        let request = McpJsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_id().await,
            method: "resources/monitor".to_string(),
            params: Some(serde_json::json!({
                "uri": uri,
                "enabled": enabled
            })),
        };

        let response = self.send_json_rpc(request).await?;

        response.result
            .as_ref()
            .and_then(|r| r.get("enabled").and_then(|e| e.as_bool()))
            .ok_or_else(|| anyhow!("Invalid response from resources/monitor"))
    }

    async fn next_id(&self) -> u64 {
        let mut counter = self.request_id.lock().await;
        *counter += 1;
        *counter
    }

    async fn send_json_rpc(&self, request: McpJsonRpcRequest) -> Result<McpJsonRpcResponse> {
        let mut attempt = 0;
        let max_retries = self.retry_config.max_retries;

        loop {
            match self.try_send_json_rpc(&request).await {
                Ok(response) => {
                    *self.last_heartbeat.lock().await = std::time::Instant::now();
                    *self.is_healthy.lock().await = true;
                    return Ok(response);
                },
                Err(e) => {
                    if attempt < max_retries {
                        let delay_ms = self.calculate_backoff(attempt);
                        tracing::warn!(module = "MCP", "Retry attempt {}/{} for {}: {} (retrying in {}ms)",
                            attempt + 1,
                            max_retries,
                            request.method,
                            e,
                            delay_ms
                        );
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

                        // 尝试重新连接
                        self.attempt_reconnect().await.ok();

                        attempt += 1;
                    } else {
                        *self.is_healthy.lock().await = false;
                        return Err(e);
                    }
                }
            }
        }
    }

    async fn try_send_json_rpc(&self, request: &McpJsonRpcRequest) -> Result<McpJsonRpcResponse> {
        let json = serde_json::to_string(request)?;
        {
            let mut stdin_guard = self.stdin.lock().await;
            let stdin = stdin_guard.as_mut().ok_or_else(|| anyhow!("MCP stdin not available"))?;
            stdin.write_all(format!("{}\n", json).as_bytes()).await?;
        }

        let mut stdout_guard = self.stdout.lock().await;
        let stdout = stdout_guard.as_mut().ok_or_else(|| anyhow!("MCP stdout not available"))?;

        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        tokio::time::timeout(tokio::time::Duration::from_secs(30), reader.read_line(&mut line)).await??;

        let response: McpJsonRpcResponse = serde_json::from_str(&line.trim())?;

        if let Some(error) = &response.error {
            return Err(anyhow!("MCP error [{}]: {}", error.code, error.message));
        }

        Ok(response)
    }

    fn calculate_backoff(&self, attempt: u32) -> u64 {
        let delay = (self.retry_config.initial_delay_ms as f64) *
            (self.retry_config.backoff_factor.powi(attempt as i32));
        delay.min(self.retry_config.max_delay_ms as f64) as u64
    }

    async fn attempt_reconnect(&self) -> Result<()> {
        tracing::warn!(module = "MCP", "Attempting to reconnect to server: {}", self.config.name);

        // 检查进程是否存在
        if self.process.is_none() {
            return Err(anyhow!("No process to reconnect"));
        }

        // 进程仍在运行，尝试重新初始化
        tracing::warn!(module = "MCP", "Process is still running, attempting to reinitialize...");

        Ok(())
    }

    pub async fn check_health(&self) -> bool {
        let last_heartbeat = *self.last_heartbeat.lock().await;
        let elapsed = last_heartbeat.elapsed();

        if elapsed > std::time::Duration::from_secs(60) {
            // 心跳超时，执行健康检查
            if let Ok(response) = self.list_tools().await {
                !response.is_empty()
            } else {
                false
            }
        } else {
            *self.is_healthy.lock().await
        }
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(mut child) = self.process.take() {
            child.kill().await?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpJsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpJsonRpcResponse {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<u64>,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<McpJsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpJsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Default)]
pub struct McpConfigManager {
    pub servers: HashMap<String, McpServerConfig>,
}

impl McpConfigManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_server(&mut self, name: String, server: McpServerConfig) {
        self.servers.insert(name, server);
    }

    pub fn remove_server(&mut self, name: &str) -> Option<McpServerConfig> {
        self.servers.remove(name)
    }

    pub fn get_server(&self, name: &str) -> Option<&McpServerConfig> {
        self.servers.get(name)
    }

    pub fn list_servers(&self) -> Vec<&McpServerConfig> {
        self.servers.values().collect()
    }
}
