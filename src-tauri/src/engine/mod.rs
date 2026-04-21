use crate::bridge::ChatRequest;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineHandle {
    pub conv_id: String,
    pub pid: Option<u32>,
    pub model: String,
    pub session_id: Option<String>,
    pub state: EngineState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineState {
    Idle,
    Processing,
    Ready,
    Closed,
}

pub struct EnginePool {
    engines: HashMap<String, EngineHandle>,
    workspace: PathBuf,
}

impl EnginePool {
    pub fn new() -> Self {
        let workspace = dirs_data_path();
        Self {
            engines: HashMap::new(),
            workspace,
        }
    }

    pub async fn spawn_engine(&mut self, conv_id: &str, model: &str) -> Result<EngineHandle> {
        let bun_path = find_bun_path();

        let handle = if let Some(bun) = bun_path {
            let engine_bin = self.workspace.join("engine").join("bin").join("claude-haha");
            let _child = Command::new(&bun)
                .arg("run")
                .arg(&engine_bin)
                .env("MODEL", model)
                .env("CLAUDE_CODE_ENTRYPOINT", "bridge")
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .stdin(std::process::Stdio::piped())
                .spawn()
                .ok();

            EngineHandle {
                conv_id: conv_id.to_string(),
                pid: None,
                model: model.to_string(),
                session_id: Some(uuid::Uuid::new_v4().to_string()),
                state: EngineState::Idle,
            }
        } else {
            EngineHandle {
                conv_id: conv_id.to_string(),
                pid: None,
                model: model.to_string(),
                session_id: Some(uuid::Uuid::new_v4().to_string()),
                state: EngineState::Idle,
            }
        };

        self.engines.insert(conv_id.to_string(), handle.clone());
        Ok(handle)
    }

    pub async fn send_message(
        &mut self,
        conv_id: &str,
        req: &ChatRequest,
    ) -> Result<serde_json::Value> {
        if !self.engines.contains_key(conv_id) {
            self.spawn_engine(conv_id, &req.model).await?;
        }

        let env_token = req.env_token.clone().unwrap_or_default();
        let env_base_url = req.env_base_url.clone().unwrap_or_default();

        let client = reqwest::Client::new();
        let api_url = resolve_api_url(&req.user_mode, &env_base_url);

        let messages = req.get_messages();

        let body = serde_json::json!({
            "model": req.model,
            "messages": messages,
            "max_tokens": 8192,
            "stream": false,
        });

        let mut request_builder = client
            .post(&api_url)
            .header("Content-Type", "application/json")
            .json(&body);

        if !env_token.is_empty() {
            if api_url.contains("anthropic") {
                request_builder = request_builder
                    .header("x-api-key", &env_token)
                    .header("anthropic-version", "2023-06-01");
            } else {
                request_builder = request_builder.bearer_auth(&env_token);
            }
        }

        let response = request_builder.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("API error {}: {}", status, text);
        }

        let result: serde_json::Value = response.json().await?;
        Ok(result)
    }

    pub fn get_engine(&self, conv_id: &str) -> Option<&EngineHandle> {
        self.engines.get(conv_id)
    }

    pub fn remove_engine(&mut self, conv_id: &str) {
        self.engines.remove(conv_id);
    }
}

fn find_bun_path() -> Option<PathBuf> {
    let candidates = if cfg!(target_os = "windows") {
        vec![
            PathBuf::from(r"C:\Users\user\.bun\bin\bun.exe"),
            PathBuf::from("bun.exe"),
        ]
    } else {
        vec![
            PathBuf::from("/usr/local/bin/bun"),
            PathBuf::from(format!("{}/.bun/bin/bun", std::env::var("HOME").unwrap_or_default())),
            PathBuf::from("bun"),
        ]
    };

    for path in &candidates {
        if path.exists() {
            return Some(path.clone());
        }
    }

    if let Ok(output) = std::process::Command::new("which").arg("bun").output() {
        if output.status.success() {
            let p = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !p.is_empty() {
                return Some(PathBuf::from(p));
            }
        }
    }

    None
}

fn dirs_data_path() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join("Claude Desktop")
}

fn resolve_api_url(user_mode: &Option<String>, env_base_url: &str) -> String {
    if !env_base_url.is_empty() {
        if env_base_url.contains("/v1/messages") || env_base_url.contains("/v1/chat/completions") {
            return env_base_url.to_string();
        }
        if env_base_url.contains("anthropic") || env_base_url.contains("claude") {
            return format!("{}/v1/messages", env_base_url.trim_end_matches('/'));
        }
        return format!("{}/v1/chat/completions", env_base_url.trim_end_matches('/'));
    }

    match user_mode.as_deref() {
        Some("clawparrot") => "http://127.0.0.1:30090/api/v1/messages".to_string(),
        _ => "https://api.anthropic.com/v1/messages".to_string(),
    }
}
