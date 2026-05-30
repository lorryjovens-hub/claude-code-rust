use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::{broadcast, Mutex};
use tokio::time::{timeout, Duration};

/// Allowed commands that can be spawned through the process manager.
/// This is an allowlist — commands not in this list are rejected,
/// providing defense-in-depth alongside the dangerous command blocklist.
const ALLOWED_COMMANDS: &[&str] = &[
    "git", "node", "npm", "npx", "pnpm", "yarn", "bun",
    "python", "python3", "pip", "pip3",
    "cargo", "rustc", "rustup",
    "make", "cmake", "gcc", "g++", "clang",
    "deno",
    "go",
    "rustfmt", "clippy-driver",
    "docker", "docker-compose",
    "code", "code-insiders",
    "ls", "cat", "head", "tail", "grep", "find", "sort", "uniq", "wc",
    "echo", "pwd", "which", "mkdir", "touch", "cp", "mv", "rm",
    "bash", "sh", "zsh", "pwsh", "powershell",
    "curl", "wget",
    "jq", "yq",
    "ps", "top",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub command: String,
    pub cwd: String,
    pub started_at: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessOutput {
    pub pid: u32,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
}

pub struct ProcessManager {
    processes: Arc<Mutex<HashMap<u32, ProcessInfo>>>,
    output_txs: Arc<Mutex<HashMap<u32, broadcast::Sender<String>>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            output_txs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn spawn(
        &self,
        command: &str,
        cwd: Option<&str>,
        env_vars: Option<HashMap<String, String>>,
    ) -> Result<ProcessInfo> {
        let cwd = cwd.unwrap_or(".");
        let is_win = cfg!(windows);

        // Allowlist check: reject commands not in the approved list
        let cmd_name = command.split_whitespace().next().unwrap_or("");
        if !cmd_name.is_empty() && !ALLOWED_COMMANDS.contains(&cmd_name) {
            return Err(anyhow::anyhow!(
                "Command '{}' is not in the allowed commands list", cmd_name
            ));
        }

        if crate::tools::is_dangerous_command(command) {
            return Err(anyhow::anyhow!(
                "Command blocked by security filter: dangerous command detected"
            ));
        }

        let (shell, arg) = if is_win {
            ("powershell", "-Command")
        } else {
            ("bash", "-c")
        };

        let mut cmd = Command::new(shell);
        cmd.arg(arg).arg(command);
        cmd.current_dir(cwd);
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        cmd.kill_on_drop(true);

        if let Some(env) = env_vars {
            cmd.envs(env);
        }

        let child = cmd.spawn()?;
        let pid = child.id().unwrap_or(0);
        let started_at = std::time::SystemTime::now();

        let info = ProcessInfo {
            pid,
            name: command.split_whitespace().next().unwrap_or("unknown").to_string(),
            command: command.to_string(),
            cwd: cwd.to_string(),
            started_at,
        };

        self.processes.lock().await.insert(pid, info.clone());
        
        Ok(info)
    }

    pub async fn wait(&self, pid: u32, timeout_secs: u64) -> Result<ProcessOutput> {
        let processes = self.processes.lock().await;
        let info = processes.get(&pid).cloned();
        drop(processes);

        if info.is_none() {
            return Err(anyhow::anyhow!("Process {} not found", pid));
        }

        let _info = info.expect("checked above");
        let start = std::time::Instant::now();

        let duration = Duration::from_secs(timeout_secs);
        let _result = timeout(duration, async {
            let mut cmd = Command::new("wait");
            cmd.arg(pid.to_string());
            cmd.output().await
        }).await;

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(ProcessOutput {
            pid,
            stdout: String::new(),
            stderr: String::new(),
            exit_code: None,
            duration_ms,
        })
    }

    pub async fn kill(&self, pid: u32) -> Result<()> {
        // Step 1: Check process exists and remove from tracking — lock is dropped before await
        {
            let mut processes = self.processes.lock().await;
            if !processes.contains_key(&pid) {
                return Err(anyhow::anyhow!("Process {} not found", pid));
            }
            processes.remove(&pid);
        }
        self.output_txs.lock().await.remove(&pid);

        // Step 2: Async operations — no MutexGuard held
        #[cfg(windows)]
        {
            Command::new("taskkill")
                .args(["/F", "/PID", &pid.to_string()])
                .output()
                .await?;
        }

        #[cfg(not(windows))]
        {
            Command::new("kill")
                .args(["-9", &pid.to_string()])
                .output()
                .await?;
        }

        Ok(())
    }

    pub async fn list_processes(&self) -> Vec<ProcessInfo> {
        self.processes.lock().await.values().cloned().collect()
    }

    pub async fn get_process(&self, pid: u32) -> Option<ProcessInfo> {
        self.processes.lock().await.get(&pid).cloned()
    }

    pub async fn subscribe_output(&self, pid: u32) -> Option<broadcast::Receiver<String>> {
        self.output_txs.lock().await.get(&pid).map(|tx| tx.subscribe())
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;