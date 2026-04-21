//! SSH Connection Support

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub private_key_path: Option<PathBuf>,
    pub known_hosts_path: Option<PathBuf>,
    pub timeout_secs: u32,
    pub keepalive_interval_secs: u32,
}

impl Default for SshConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 22,
            username: String::new(),
            password: None,
            private_key_path: None,
            known_hosts_path: None,
            timeout_secs: 30,
            keepalive_interval_secs: 60,
        }
    }
}

impl SshConfig {
    pub fn new(host: &str, username: &str) -> Self {
        Self {
            host: host.to_string(),
            username: username.to_string(),
            ..Default::default()
        }
    }
    
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
    
    pub fn with_password(mut self, password: &str) -> Self {
        self.password = Some(password.to_string());
        self
    }
    
    pub fn with_private_key(mut self, path: PathBuf) -> Self {
        self.private_key_path = Some(path);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshSession {
    pub id: String,
    pub config: SshConfig,
    pub status: SshStatus,
    pub connected_at: Option<DateTime<Utc>>,
    pub last_activity: Option<DateTime<Utc>>,
    pub commands_executed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SshStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshCommandResult {
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

pub struct SshClient {
    sessions: Arc<RwLock<HashMap<String, SshSession>>>,
    config_dir: PathBuf,
}

impl SshClient {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let config_dir = home.join(".claude-code").join("ssh");
        
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config_dir,
        }
    }
    
    pub async fn connect(&self, config: SshConfig) -> anyhow::Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        
        let session = SshSession {
            id: session_id.clone(),
            config: config.clone(),
            status: SshStatus::Connecting,
            connected_at: Some(Utc::now()),
            last_activity: Some(Utc::now()),
            commands_executed: 0,
        };
        
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.clone(), session);
        }
        
        println!("🔌 Connecting to {}@{}:{}", config.username, config.host, config.port);
        
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.status = SshStatus::Connected;
        }
        
        println!("✅ SSH session established: {}", session_id);
        
        Ok(session_id)
    }
    
    pub async fn disconnect(&self, session_id: &str) -> anyhow::Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = SshStatus::Disconnected;
            println!("🔌 SSH session disconnected: {}", session_id);
        }
        Ok(())
    }
    
    pub async fn execute(&self, session_id: &str, command: &str) -> anyhow::Result<SshCommandResult> {
        let start = std::time::Instant::now();
        
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;
        
        if session.status != SshStatus::Connected {
            return Err(anyhow::anyhow!("Session not connected"));
        }
        drop(sessions);
        
        println!("🔧 Executing: {}", command);
        
        let ssh_args = self.build_ssh_args(session_id, command);
        
        let output = tokio::process::Command::new("ssh")
            .args(&ssh_args)
            .output()
            .await;
        
        let result = match output {
            Ok(output) => SshCommandResult {
                command: command.to_string(),
                exit_code: output.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                duration_ms: start.elapsed().as_millis() as u64,
            },
            Err(e) => SshCommandResult {
                command: command.to_string(),
                exit_code: -1,
                stdout: String::new(),
                stderr: e.to_string(),
                duration_ms: start.elapsed().as_millis() as u64,
            },
        };
        
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.commands_executed += 1;
            session.last_activity = Some(Utc::now());
        }
        
        Ok(result)
    }
    
    fn build_ssh_args(&self, session_id: &str, command: &str) -> Vec<String> {
        let sessions = futures::executor::block_on(self.sessions.read());
        let session = sessions.get(session_id);
        
        let mut args = Vec::new();
        
        if let Some(session) = session {
            args.push("-p".to_string());
            args.push(session.config.port.to_string());
            
            if let Some(ref key_path) = session.config.private_key_path {
                args.push("-i".to_string());
                args.push(key_path.to_string_lossy().to_string());
            }
            
            args.push(format!("{}@{}", session.config.username, session.config.host));
            args.push(command.to_string());
        }
        
        args
    }
    
    pub async fn upload(&self, session_id: &str, local: &PathBuf, remote: &str) -> anyhow::Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;
        
        let remote_path = format!("{}@{}:{}", 
            session.config.username, 
            session.config.host, 
            remote
        );
        
        println!("📤 Uploading {:?} to {}", local, remote_path);
        
        let output = tokio::process::Command::new("scp")
            .arg("-P")
            .arg(session.config.port.to_string())
            .arg(local)
            .arg(&remote_path)
            .output()
            .await?;
        
        if output.status.success() {
            println!("✅ Upload complete");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Upload failed: {}", 
                String::from_utf8_lossy(&output.stderr)))
        }
    }
    
    pub async fn download(&self, session_id: &str, remote: &str, local: &PathBuf) -> anyhow::Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;
        
        let remote_path = format!("{}@{}:{}", 
            session.config.username, 
            session.config.host, 
            remote
        );
        
        println!("📥 Downloading {} to {:?}", remote_path, local);
        
        let output = tokio::process::Command::new("scp")
            .arg("-P")
            .arg(session.config.port.to_string())
            .arg(&remote_path)
            .arg(local)
            .output()
            .await?;
        
        if output.status.success() {
            println!("✅ Download complete");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Download failed: {}", 
                String::from_utf8_lossy(&output.stderr)))
        }
    }
    
    pub async fn list_sessions(&self) -> Vec<SshSession> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }
    
    pub async fn get_session(&self, session_id: &str) -> Option<SshSession> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }
    
    pub async fn save_config(&self, name: &str, config: &SshConfig) -> anyhow::Result<()> {
        tokio::fs::create_dir_all(&self.config_dir).await?;
        
        let path = self.config_dir.join(format!("{}.json", name));
        let content = serde_json::to_string_pretty(config)?;
        tokio::fs::write(&path, content).await?;
        
        Ok(())
    }
    
    pub async fn load_config(&self, name: &str) -> anyhow::Result<Option<SshConfig>> {
        let path = self.config_dir.join(format!("{}.json", name));
        
        if !path.exists() {
            return Ok(None);
        }
        
        let content = tokio::fs::read_to_string(&path).await?;
        let config: SshConfig = serde_json::from_str(&content)?;
        
        Ok(Some(config))
    }
}

impl Default for SshClient {
    fn default() -> Self {
        Self::new()
    }
}
