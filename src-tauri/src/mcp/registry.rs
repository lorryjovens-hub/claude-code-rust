use crate::mcp::discovery::McpDiscovery;
use crate::mcp::types::*;
use crate::tools::ToolDefinition;
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Semaphore};
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub enum McpRegistryEvent {
    ServerDiscovered(McpServerInfo),
    ServerConnected(String),
    ServerDisconnected(String),
    ServerError { id: String, error: String },
    ToolRegistered { server_id: String, tool_count: usize },
}

pub struct McpRegistry {
    servers: DashMap<String, McpServerInfo>,
    connections: DashMap<String, Arc<McpClientConnection>>,
    connection_semaphore: Arc<Semaphore>,
    discovery: McpDiscovery,
    event_tx: tokio::sync::broadcast::Sender<McpRegistryEvent>,
    max_connections: usize,
}

impl McpRegistry {
    pub fn new(max_connections: usize) -> Self {
        let (event_tx, _) = tokio::sync::broadcast::channel(1000);

        Self {
            servers: DashMap::new(),
            connections: DashMap::new(),
            connection_semaphore: Arc::new(Semaphore::new(max_connections)),
            discovery: McpDiscovery::new(),
            event_tx,
            max_connections,
        }
    }

    pub async fn discover(&self) -> Vec<McpServerInfo> {
        let servers = self.discovery.discover_all().await;

        for server in &servers {
            self.servers.insert(server.id.clone(), server.clone());
            let _ = self.event_tx.send(McpRegistryEvent::ServerDiscovered(server.clone()));
        }

        info!(target: "mcp_registry", "MCP discovery complete: {} servers", servers.len());
        servers
    }

    pub async fn connect(
        &self,
        server_id: &str,
    ) -> Result<Arc<McpClientConnection>, String> {
        if let Some(conn) = self.connections.get(server_id) {
            return Ok(conn.clone());
        }

        let server_info = self.servers
            .get(server_id)
            .ok_or_else(|| format!("Server not found: {}", server_id))?
            .clone();

        let _permit = self.connection_semaphore
            .acquire()
            .await
            .map_err(|_| "Too many connections".to_string())?;

        let connection = McpClientConnection::connect(server_info.clone()).await?;
        let connection = Arc::new(connection);

        self.connections.insert(server_id.to_string(), connection.clone());

        let _ = self.event_tx.send(McpRegistryEvent::ServerConnected(server_id.to_string()));

        info!(target: "mcp_registry", "MCP server connected: {} ({})", server_info.name, server_id);

        Ok(connection)
    }

    pub async fn connect_all(&self, tags: &[&str]) -> Vec<Result<Arc<McpClientConnection>, String>> {
        let mut results = Vec::new();

        for entry in self.servers.iter() {
            let server = entry.value();
            let should_connect = tags.is_empty() || tags.iter().any(|tag| {
                server.metadata.get("source").map(|s| s == *tag).unwrap_or(false)
                    || server.name.contains(tag)
            });

            if should_connect {
                match self.connect(&server.id).await {
                    Ok(conn) => results.push(Ok(conn)),
                    Err(e) => {
                        warn!(target: "mcp_registry", "Failed to connect MCP server {}: {}", server.name, e);
                        results.push(Err(e));
                    }
                }
            }
        }

        results
    }

    pub fn disconnect(&self, server_id: &str) {
        self.connections.remove(server_id);

        let _ = self.event_tx.send(McpRegistryEvent::ServerDisconnected(server_id.to_string()));

        self.connection_semaphore.add_permits(1);
    }

    pub fn register_server(&self, info: McpServerInfo) {
        self.servers.insert(info.id.clone(), info);
    }

