//! MCP Transport - Communication transport layer

use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use super::McpMessage;

pub enum Transport {
    Stdio(StdioTransport),
    Tcp(TcpTransport),
    WebSocket(WebSocketTransport),
}

pub struct StdioTransport {
    reader: Arc<RwLock<Option<Box<dyn BufRead + Send + Sync>>>>,
    writer: Arc<RwLock<Option<Box<dyn Write + Send + Sync>>>>,
}

impl StdioTransport {
    pub fn new() -> Self {
        Self {
            reader: Arc::new(RwLock::new(None)),
            writer: Arc::new(RwLock::new(None)),
        }
    }
    
    pub async fn send(&self, message: &McpMessage) -> anyhow::Result<()> {
        let mut writer = self.writer.write().await;
        if let Some(w) = writer.as_mut() {
            let json = serde_json::to_string(message)?;
            writeln!(w, "Content-Length: {}", json.len())?;
            writeln!(w)?;
            write!(w, "{}", json)?;
            w.flush()?;
        }
        Ok(())
    }
    
    pub async fn receive(&self) -> anyhow::Result<Option<McpMessage>> {
        let mut reader = self.reader.write().await;
        if let Some(r) = reader.as_mut() {
            let mut line = String::new();
            
            let mut content_length = 0;
            loop {
                line.clear();
                r.read_line(&mut line)?;
                let line = line.trim();
                if line.is_empty() {
                    break;
                }
                if let Some(len) = line.strip_prefix("Content-Length: ") {
                    content_length = len.parse()?;
                }
            }
            
            if content_length > 0 {
                let mut buffer = vec![0u8; content_length];
                r.read_exact(&mut buffer)?;
                let message: McpMessage = serde_json::from_slice(&buffer)?;
                return Ok(Some(message));
            }
        }
        Ok(None)
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TcpTransport {
    sender: Option<mpsc::Sender<McpMessage>>,
    receiver: Option<mpsc::Receiver<McpMessage>>,
}

impl TcpTransport {
    pub fn new(_host: &str, _port: u16) -> Self {
        Self {
            sender: None,
            receiver: None,
        }
    }
    
    pub async fn connect(&mut self) -> anyhow::Result<()> {
        let (tx, rx) = mpsc::channel(100);
        self.sender = Some(tx);
        self.receiver = Some(rx);
        Ok(())
    }
    
    pub async fn send(&self, message: &McpMessage) -> anyhow::Result<()> {
        if let Some(sender) = &self.sender {
            sender.send(message.clone()).await?;
        }
        Ok(())
    }
    
    pub async fn receive(&mut self) -> Option<McpMessage> {
        if let Some(receiver) = &mut self.receiver {
            receiver.recv().await
        } else {
            None
        }
    }
}

pub struct WebSocketTransport {
    sender: Option<mpsc::Sender<McpMessage>>,
    receiver: Option<mpsc::Receiver<McpMessage>>,
}

impl WebSocketTransport {
    pub fn new(_url: &str) -> Self {
        Self {
            sender: None,
            receiver: None,
        }
    }
    
    pub async fn connect(&mut self) -> anyhow::Result<()> {
        let (tx, rx) = mpsc::channel(100);
        self.sender = Some(tx);
        self.receiver = Some(rx);
        Ok(())
    }
    
    pub async fn send(&self, message: &McpMessage) -> anyhow::Result<()> {
        if let Some(sender) = &self.sender {
            sender.send(message.clone()).await?;
        }
        Ok(())
    }
    
    pub async fn receive(&mut self) -> Option<McpMessage> {
        if let Some(receiver) = &mut self.receiver {
            receiver.recv().await
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    #[serde(rename = "type")]
    pub transport_type: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub url: Option<String>,
}

impl TransportConfig {
    pub fn stdio() -> Self {
        Self {
            transport_type: "stdio".to_string(),
            host: None,
            port: None,
            url: None,
        }
    }
    
    pub fn tcp(host: &str, port: u16) -> Self {
        Self {
            transport_type: "tcp".to_string(),
            host: Some(host.to_string()),
            port: Some(port),
            url: None,
        }
    }
    
    pub fn websocket(url: &str) -> Self {
        Self {
            transport_type: "websocket".to_string(),
            host: None,
            port: None,
            url: Some(url.to_string()),
        }
    }
}
