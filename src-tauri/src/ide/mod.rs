use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::watch;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeConnection {
    pub id: String,
    pub ide_type: IdeType,
    pub status: IdeStatus,
    pub workspace: Option<String>,
    pub connected_at: String,
    pub last_heartbeat: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IdeType {
    VSCode,
    Cursor,
    JetBrains,
    Neovim,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IdeStatus {
    Connected,
    Disconnected,
    Reconnecting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeMessage {
    pub id: String,
    pub msg_type: IdeMessageType,
    pub payload: serde_json::Value,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IdeMessageType {
    OpenFile,
    CloseFile,
    SaveFile,
    Diagnostics,
    CursorPosition,
    Selection,
    TerminalData,
    Command,
    Response,
    Heartbeat,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenFilePayload {
    pub path: String,
    pub content: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsPayload {
    pub file: String,
    pub diagnostics: Vec<DiagnosticItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticItem {
    pub severity: String,
    pub message: String,
    pub line: u32,
    pub column: u32,
    pub source: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPositionPayload {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPayload {
    pub command: String,
    pub args: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeConfig {
    pub port: u16,
    pub auto_connect: bool,
    pub extensions: Vec<ExtensionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionConfig {
    pub ide_type: IdeType,
    pub extension_id: String,
    pub version: String,
}

impl Default for IdeConfig {
    fn default() -> Self {
        Self {
            port: 9527,
            auto_connect: true,
            extensions: vec![
                ExtensionConfig {
                    ide_type: IdeType::VSCode,
                    extension_id: "claude-dev.tauri-bridge".to_string(),
                    version: "0.1.0".to_string(),
                },
            ],
        }
    }
}

pub struct IdeBridge {
    config: IdeConfig,
    connections: Arc<Mutex<HashMap<String, IdeConnection>>>,
    shutdown_senders: Arc<Mutex<HashMap<String, watch::Sender<bool>>>>,
    #[allow(dead_code)]
    message_handlers: Arc<Mutex<HashMap<String, Box<dyn Fn(IdeMessage) -> Option<IdeMessage> + Send + Sync>>>>,
    running: Arc<Mutex<bool>>,
}

impl IdeBridge {
    pub fn new(config: IdeConfig) -> Self {
        Self {
            config,
            connections: Arc::new(Mutex::new(HashMap::new())),
            shutdown_senders: Arc::new(Mutex::new(HashMap::new())),
            message_handlers: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn with_default() -> Self {
        Self::new(IdeConfig::default())
    }

    pub async fn start_server(&self) -> Result<u16> {
        let mut running = self.running.lock().await;
        if *running {
            return Ok(self.config.port);
        }

        let addr = format!("127.0.0.1:{}", self.config.port);
        let listener = TcpListener::bind(&addr).await
            .map_err(|e| anyhow!("Failed to bind IDE bridge server: {}", e))?;

        let local_port = listener.local_addr()?.port();
        *running = true;
        drop(running);

        let connections = self.connections.clone();
        let shutdown_senders = self.shutdown_senders.clone();
        let running_flag = self.running.clone();

        tokio::spawn(async move {
            loop {
                let is_running = *running_flag.lock().await;
                if !is_running {
                    break;
                }

                tokio::select! {
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((stream, _addr)) => {
                                let conn_id = format!("ide-{}", uuid::Uuid::new_v4().to_string()[..8].to_string());
                                let (tx, rx) = watch::channel(false);

                                let conn = IdeConnection {
                                    id: conn_id.clone(),
                                    ide_type: IdeType::Unknown,
                                    status: IdeStatus::Connected,
                                    workspace: None,
                                    connected_at: chrono::Utc::now().to_rfc3339(),
                                    last_heartbeat: chrono::Utc::now().to_rfc3339(),
                                };
                                connections.lock().await.insert(conn_id.clone(), conn);
                                shutdown_senders.lock().await.insert(conn_id.clone(), tx);

                                let conns = connections.clone();
                                let senders = shutdown_senders.clone();
                                let cid = conn_id.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = Self::handle_connection(stream, conns, senders, cid.clone(), rx).await {
                                        tracing::error!(module = "IDE", "Connection {} error: {}", cid, e);
                                    }
                                });
                            }
                            Err(e) => {
                                tracing::error!(module = "IDE", "Accept error: {}", e);
                            }
                        }
                    }
                    _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {}
                }
            }
        });

        Ok(local_port)
    }

    pub async fn stop_server(&self) {
        let mut running = self.running.lock().await;
        *running = false;

        let senders = self.shutdown_senders.lock().await;
        for (_, tx) in senders.iter() {
            let _ = tx.send(true);
        }
        drop(senders);

        self.connections.lock().await.clear();
        self.shutdown_senders.lock().await.clear();
    }

    async fn handle_connection(
        stream: TcpStream,
        connections: Arc<Mutex<HashMap<String, IdeConnection>>>,
        shutdown_senders: Arc<Mutex<HashMap<String, watch::Sender<bool>>>>,
        conn_id: String,
        mut shutdown_rx: watch::Receiver<bool>,
    ) -> Result<()> {
        let (reader, mut writer) = stream.into_split();
        let mut lines = BufReader::new(reader).lines();

        loop {
            tokio::select! {
                line_result = lines.next_line() => {
                    match line_result {
                        Ok(Some(line)) => {
                            if let Ok(msg) = serde_json::from_str::<IdeMessage>(&line) {
                                match msg.msg_type {
                                    IdeMessageType::Heartbeat => {
                                        let mut conns = connections.lock().await;
                                        if let Some(conn) = conns.get_mut(&conn_id) {
                                            conn.last_heartbeat = chrono::Utc::now().to_rfc3339();
                                        }
                                        let response = IdeMessage {
                                            id: msg.id,
                                            msg_type: IdeMessageType::Heartbeat,
                                            payload: serde_json::json!({"status": "ok"}),
                                            timestamp: chrono::Utc::now().to_rfc3339(),
                                        };
                                        let data = serde_json::to_string(&response)? + "\n";
                                        let _ = writer.write_all(data.as_bytes()).await;
                                    }
                                    IdeMessageType::Command => {
                                        if let Some(ide_type_str) = msg.payload.get("ide_type").and_then(|v| v.as_str()) {
                                            let ide_type = match ide_type_str {
                                                "vscode" => IdeType::VSCode,
                                                "cursor" => IdeType::Cursor,
                                                "jetbrains" => IdeType::JetBrains,
                                                "neovim" => IdeType::Neovim,
                                                _ => IdeType::Unknown,
                                            };
                                            let mut conns = connections.lock().await;
                                            if let Some(conn) = conns.get_mut(&conn_id) {
                                                conn.ide_type = ide_type;
                                                conn.workspace = msg.payload.get("workspace").and_then(|v| v.as_str()).map(String::from);
                                            }
                                        }
                                        let response = IdeMessage {
                                            id: msg.id,
                                            msg_type: IdeMessageType::Response,
                                            payload: serde_json::json!({"status": "registered"}),
                                            timestamp: chrono::Utc::now().to_rfc3339(),
                                        };
                                        let data = serde_json::to_string(&response)? + "\n";
                                        let _ = writer.write_all(data.as_bytes()).await;
                                    }
                                    _ => {
                                        let response = IdeMessage {
                                            id: msg.id,
                                            msg_type: IdeMessageType::Response,
                                            payload: serde_json::json!({"status": "received", "type": format!("{:?}", msg.msg_type)}),
                                            timestamp: chrono::Utc::now().to_rfc3339(),
                                        };
                                        let data = serde_json::to_string(&response)? + "\n";
                                        let _ = writer.write_all(data.as_bytes()).await;
                                    }
                                }
                            }
                        }
                        Ok(None) => break,
                        Err(_) => break,
                    }
                }
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        break;
                    }
                }
            }
        }

        let mut conns = connections.lock().await;
        if let Some(conn) = conns.get_mut(&conn_id) {
            conn.status = IdeStatus::Disconnected;
        }
        shutdown_senders.lock().await.remove(&conn_id);

        Ok(())
    }

    pub async fn list_connections(&self) -> Vec<IdeConnection> {
        let conns = self.connections.lock().await;
        conns.values().filter(|c| c.status == IdeStatus::Connected).cloned().collect()
    }

    pub async fn get_connection(&self, id: &str) -> Option<IdeConnection> {
        let conns = self.connections.lock().await;
        conns.get(id).cloned()
    }

    pub async fn disconnect(&self, id: &str) -> Result<()> {
        if let Some(tx) = self.shutdown_senders.lock().await.remove(id) {
            let _ = tx.send(true);
        }
        let mut conns = self.connections.lock().await;
        if let Some(conn) = conns.get_mut(id) {
            conn.status = IdeStatus::Disconnected;
        }
        Ok(())
    }

    pub async fn get_status(&self) -> IdeBridgeStatus {
        let conns = self.connections.lock().await;
        let running = self.running.lock().await;
        IdeBridgeStatus {
            server_running: *running,
            port: self.config.port,
            active_connections: conns.values().filter(|c| c.status == IdeStatus::Connected).count(),
            total_connections: conns.len(),
        }
    }

    pub async fn send_to_ide(&self, conn_id: &str, _message: IdeMessage) -> Result<()> {
        let conns = self.connections.lock().await;
        if let Some(conn) = conns.get(conn_id) {
            if conn.status != IdeStatus::Connected {
                return Err(anyhow!("IDE connection not active"));
            }
        } else {
            return Err(anyhow!("IDE connection not found"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeBridgeStatus {
    pub server_running: bool,
    pub port: u16,
    pub active_connections: usize,
    pub total_connections: usize,
}