    pub async fn get_all_mcp_tools(&self) -> Vec<ToolDefinition> {
        let mut all_tools = Vec::new();

        for entry in self.connections.iter() {
            let conn = entry.value();
            match conn.to_tool_definitions().await {
                Ok(tools) => {
                    let _ = self.event_tx.send(McpRegistryEvent::ToolRegistered {
                        server_id: entry.key().clone(),
                        tool_count: tools.len(),
                    });
                    all_tools.extend(tools);
                }
                Err(e) => {
                    warn!(target: "mcp_registry", "Failed to get MCP tools from {}: {}", entry.key(), e);
                }
            }
        }

        all_tools
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<McpRegistryEvent> {
        self.event_tx.subscribe()
    }

    pub async fn health_check(&self) -> McpHealthSummary {
        let total = self.servers.len();
        let mut connected = 0usize;
        let mut errored = 0usize;
        let mut details = Vec::new();

        for entry in self.connections.iter() {
            let conn = entry.value();
            match conn.ping().await {
                Ok(latency) => {
                    connected += 1;
                    details.push(ServerHealth {
                        id: entry.key().clone(),
                        status: "healthy".into(),
                        latency_ms: latency.as_millis() as u64,
                    });
                }
                Err(_) => {
                    errored += 1;
                    details.push(ServerHealth {
                        id: entry.key().clone(),
                        status: "error".into(),
                        latency_ms: 0,
                    });
                }
            }
        }

        let disconnected = total.saturating_sub(connected + errored);

        McpHealthSummary {
            total,
            connected,
            disconnected,
            errored,
            details,
        }
    }

    pub fn server_count(&self) -> usize {
        self.servers.len()
    }

    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }
}

#[allow(dead_code)]
struct ConnectionState {
    status: McpHealth,
    connected_at: Option<u64>,
    retry_count: u32,
    last_error: Option<String>,
}

struct RetryPolicy {
    max_retries: u32,
    base_delay: Duration,
    max_delay: Duration,
}

pub struct McpClientConnection {
    server_info: McpServerInfo,
    client: Arc<tokio::sync::Mutex<Option<McpTransportClient>>>,
    tool_cache: Arc<tokio::sync::Mutex<lru::LruCache<String, Vec<(String, String)>>>>,
    state: Arc<RwLock<ConnectionState>>,
    retry_policy: RetryPolicy,
    heartbeat_interval: Duration,
    event_tx: tokio::sync::mpsc::Sender<McpEvent>,
}

#[derive(Clone)]
enum McpTransportClient {
    Stdio {
        stdin: Arc<tokio::sync::Mutex<tokio::process::ChildStdin>>,
        stdout: Arc<tokio::sync::Mutex<tokio::io::BufReader<tokio::process::ChildStdout>>>,
        child: Arc<tokio::sync::Mutex<tokio::process::Child>>,
        request_id: Arc<tokio::sync::Mutex<u64>>,
    },
    Sse {
        url: String,
        headers: HashMap<String, String>,
    },
    WebSocket {
        url: String,
        headers: HashMap<String, String>,
    },
}

impl McpClientConnection {
    pub async fn connect(info: McpServerInfo) -> Result<Self, String> {
        let (event_tx, event_rx) = tokio::sync::mpsc::channel(1000);

        let mut connection = Self {
            server_info: info.clone(),
            client: Arc::new(tokio::sync::Mutex::new(None)),
            tool_cache: Arc::new(tokio::sync::Mutex::new(
                lru::LruCache::new(std::num::NonZeroUsize::new(100).unwrap()),
            )),
            state: Arc::new(RwLock::new(ConnectionState {
                status: McpHealth::Connecting,
                connected_at: None,
                retry_count: 0,
                last_error: None,
            })),
            retry_policy: RetryPolicy {
                max_retries: 3,
                base_delay: Duration::from_secs(1),
                max_delay: Duration::from_secs(30),
            },
            heartbeat_interval: Duration::from_secs(30),
            event_tx,
        };

        connection.establish_connection().await?;

        Ok(connection)
    }

