use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[cfg(unix)]
use tokio::net::UnixStream;

#[cfg(windows)]
use tokio::net::windows::named_pipe::ClientOptions as PipeClientOptions;

const SOCKET_PATH: &str = "/tmp/agent-worker.sock";
const PIPE_NAME: &str = r"\\.\pipe\agent-worker";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarCommand {
    pub id: String,
    pub command: SidecarCommandType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SidecarCommandType {
    #[serde(rename = "execute_action")]
    ExecuteAction {
        action_type: String,
        coordinate: Option<[i32; 2]>,
        button: Option<String>,
        key: Option<String>,
        text: Option<String>,
        scroll_y: Option<i32>,
        scroll_x: Option<i32>,
        duration_ms: Option<u64>,
    },
    #[serde(rename = "open_browser")]
    OpenBrowser { url: Option<String> },
    #[serde(rename = "screenshot")]
    Screenshot,
    #[serde(rename = "ping")]
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarResponse {
    pub id: String,
    pub success: bool,
    pub screenshot: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

pub struct SidecarClient;

impl SidecarClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn send_command(&self, command: SidecarCommand) -> Result<SidecarResponse> {
        let json = serde_json::to_vec(&command)?;

        #[cfg(unix)]
        {
            let mut stream = UnixStream::connect(SOCKET_PATH)
                .await
                .map_err(|e| anyhow!("Failed to connect to sidecar worker: {}", e))?;

            stream.write_all(&json).await?;
            stream.write_all(b"\n").await?;

            let mut buf = vec![0u8; 65536];
            let n = stream.read(&mut buf).await?;

            serde_json::from_slice(&buf[..n])
                .map_err(|e| anyhow!("Failed to parse sidecar response: {}", e))
        }

        #[cfg(windows)]
        {
            let client = PipeClientOptions::new()
                .open(PIPE_NAME)
                .map_err(|e| anyhow!("Failed to connect to sidecar named pipe: {}", e))?;

            client.writable().await?;
            client.try_write(&json)?;

            let mut buf = vec![0u8; 65536];
            client.readable().await?;
            let n = client.try_read(&mut buf)?;

            serde_json::from_slice(&buf[..n])
                .map_err(|e| anyhow!("Failed to parse sidecar response: {}", e))
        }
    }

    pub async fn ping(&self) -> Result<bool> {
        let cmd = SidecarCommand {
            id: uuid::Uuid::new_v4().to_string(),
            command: SidecarCommandType::Ping,
        };
        match self.send_command(cmd).await {
            Ok(resp) => Ok(resp.success),
            Err(_) => Ok(false),
        }
    }

    pub async fn execute_action(
        &self,
        action_type: &str,
        coordinate: Option<[i32; 2]>,
        button: Option<&str>,
        key: Option<&str>,
        text: Option<&str>,
        scroll_y: Option<i32>,
        scroll_x: Option<i32>,
        duration_ms: Option<u64>,
    ) -> Result<SidecarResponse> {
        let cmd = SidecarCommand {
            id: uuid::Uuid::new_v4().to_string(),
            command: SidecarCommandType::ExecuteAction {
                action_type: action_type.to_string(),
                coordinate,
                button: button.map(|s| s.to_string()),
                key: key.map(|s| s.to_string()),
                text: text.map(|s| s.to_string()),
                scroll_y,
                scroll_x,
                duration_ms,
            },
        };
        self.send_command(cmd).await
    }

    pub async fn open_browser(&self, url: Option<&str>) -> Result<SidecarResponse> {
        let cmd = SidecarCommand {
            id: uuid::Uuid::new_v4().to_string(),
            command: SidecarCommandType::OpenBrowser {
                url: url.map(|s| s.to_string()),
            },
        };
        self.send_command(cmd).await
    }

    pub async fn take_screenshot(&self) -> Result<SidecarResponse> {
        let cmd = SidecarCommand {
            id: uuid::Uuid::new_v4().to_string(),
            command: SidecarCommandType::Screenshot,
        };
        self.send_command(cmd).await
    }
}