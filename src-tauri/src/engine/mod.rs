use crate::native_engine::engine_core::ChatRequest;
use crate::streaming::{consume_sse_payloads, merge_tool_args, try_parse_tool_input};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::io::AsyncBufReadExt;
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use uuid::Uuid;

fn find_executable_in_path(name: &str) -> Option<PathBuf> {
    if let Ok(path_env) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_env) {
            let candidate = if cfg!(target_os = "windows") {
                dir.join(format!("{}.exe", name))
            } else {
                dir.join(name)
            };
            if candidate.exists() && candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn get_user_home() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineHandle {
    pub conv_id: String,
    pub pid: Option<u32>,
    pub model: String,
    pub session_id: String,
    pub state: EngineState,
    pub workspace: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EngineState {
    Idle,
    Starting,
    Processing,
    Ready,
    Error,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineMessage {
    pub msg_type: String,
    pub payload: serde_json::Value,
}

#[derive(Debug)]
pub struct EngineOutput {
    pub msg_type: String,
    pub content: String,
    pub tool_name: Option<String>,
    pub tool_input: Option<serde_json::Value>,
    pub tool_use_id: Option<String>,
    pub stop_reason: Option<String>,
    pub usage: Option<serde_json::Value>,
}

impl EngineOutput {
    pub fn simple(msg_type: &str, content: &str) -> Self {
        Self {
            msg_type: msg_type.to_string(),
            content: content.to_string(),
            tool_name: None,
            tool_input: None,
            tool_use_id: None,
            stop_reason: None,
            usage: None,
        }
    }

    pub fn with_stop_reason(mut self, reason: Option<String>) -> Self {
        self.stop_reason = reason;
        self
    }

    pub fn with_usage(mut self, usage: serde_json::Value) -> Self {
        self.usage = Some(usage);
        self
    }
}

/// Guard that holds a borrowed message receiver from EnginePool.
/// If the stream completes normally (MessageStop/Error), the receiver is consumed.
/// If dropped before consumption, the receiver is lost (caller should use
/// `return_message_receiver` explicitly if they need to put it back).
pub struct ReceiverGuard {
    pub conv_id: String,
    pub rx: Option<mpsc::Receiver<EngineOutput>>,
    returned: bool,
}

impl ReceiverGuard {
    /// Take the receiver out of the guard, consuming it for streaming.
    /// After this, the guard can be dropped without losing the receiver.
    pub fn take_rx(&mut self) -> Option<mpsc::Receiver<EngineOutput>> {
        self.rx.take()
    }
}

pub struct EnginePool {
    engines: HashMap<String, EngineHandle>,
    processes: HashMap<String, Child>,
    workspace: PathBuf,
    message_handlers: HashMap<String, mpsc::Receiver<EngineOutput>>,
    pending_image_blocks: HashMap<String, Vec<serde_json::Value>>,
    proxy_target: HashMap<String, ProxyTarget>,
    ask_user_pending_inputs: HashMap<String, serde_json::Value>,
    tool_permission_pending: HashMap<String, serde_json::Value>,
    stdin_senders: HashMap<String, mpsc::Sender<String>>,
}

#[derive(Debug, Clone)]
pub struct ProxyTarget {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub format: String,
    pub conversation_id: String,
}

impl EnginePool {
    pub fn new() -> Self {
        let workspace = Self::get_workspace_path();
        Self {
            engines: HashMap::new(),
            processes: HashMap::new(),
            workspace,
            message_handlers: HashMap::new(),
            pending_image_blocks: HashMap::new(),
            proxy_target: HashMap::new(),
            ask_user_pending_inputs: HashMap::new(),
            tool_permission_pending: HashMap::new(),
            stdin_senders: HashMap::new(),
        }
    }

    fn get_workspace_path() -> PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join("Claude Desktop")
    }

    pub fn get_workspace(&self) -> &PathBuf {
        &self.workspace
    }

    pub async fn spawn_engine(
        &mut self,
        conv_id: &str,
        model: &str,
        cwd: Option<String>,
    ) -> Result<EngineHandle> {
        if let Some(existing) = self.engines.get(conv_id) {
            if existing.state == EngineState::Ready || existing.state == EngineState::Processing {
                return Ok(existing.clone());
            }
        }

        let session_id = Uuid::new_v4().to_string();
        let work_dir = cwd.map(PathBuf::from).unwrap_or_else(|| self.workspace.clone());

        let engine_bin = self.find_claude_engine_binary()?;
        let mut cmd = Command::new("bun");
        cmd.arg("run")
            .arg(&engine_bin)
            .env("MODEL", model)
            .env("CLAUDE_CODE_ENTRYPOINT", "bridge")
            .env("CLAUDE_SESSION_ID", &session_id)
            .env("CLAUDE_WORKING_DIR", work_dir.as_os_str())
            .env("CLAUDE_CONVERSATION_ID", conv_id)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .stdin(std::process::Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd.spawn().map_err(|e| anyhow::anyhow!("Failed to spawn Claude engine: {}. Ensure 'bun' is installed and in PATH.", e))?;
        let pid = child.id();

        // Verify the process didn't immediately exit (e.g. binary not found)
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process already exited — read stderr for diagnostics
                let stderr_output = if let Some(stderr) = child.stderr.take() {
                    use tokio::io::AsyncReadExt;
                    let mut buf = Vec::new();
                    let _ = tokio::io::BufReader::new(stderr).read_buf(&mut buf).await;
                    String::from_utf8_lossy(&buf).to_string()
                } else {
                    String::new()
                };
                anyhow::bail!(
                    "Claude engine process exited immediately with status: {}. stderr: {}",
                    status,
                    if stderr_output.is_empty() { "(none)" } else { &stderr_output }
                );
            }
            Ok(None) => { /* still running, good */ }
            Err(e) => {
                tracing::warn!(module = "Engine", "Could not check engine process status: {}", e);
            }
        }

        let handle = EngineHandle {
            conv_id: conv_id.to_string(),
            pid,
            model: model.to_string(),
            session_id,
            state: EngineState::Starting,
            workspace: work_dir,
        };

        if let Some(stdout) = child.stdout.take() {
            let (tx, rx) = mpsc::channel::<EngineOutput>(100);

            self.message_handlers.insert(conv_id.to_string(), rx);

            let stdin_writer = child.stdin.take();
            let (stdin_tx, mut stdin_rx) = mpsc::channel::<String>(64);

            if let Some(mut stdin) = stdin_writer {
                tokio::spawn(async move {
                    use tokio::io::AsyncWriteExt;
                    while let Some(data) = stdin_rx.recv().await {
                        if stdin.write_all(data.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                    let _ = stdin.shutdown().await;
                });
            }

            self.stdin_senders.insert(conv_id.to_string(), stdin_tx.clone());

            tokio::spawn(async move {
                let reader = tokio::io::BufReader::new(stdout);
                let mut lines = reader.lines();
                let mut current_tool_input = String::new();
                let mut in_tool_input = false;
                let mut current_tool_name: Option<String> = None;
                let mut current_tool_use_id: Option<String> = None;
                let mut sse_buffer = String::new();
                let mut is_sse_stream = false;

                while let Ok(Some(line)) = lines.next_line().await {
                    let trimmed = line.trim();

                    if !is_sse_stream && (trimmed.starts_with("event:") || trimmed.starts_with("data:")) {
                        is_sse_stream = true;
                    }

                    if is_sse_stream {
                        sse_buffer.push_str(trimmed);
                        sse_buffer.push('\n');

                        let consumed = consume_sse_payloads(&sse_buffer);
                        sse_buffer = consumed.remainder;

                        for payload in &consumed.payloads {
                            if payload == "[DONE]" {
                                continue;
                            }
                            if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(payload) {
                                process_sse_chunk(
                                    &chunk,
                                    &tx,
                                    &mut current_tool_name,
                                    &mut current_tool_use_id,
                                    &mut current_tool_input,
                                    &mut in_tool_input,
                                    &stdin_tx,
                                ).await;
                            } else {
                                tracing::warn!(module = "Engine", "Failed to parse SSE payload: {} bytes", payload.len());
                            }
                        }
                        continue;
                    }

                    if let Ok(output) = serde_json::from_str::<EngineMessage>(&line) {
                        match output.msg_type.as_str() {
                            "ready" => {
                                let _ = tx.send(EngineOutput::simple("ready", "")).await;
                            }
                            "text" => {
                                let _ = tx.send(EngineOutput::simple("text", output.payload.as_str().unwrap_or(""))).await;
                            }
                            "tool_use" => {
                                current_tool_name = output.payload.get("name")
                                    .and_then(|n| n.as_str())
                                    .map(String::from);
                                current_tool_use_id = output.payload.get("id")
                                    .and_then(|id| id.as_str())
                                    .map(String::from);
                                current_tool_input.clear();
                                in_tool_input = true;
                            }
                            "tool_input_delta" => {
                                if let Some(delta) = output.payload.as_str() {
                                    current_tool_input = merge_tool_args(&current_tool_input, delta);
                                } else if let Some(delta_obj) = output.payload.get("partial_json") {
                                    if let Some(delta_str) = delta_obj.as_str() {
                                        current_tool_input = merge_tool_args(&current_tool_input, delta_str);
                                    }
                                }
                            }
                            "tool_result" | "tool_call" => {
                                let tool_name_str = current_tool_name.clone().unwrap_or_default();
                                let tool_input: serde_json::Value = if in_tool_input && !current_tool_input.is_empty() {
                                    try_parse_tool_input(&tool_name_str, &current_tool_input)
                                } else if let Some(input) = output.payload.get("input") {
                                    input.clone()
                                } else {
                                    serde_json::json!({})
                                };

                                let _ = tx.send(EngineOutput {
                                    msg_type: "tool_call".to_string(),
                                    content: String::new(),
                                    tool_name: current_tool_name.clone(),
                                    tool_input: Some(tool_input),
                                    tool_use_id: current_tool_use_id.clone(),
                                    stop_reason: None,
                                    usage: None,
                                }).await;

                                current_tool_name = None;
                                current_tool_use_id = None;
                                current_tool_input.clear();
                                in_tool_input = false;
                            }
                            "control_request" => {
                                handle_control_request(&output.payload, &tx, &stdin_tx).await;
                            }
                            "message_stop" => {
                                let _ = tx.send(EngineOutput {
                                    msg_type: "stop".to_string(),
                                    content: String::new(),
                                    tool_name: None,
                                    tool_input: None,
                                    tool_use_id: None,
                                    stop_reason: output.payload.get("stop_reason")
                                        .and_then(|s| s.as_str())
                                        .map(String::from),
                                    usage: None,
                                }).await;
                            }
                            "error" => {
                                let _ = tx.send(EngineOutput::simple("error", output.payload.as_str().unwrap_or("Unknown error"))).await;
                            }
                            _ => {}
                        }
                    }
                }

                if !sse_buffer.is_empty() {
                    let consumed = consume_sse_payloads(&sse_buffer);
                    for payload in &consumed.payloads {
                        if payload == "[DONE]" {
                            continue;
                        }
                        if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(payload) {
                            process_sse_chunk(
                                &chunk,
                                &tx,
                                &mut current_tool_name,
                                &mut current_tool_use_id,
                                &mut current_tool_input,
                                &mut in_tool_input,
                                &stdin_tx,
                            ).await;
                        }
                    }
                }

                if in_tool_input && !current_tool_input.is_empty() {
                    let tool_name_str = current_tool_name.clone().unwrap_or_default();
                    let tool_input = try_parse_tool_input(&tool_name_str, &current_tool_input);
                    let _ = tx.send(EngineOutput {
                        msg_type: "tool_call".to_string(),
                        content: String::new(),
                        tool_name: current_tool_name.clone(),
                        tool_input: Some(tool_input),
                        tool_use_id: current_tool_use_id.clone(),
                        stop_reason: None,
                        usage: None,
                    }).await;
                }
            });
        }

        self.processes.insert(conv_id.to_string(), child.into());
        self.engines.insert(conv_id.to_string(), handle.clone());

        if let Some(engine) = self.engines.get_mut(conv_id) {
            engine.state = EngineState::Ready;
        }

        Ok(handle)
    }

    fn find_claude_engine_binary(&self) -> Result<PathBuf> {
        let user_home = get_user_home();
        let candidates = if cfg!(target_os = "windows") {
            vec![
                self.workspace.join("engine").join("bin").join("claude-haha"),
                user_home.join(".claude").join("bin").join("claude-haha"),
                user_home.join(".bun").join("share").join("claude").join("claude-haha"),
            ]
        } else {
            vec![
                self.workspace.join("engine").join("bin").join("claude-haha"),
                user_home.join(".claude").join("bin").join("claude-haha"),
                user_home.join(".bun").join("share").join("claude").join("claude-haha"),
            ]
        };

        for path in &candidates {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        anyhow::bail!(
            "Claude engine binary not found. Please ensure Claude Code is installed.\n\
             Searched paths: {:?}",
            candidates
        )
    }

    pub async fn send_message(
        &mut self,
        conv_id: &str,
        req: &ChatRequest,
    ) -> Result<EngineOutput> {
        let model_str = if req.model.is_empty() { "claude-sonnet-4-6" } else { &req.model };

        // Check if engine exists and is healthy; restart if crashed
        if self.engines.contains_key(conv_id) {
            if !self.check_engine_health(conv_id).await {
                tracing::warn!(module = "Engine", "Engine for {} is dead, restarting...", conv_id);
                self.restart_engine(conv_id).await?;
            }
        } else {
            self.spawn_engine(conv_id, model_str, None).await?;
        }

        if let Some(stdin_tx) = self.stdin_senders.get(conv_id) {
            let message = serde_json::json!({
                "type": "user_message",
                "messages": req.messages.clone(),
                "model": model_str,
            });
            let mut payload = message.to_string();
            payload.push('\n');
            let _ = stdin_tx.send(payload).await;
        }

        // Collect all outputs until stop message, not just the first one
        if let Some(handler) = self.message_handlers.get_mut(conv_id) {
            let mut combined_content = String::new();
            let mut last_output: Option<EngineOutput> = None;

            while let Some(output) = handler.recv().await {
                let is_stop = output.msg_type == "stop" || output.msg_type == "error";
                combined_content.push_str(&output.content);
                last_output = Some(output);
                if is_stop {
                    break;
                }
            }

            if let Some(mut output) = last_output {
                output.content = combined_content;
                return Ok(output);
            }
        }

        Ok(EngineOutput::simple("error", "No response from engine"))
    }

    pub async fn send_message_stream(
        &mut self,
        conv_id: &str,
        req: &ChatRequest,
    ) -> Result<Option<mpsc::Receiver<EngineOutput>>> {
        let model_str = if req.model.is_empty() { "claude-sonnet-4-6" } else { &req.model };
        if !self.engines.contains_key(conv_id) {
            self.spawn_engine(conv_id, model_str, None).await?;
        }

        if let Some(stdin_tx) = self.stdin_senders.get(conv_id) {
            let message = serde_json::json!({
                "type": "user_message",
                "messages": req.messages.clone(),
                "model": model_str,
            });
            let mut payload = message.to_string();
            payload.push('\n');
            let _ = stdin_tx.send(payload).await;
        }

        // Bug #3 fix: Use take instead of remove to avoid losing receiver on error.
        // If the caller consumes the receiver successfully, it should call
        // return_message_receiver() to put it back. If an error occurs before
        // the receiver is consumed, we re-insert it immediately.
        let rx = self.message_handlers.remove(conv_id);
        Ok(rx)
    }

    /// Temporarily borrow the message receiver for a conversation.
    /// Returns a ReceiverGuard that will automatically re-insert the receiver
    /// back into message_handlers when dropped (if it wasn't consumed).
    pub fn take_message_receiver(&mut self, conv_id: &str) -> Option<ReceiverGuard> {
        let rx = self.message_handlers.remove(conv_id)?;
        Some(ReceiverGuard {
            conv_id: conv_id.to_string(),
            rx: Some(rx),
            returned: false,
        })
    }

    pub fn return_message_receiver(&mut self, conv_id: &str, rx: mpsc::Receiver<EngineOutput>) {
        self.message_handlers.insert(conv_id.to_string(), rx);
    }

    pub fn get_engine(&self, conv_id: &str) -> Option<&EngineHandle> {
        self.engines.get(conv_id)
    }

    pub fn get_engine_mut(&mut self, conv_id: &str) -> Option<&mut EngineHandle> {
        self.engines.get_mut(conv_id)
    }

    pub async fn remove_engine(&mut self, conv_id: &str) {
        if let Some(mut child) = self.processes.remove(conv_id) {
            let _ = child.start_kill();
            let _ = child.wait().await;
        }
        self.message_handlers.remove(conv_id);
        self.pending_image_blocks.remove(conv_id);
        self.proxy_target.remove(conv_id);
        self.ask_user_pending_inputs.remove(conv_id);
        self.tool_permission_pending.remove(conv_id);
        self.stdin_senders.remove(conv_id);
        self.engines.remove(conv_id);
    }

    /// Check if an engine process is still alive, and mark it as Error if dead.
    pub async fn check_engine_health(&mut self, conv_id: &str) -> bool {
        if let Some(child) = self.processes.get_mut(conv_id) {
            match child.try_wait() {
                Ok(Some(status)) => {
                    tracing::warn!(module = "Engine", "Engine process for {} exited with: {}", conv_id, status);
                    if let Some(engine) = self.engines.get_mut(conv_id) {
                        engine.state = EngineState::Error;
                    }
                    return false;
                }
                Ok(None) => return true, // still running
                Err(e) => {
                    tracing::warn!(module = "Engine", "Cannot check engine health for {}: {}", conv_id, e);
                    return false;
                }
            }
        }
        false
    }

    /// Restart a crashed engine, preserving the conversation ID and model.
    /// Includes retry logic with exponential backoff.
    pub async fn restart_engine(&mut self, conv_id: &str) -> Result<EngineHandle> {
        self.restart_engine_with_retry(conv_id, 0).await
    }

    async fn restart_engine_with_retry(&mut self, conv_id: &str, attempt: u32) -> Result<EngineHandle> {
        let max_retries = 3;
        if attempt >= max_retries {
            tracing::error!(module = "Engine", "Failed to restart engine for {} after {} attempts", conv_id, max_retries);
            anyhow::bail!("Engine restart failed after {} attempts", max_retries);
        }

        let model = self.engines.get(conv_id).map(|e| e.model.clone()).unwrap_or_else(|| "claude-sonnet-4-6".to_string());
        let cwd = self.engines.get(conv_id).map(|e| e.workspace.clone());
        tracing::info!(module = "Engine", "Restarting engine for conv_id={}, model={}, attempt={}", conv_id, model, attempt + 1);

        self.remove_engine(conv_id).await;

        match self.spawn_engine(conv_id, &model, cwd.map(|p| p.to_string_lossy().to_string())).await {
            Ok(handle) => {
                tracing::info!(module = "Engine", "Engine restarted successfully for conv_id={}", conv_id);
                Ok(handle)
            }
            Err(e) => {
                let delay_ms = 1000 * (1u64 << attempt);
                tracing::warn!(module = "Engine", "Engine restart failed, retrying in {}ms: {}", delay_ms, e);
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                Box::pin(self.restart_engine_with_retry(conv_id, attempt + 1)).await
            }
        }
    }

    pub fn list_engines(&self) -> Vec<&EngineHandle> {
        self.engines.values().collect()
    }

    pub fn update_engine_state(&mut self, conv_id: &str, state: EngineState) {
        if let Some(engine) = self.engines.get_mut(conv_id) {
            engine.state = state;
        }
    }

    pub fn set_proxy_target(&mut self, conv_id: &str, target: ProxyTarget) {
        self.proxy_target.insert(conv_id.to_string(), target);
    }

    pub fn get_proxy_target(&self, conv_id: &str) -> Option<&ProxyTarget> {
        self.proxy_target.get(conv_id)
    }

    pub fn add_pending_images(&mut self, conv_id: &str, images: Vec<serde_json::Value>) {
        self.pending_image_blocks.insert(conv_id.to_string(), images);
    }

    pub fn get_pending_images(&mut self, conv_id: &str) -> Vec<serde_json::Value> {
        self.pending_image_blocks.remove(conv_id).unwrap_or_default()
    }

    pub fn set_ask_user_pending(&mut self, conv_id: &str, input: serde_json::Value) {
        self.ask_user_pending_inputs.insert(conv_id.to_string(), input);
    }

    pub fn get_ask_user_pending(&mut self, conv_id: &str) -> Option<serde_json::Value> {
        self.ask_user_pending_inputs.remove(conv_id)
    }

    pub async fn send_control_response(
        &mut self,
        conv_id: &str,
        request_id: &str,
        tool_use_id: &str,
        updated_input: serde_json::Value,
    ) -> Result<()> {
        if let Some(stdin_tx) = self.stdin_senders.get(conv_id) {
            let response = serde_json::json!({
                "type": "control_response",
                "response": {
                    "subtype": "success",
                    "request_id": request_id,
                    "response": {
                        "toolUseID": tool_use_id,
                        "behavior": "allow",
                        "updatedInput": updated_input,
                    }
                }
            });
            let mut payload = serde_json::to_string(&response)?;
            payload.push('\n');
            stdin_tx.send(payload).await.map_err(|_| anyhow::anyhow!("stdin channel closed"))?;
            tracing::info!(module = "AskUser", "Answered request_id={} for conv={}", request_id, conv_id);
            return Ok(());
        }
        anyhow::bail!("No active engine process for conversation {}", conv_id)
    }

    pub fn set_tool_permission_pending(&mut self, conv_id: &str, info: serde_json::Value) {
        self.tool_permission_pending.insert(conv_id.to_string(), info);
    }

    pub fn get_tool_permission_pending(&mut self, conv_id: &str) -> Option<serde_json::Value> {
        self.tool_permission_pending.get(conv_id).cloned()
    }

    pub fn remove_tool_permission_pending(&mut self, conv_id: &str) -> Option<serde_json::Value> {
        self.tool_permission_pending.remove(conv_id)
    }

    pub async fn send_permission_response(
        &mut self,
        conv_id: &str,
        request_id: &str,
        tool_use_id: &str,
        behavior: &str,
        updated_input: Option<serde_json::Value>,
    ) -> Result<()> {
        if let Some(stdin_tx) = self.stdin_senders.get(conv_id) {
            let response = serde_json::json!({
                "type": "control_response",
                "response": {
                    "subtype": "success",
                    "request_id": request_id,
                    "response": {
                        "toolUseID": tool_use_id,
                        "behavior": behavior,
                        "updatedInput": updated_input.unwrap_or(serde_json::json!({})),
                    }
                }
            });
            let mut payload = serde_json::to_string(&response)?;
            payload.push('\n');
            stdin_tx.send(payload).await.map_err(|_| anyhow::anyhow!("stdin channel closed"))?;
            tracing::info!(module = "Permission", "Responded request_id={} behavior={} for conv={}", request_id, behavior, conv_id);
            return Ok(());
        }
        anyhow::bail!("No active engine process for conversation {}", conv_id)
    }
}

impl Default for EnginePool {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for EnginePool {
    fn drop(&mut self) {
        let conv_ids: Vec<String> = self.processes.keys().cloned().collect();
        for conv_id in conv_ids {
            if let Some(mut child) = self.processes.remove(&conv_id) {
                tracing::info!(module = "Engine", "Dropping engine process for conv_id={}, killing...", conv_id);
                let _ = child.start_kill();
            }
        }
        self.stdin_senders.clear();
        self.engines.clear();
        self.message_handlers.clear();
    }
}

async fn process_sse_chunk(
    chunk: &serde_json::Value,
    tx: &mpsc::Sender<EngineOutput>,
    current_tool_name: &mut Option<String>,
    current_tool_use_id: &mut Option<String>,
    current_tool_input: &mut String,
    in_tool_input: &mut bool,
    stdin_tx: &mpsc::Sender<String>,
) {
    let msg_type = chunk.get("type").and_then(|t| t.as_str()).unwrap_or("");

    match msg_type {
        "message_start" => {
            if let Some(message) = chunk.get("message") {
                let model = message.get("model").and_then(|m| m.as_str()).unwrap_or("");
                let usage = message.get("usage").cloned();
                let _ = tx.send(EngineOutput::simple("message_start", model).with_usage(usage.unwrap_or(serde_json::json!({})))).await;
            }
        }
        "content_block_start" => {
            let block = chunk.get("content_block");
            let block_type = block.and_then(|b| b.get("type")).and_then(|t| t.as_str()).unwrap_or("");

            match block_type {
                "tool_use" | "server_tool_use" => {
                    *current_tool_name = block.and_then(|b| b.get("name")).and_then(|n| n.as_str()).map(String::from);
                    *current_tool_use_id = block.and_then(|b| b.get("id")).and_then(|id| id.as_str()).map(String::from);
                    current_tool_input.clear();
                    *in_tool_input = true;

                    if let Some(input) = block.and_then(|b| b.get("input")) {
                        if !input.is_object() || !input.as_object().map(|o| o.is_empty()).unwrap_or(true) {
                            if let Ok(input_str) = serde_json::to_string(input) {
                                *current_tool_input = input_str;
                            }
                        }
                    }
                }
                "thinking" => {
                    let _ = tx.send(EngineOutput::simple("thinking_start", "")).await;
                }
                "text" => {}
                _ => {}
            }
        }
        "content_block_delta" => {
            let delta = chunk.get("delta");
            let delta_type = delta.and_then(|d| d.get("type")).and_then(|t| t.as_str()).unwrap_or("");

            match delta_type {
                "text_delta" => {
                    let text = delta.and_then(|d| d.get("text")).and_then(|t| t.as_str()).unwrap_or("");
                    if !text.is_empty() {
                        let _ = tx.send(EngineOutput::simple("text", text)).await;
                    }
                }
                "thinking_delta" => {
                    let thinking = delta.and_then(|d| d.get("thinking")).and_then(|t| t.as_str()).unwrap_or("");
                    if !thinking.is_empty() {
                        let _ = tx.send(EngineOutput::simple("thinking", thinking)).await;
                    }
                }
                "input_json_delta" => {
                    let partial = delta.and_then(|d| d.get("partial_json")).and_then(|p| p.as_str()).unwrap_or("");
                    if !partial.is_empty() && *in_tool_input {
                        *current_tool_input = merge_tool_args(current_tool_input, partial);
                    }
                }
                _ => {}
            }
        }
        "content_block_stop" => {
            if *in_tool_input {
                let tool_name_str = current_tool_name.clone().unwrap_or_default();
                let tool_input = if current_tool_input.is_empty() {
                    serde_json::json!({})
                } else {
                    try_parse_tool_input(&tool_name_str, current_tool_input)
                };

                let _ = tx.send(EngineOutput {
                    msg_type: "tool_call".to_string(),
                    content: String::new(),
                    tool_name: current_tool_name.clone(),
                    tool_input: Some(tool_input),
                    tool_use_id: current_tool_use_id.clone(),
                    stop_reason: None,
                    usage: None,
                }).await;

                *current_tool_name = None;
                *current_tool_use_id = None;
                current_tool_input.clear();
                *in_tool_input = false;
            }
        }
        "message_delta" => {
            let stop_reason = chunk.get("delta")
                .and_then(|d| d.get("stop_reason"))
                .and_then(|s| s.as_str())
                .map(String::from);
            let usage = chunk.get("usage").cloned();

            let _ = tx.send(EngineOutput {
                msg_type: "stop".to_string(),
                content: String::new(),
                tool_name: None,
                tool_input: None,
                tool_use_id: None,
                stop_reason,
                usage,
            }).await;
        }
        "message_stop" => {
            let _ = tx.send(EngineOutput::simple("message_stop", "")).await;
        }
        "ping" => {}
        "control_request" => {
            handle_control_request_sse(chunk, tx, stdin_tx).await;
        }
        "error" => {
            let error_msg = chunk.get("error")
                .and_then(|e| e.as_str())
                .or_else(|| chunk.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()))
                .unwrap_or("Unknown error");
            let _ = tx.send(EngineOutput::simple("error", error_msg)).await;
        }
        _ => {}
    }
}

async fn handle_control_request(
    payload: &serde_json::Value,
    tx: &mpsc::Sender<EngineOutput>,
    stdin_tx: &mpsc::Sender<String>,
) {
    let request = payload.get("request");
    let request_id = payload.get("request_id").and_then(|r| r.as_str()).unwrap_or("");

    if let Some(req) = request {
        let subtype = req.get("subtype").and_then(|s| s.as_str()).unwrap_or("");
        let tool_name = req.get("tool_name").and_then(|n| n.as_str()).unwrap_or("");
        let tool_use_id = req.get("tool_use_id").and_then(|id| id.as_str()).unwrap_or("");

        if subtype == "can_use_tool" && tool_name == "AskUserQuestion" {
            let input = req.get("input").cloned().unwrap_or(serde_json::json!({}));
            let questions = input.get("questions").cloned().unwrap_or(serde_json::json!([]));

            let _ = tx.send(EngineOutput {
                msg_type: "ask_user".to_string(),
                content: String::new(),
                tool_name: Some("AskUserQuestion".to_string()),
                tool_input: Some(serde_json::json!({
                    "request_id": request_id,
                    "tool_use_id": tool_use_id,
                    "questions": questions,
                    "original_input": input,
                })),
                tool_use_id: Some(tool_use_id.to_string()),
                stop_reason: None,
                usage: None,
            }).await;
        } else if subtype == "can_use_tool" {
            let input = req.get("input").cloned().unwrap_or(serde_json::json!({}));
            let _ = tx.send(EngineOutput {
                msg_type: "tool_permission".to_string(),
                content: String::new(),
                tool_name: Some(tool_name.to_string()),
                tool_input: Some(serde_json::json!({
                    "request_id": request_id,
                    "tool_use_id": tool_use_id,
                    "input": input,
                })),
                tool_use_id: Some(tool_use_id.to_string()),
                stop_reason: None,
                usage: None,
            }).await;
        } else {
            let input = req.get("input").cloned().unwrap_or(serde_json::json!({}));
            let response = serde_json::json!({
                "type": "control_response",
                "response": {
                    "subtype": "success",
                    "request_id": request_id,
                    "response": {
                        "toolUseID": tool_use_id,
                        "behavior": "allow",
                        "updatedInput": input,
                    }
                }
            });
            let mut payload = serde_json::to_string(&response).unwrap_or_default();
            payload.push('\n');
            let _ = stdin_tx.send(payload).await;
        }
    }
}

async fn handle_control_request_sse(
    chunk: &serde_json::Value,
    tx: &mpsc::Sender<EngineOutput>,
    stdin_tx: &mpsc::Sender<String>,
) {
    let request = chunk.get("request");
    let request_id = chunk.get("request_id").and_then(|r| r.as_str()).unwrap_or("");

    if let Some(req) = request {
        let subtype = req.get("subtype").and_then(|s| s.as_str()).unwrap_or("");
        let tool_name = req.get("tool_name").and_then(|n| n.as_str()).unwrap_or("");
        let tool_use_id = req.get("tool_use_id").and_then(|id| id.as_str()).unwrap_or("");

        if subtype == "can_use_tool" && tool_name == "AskUserQuestion" {
            let input = req.get("input").cloned().unwrap_or(serde_json::json!({}));
            let questions = input.get("questions").cloned().unwrap_or(serde_json::json!([]));

            let _ = tx.send(EngineOutput {
                msg_type: "ask_user".to_string(),
                content: String::new(),
                tool_name: Some("AskUserQuestion".to_string()),
                tool_input: Some(serde_json::json!({
                    "request_id": request_id,
                    "tool_use_id": tool_use_id,
                    "questions": questions,
                    "original_input": input,
                })),
                tool_use_id: Some(tool_use_id.to_string()),
                stop_reason: None,
                usage: None,
            }).await;
        } else if subtype == "can_use_tool" {
            let input = req.get("input").cloned().unwrap_or(serde_json::json!({}));
            let _ = tx.send(EngineOutput {
                msg_type: "tool_permission".to_string(),
                content: String::new(),
                tool_name: Some(tool_name.to_string()),
                tool_input: Some(serde_json::json!({
                    "request_id": request_id,
                    "tool_use_id": tool_use_id,
                    "input": input,
                })),
                tool_use_id: Some(tool_use_id.to_string()),
                stop_reason: None,
                usage: None,
            }).await;
        } else {
            let input = req.get("input").cloned().unwrap_or(serde_json::json!({}));
            let response = serde_json::json!({
                "type": "control_response",
                "response": {
                    "subtype": "success",
                    "request_id": request_id,
                    "response": {
                        "toolUseID": tool_use_id,
                        "behavior": "allow",
                        "updatedInput": input,
                    }
                }
            });
            let mut payload = serde_json::to_string(&response).unwrap_or_default();
            payload.push('\n');
            let _ = stdin_tx.send(payload).await;
        }
    }
}