    async fn establish_connection(&mut self) -> Result<(), String> {
        let client = match &self.server_info.transport {
            McpTransport::Stdio { command, args, env } => {
                self.connect_stdio(command, args, env).await?
            }
            McpTransport::Sse { url, headers } => {
                self.connect_sse(url, headers).await?
            }
            McpTransport::WebSocket { url, headers } => {
                self.connect_websocket(url, headers).await?
            }
            McpTransport::Embedded { .. } => {
                return Err("Embedded transport not yet implemented".into());
            }
        };

        {
            let mut state = self.state.write().await;
            state.status = McpHealth::Connected;
            state.connected_at = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
            state.retry_count = 0;
        }

        *self.client.lock().await = Some(client);

        let _ = self.event_tx.send(McpEvent::Connected {
            server_id: self.server_info.id.clone(),
        }).await;

        self.start_heartbeat();

        Ok(())
    }

    async fn connect_stdio(
        &self,
        command: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<McpTransportClient, String> {
        let mut cmd = tokio::process::Command::new(command);
        cmd.args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        for (k, v) in env {
            cmd.env(k, v);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn process: {}", e))?;

        let stdin = child.stdin.take()
            .ok_or_else(|| "Failed to take stdin".to_string())?;
        let stdout = child.stdout.take()
            .ok_or_else(|| "Failed to take stdout".to_string())?;

        let request_id = Arc::new(tokio::sync::Mutex::new(0u64));

        Ok(McpTransportClient::Stdio {
            stdin: Arc::new(tokio::sync::Mutex::new(stdin)),
            stdout: Arc::new(tokio::sync::Mutex::new(tokio::io::BufReader::new(stdout))),
            child: Arc::new(tokio::sync::Mutex::new(child)),
            request_id,
        })
    }

    async fn connect_sse(
        &self,
        _url: &str,
        _headers: &HashMap<String, String>,
    ) -> Result<McpTransportClient, String> {
        Ok(McpTransportClient::Sse {
            url: _url.to_string(),
            headers: _headers.clone(),
        })
    }

    async fn connect_websocket(
        &self,
        url_str: &str,
        _headers: &std::collections::HashMap<String, String>,
    ) -> Result<McpTransportClient, String> {
        let (_ws_stream, _) = tokio_tungstenite::connect_async(url_str)
            .await
            .map_err(|e| format!("WebSocket failed: {}", e))?;

        Ok(McpTransportClient::WebSocket {
            url: url_str.to_string(),
            headers: _headers.clone(),
        })
    }

    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<McpToolCallResult, String> {
        let start = std::time::Instant::now();

        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref()
            .ok_or_else(|| "Not connected".to_string())?;

        let result = self.invoke_tool_via_transport(client, tool_name, &arguments).await?;

        let duration = start.elapsed();

        Ok(McpToolCallResult {
            content: result,
            is_error: false,
            duration,
            metadata: HashMap::new(),
        })
    }

    async fn invoke_tool_via_transport(
        &self,
        client: &McpTransportClient,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> Result<Vec<ContentItem>, String> {
        match client {
            McpTransportClient::Stdio { stdin, stdout, child: _, request_id } => {
                let id = {
                    let mut rid = request_id.lock().await;
                    *rid += 1;
                    *rid
                };

                let request = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "method": "tools/call",
                    "params": {
                        "name": tool_name,
                        "arguments": arguments
                    }
                });

                let request_str = format!("{}\n", serde_json::to_string(&request).unwrap());

                {
                    let mut stdin_guard = stdin.lock().await;
                    use tokio::io::AsyncWriteExt;
                    stdin_guard.write_all(request_str.as_bytes()).await
                        .map_err(|e| format!("Write failed: {}", e))?;
                }

                let mut stdout_guard = stdout.lock().await;
                use tokio::io::AsyncBufReadExt;
                let mut line = String::new();
                tokio::time::timeout(
                    Duration::from_secs(30),
                    stdout_guard.read_line(&mut line),
                )
                .await
                .map_err(|_| "Timeout".to_string())?
                .map_err(|e| format!("Read failed: {}", e))?;

                let response: serde_json::Value = serde_json::from_str(line.trim())
                    .map_err(|e| format!("Parse failed: {}", e))?;

                let content = response
                    .get("result")
                    .and_then(|r| r.get("content"))
                    .and_then(|c| c.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|item| {
                                let item_type = item.get("type")?.as_str()?;
                                match item_type {
                                    "text" => Some(ContentItem::Text {
                                        text: item.get("text")?.as_str()?.to_string(),
                                    }),
                                    "image" => Some(ContentItem::Image {
                                        data: item.get("data")?.as_str()?.to_string(),
                                        mime_type: item.get("mimeType")?.as_str()?.to_string(),
                                    }),
                                    "resource" => Some(ContentItem::Resource {
                                        resource: ResourceContent {
                                            uri: item.get("resource")?.get("uri")?.as_str()?.to_string(),
                                            mime_type: None,
                                            text: item.get("resource")?.get("text")?.as_str().map(String::from),
                                        },
                                    }),
                                    _ => None,
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                Ok(content)
            }
            McpTransportClient::Sse { url, headers } => {
                let client = reqwest::Client::new();
                let mut req = client
                    .post(url)
                    .header("Content-Type", "application/json");

                for (k, v) in headers {
                    req = req.header(k, v);
                }

                let body = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "tools/call",
                    "params": {
                        "name": tool_name,
                        "arguments": arguments
                    }
                });

                let response = req
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| format!("SSE request failed: {}", e))?;

                let result: serde_json::Value = response
                    .json()
                    .await
                    .map_err(|e| format!("SSE parse failed: {}", e))?;

                let content = result
                    .get("result")
                    .and_then(|r| r.get("content"))
                    .map(|c| vec![ContentItem::Text {
                        text: c.to_string(),
                    }])
                    .unwrap_or_default();

                Ok(content)
            }
            McpTransportClient::WebSocket { url, headers: _ } => {
                let (ws_stream, _) = tokio_tungstenite::connect_async(url)
                    .await
                    .map_err(|e| format!("WS connect failed: {}", e))?;

                let (mut write, mut read): (
                    futures::stream::SplitSink<_, _>,
                    futures::stream::SplitStream<_>,
                ) = ws_stream.split();

                let request = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "tools/call",
                    "params": {
                        "name": tool_name,
                        "arguments": arguments
                    }
                });

                write
                    .send(tokio_tungstenite::tungstenite::Message::Text(
                        request.to_string(),
                    ))
                    .await
                    .map_err(|e| format!("WS send failed: {}", e))?;

                let response_text = tokio::time::timeout(Duration::from_secs(30), async {
                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                                return Ok(text);
                            }
                            Ok(_) => continue,
                            Err(e) => return Err(format!("WS error: {}", e)),
                        }
                    }
                    Err("WS closed".into())
                })
                .await
                .map_err(|_| "WS timeout".to_string())??;

                let result: serde_json::Value = serde_json::from_str(&response_text)
                    .map_err(|e| format!("WS parse failed: {}", e))?;

                let content = result
                    .get("result")
                    .and_then(|r| r.get("content"))
                    .map(|c| vec![ContentItem::Text {
                        text: c.to_string(),
                    }])
                    .unwrap_or_default();

                Ok(content)
            }
        }
    }

    pub async fn list_tools(&self) -> Result<Vec<(String, String)>, String> {
        {
            let mut cache = self.tool_cache.lock().await;
            if let Some(tools) = cache.get(&self.server_info.id) {
                return Ok(tools.clone());
            }
        }

        let client_guard = self.client.lock().await;
        let client = client_guard.as_ref().ok_or_else(|| "Not connected".to_string())?;

        let tools = match client {
            McpTransportClient::Stdio { stdin, stdout, child: _, request_id } => {
                let id = {
                    let mut rid = request_id.lock().await;
                    *rid += 1;
                    *rid
                };

                let request = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "method": "tools/list",
                    "params": {}
                });

                let request_str = format!("{}\n", serde_json::to_string(&request).unwrap());

                {
                    let mut stdin_guard = stdin.lock().await;
                    use tokio::io::AsyncWriteExt;
                    stdin_guard.write_all(request_str.as_bytes()).await
                        .map_err(|e| format!("Write failed: {}", e))?;
                }

                let mut stdout_guard = stdout.lock().await;
                use tokio::io::AsyncBufReadExt;
                let mut line = String::new();
                tokio::time::timeout(
                    Duration::from_secs(30),
                    stdout_guard.read_line(&mut line),
                )
                .await
                .map_err(|_| "Timeout".to_string())?
                .map_err(|e| format!("Read failed: {}", e))?;

                let response: serde_json::Value = serde_json::from_str(line.trim())
                    .map_err(|e| format!("Parse failed: {}", e))?;

                response
                    .get("result")
                    .and_then(|r| r.get("tools"))
                    .and_then(|t| t.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|tool| {
                                Some((
                                    tool.get("name")?.as_str()?.to_string(),
                                    tool.get("description")
                                        .and_then(|d| d.as_str())
                                        .unwrap_or("")
                                        .to_string(),
                                ))
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            }
            McpTransportClient::Sse { .. } => vec![],
            McpTransportClient::WebSocket { .. } => vec![],
        };

        {
            let mut cache = self.tool_cache.lock().await;
            cache.put(self.server_info.id.clone(), tools.clone());
        }

        Ok(tools)
    }

    pub async fn ping(&self) -> Result<Duration, String> {
        let start = std::time::Instant::now();
        let _ = self.list_tools().await?;
        Ok(start.elapsed())
    }

    pub async fn to_tool_definitions(&self) -> Result<Vec<ToolDefinition>, String> {
        let tools = self.list_tools().await?;
        let server_id = self.server_info.id.clone();

        let definitions: Vec<ToolDefinition> = tools
            .into_iter()
            .map(|(name, description)| ToolDefinition {
                name: format!("mcp_{}:{}", server_id, name),
                description: format!("[MCP:{}] {}", self.server_info.name, description),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "arguments": {
                            "type": "object",
                            "description": "Tool arguments"
                        }
                    }
                }),
            })
            .collect();

        Ok(definitions)
    }

    pub async fn reconnect(&mut self) -> Result<(), String> {
        let mut state = self.state.write().await;

        if state.retry_count >= self.retry_policy.max_retries {
            state.status = McpHealth::Error {
                message: format!("Max retries exceeded ({})", self.retry_policy.max_retries),
                retry_at: None,
            };
            return Err("Max retries exceeded".into());
        }

        let delay = self.retry_policy.base_delay * 2u32.pow(state.retry_count);
        let delay = delay.min(self.retry_policy.max_delay);

        tokio::time::sleep(delay).await;

        state.retry_count += 1;
        state.status = McpHealth::Connecting;
        drop(state);

        *self.client.lock().await = None;

        self.establish_connection().await
    }

    fn start_heartbeat(&self) {
        let client = self.client.clone();
        let state = self.state.clone();
        let interval = self.heartbeat_interval;
        let server_id = self.server_info.id.clone();
        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;

                let client_guard = client.lock().await;
                if client_guard.is_some() {
                    let mut s = state.write().await;
                    let elapsed = s.connected_at
                        .map(|at| {
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs();
                            now.saturating_sub(at)
                        })
                        .unwrap_or(0);

                    if elapsed > 120 {
                        s.status = McpHealth::Disconnected;
                        let _ = event_tx.send(McpEvent::Disconnected {
                            server_id: server_id.clone(),
                            reason: "Heartbeat timeout".into(),
                        }).await;
                        break;
                    }
                } else {
                    break;
                }
            }
        });
    }
}