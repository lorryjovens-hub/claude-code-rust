use anyhow::{Context, Result};
use portable_pty::{native_pty_system, CommandBuilder, PtySize, MasterPty};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex as StdMutex};
use std::thread;
use tokio::sync::{broadcast, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtySession {
    pub id: String,
    pub cwd: String,
    pub pid: u32,
    pub shell: String,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtyOutput {
    pub session_id: String,
    pub data: String,
    pub is_stderr: bool,
}

struct ActivePty {
    child_pid: u32,
    master: Box<dyn MasterPty + Send>,
    writer: StdMutex<Box<dyn Write + Send>>,
}

pub struct PtyManager {
    sessions: Arc<Mutex<HashMap<String, PtySession>>>,
    outputs: Arc<Mutex<HashMap<String, broadcast::Sender<PtyOutput>>>>,
    active_ptys: Arc<Mutex<HashMap<String, ActivePty>>>,
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            outputs: Arc::new(Mutex::new(HashMap::new())),
            active_ptys: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn create_session(&self, cwd: Option<String>, shell: Option<String>) -> Result<PtySession> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let cwd = cwd.unwrap_or_else(|| {
            std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string())
        });

        let shell_cmd = if let Some(ref s) = shell {
            s.clone()
        } else if cfg!(windows) {
            resolve_windows_shell()
        } else {
            resolve_unix_shell()
        };

        let shell_name = shell_cmd.clone();

        let pty_system = native_pty_system();
        let default_size = PtySize {
            rows: 30,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pair = pty_system
            .openpty(default_size)
            .context("Failed to open PTY")?;

        let mut cmd = if cfg!(windows) {
            let mut c = CommandBuilder::new(&shell_cmd);
            c.cwd(std::path::Path::new(&cwd));
            c
        } else {
            let mut c = CommandBuilder::new(&shell_cmd);
            c.cwd(std::path::Path::new(&cwd));
            c.env("TERM", "xterm-256color");
            c.env("LANG", "en_US.UTF-8");
            c.env("COLORTERM", "truecolor");
            c
        };

        let child = pair.slave.spawn_command(cmd).context("Failed to spawn shell")?;
        let child_pid = child.process_id().unwrap_or(0);

        let (tx, _) = broadcast::channel::<PtyOutput>(256);

        let mut reader = pair
            .master
            .try_clone_reader()
            .context("Failed to clone PTY reader")?;

        let writer: Box<dyn Write + Send> = Box::new(
            pair.master
                .take_writer()
                .context("Failed to take PTY writer")?,
        );

        let tx_clone = tx.clone();
        let sid = session_id.clone();

        thread::spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buf[..n]).to_string();
                        if tx_clone
                            .send(PtyOutput {
                                session_id: sid.clone(),
                                data,
                                is_stderr: false,
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let session = PtySession {
            id: session_id.clone(),
            cwd: cwd.clone(),
            pid: child_pid,
            shell: shell_name,
            cols: default_size.cols,
            rows: default_size.rows,
        };

        let active = ActivePty {
            child_pid,
            master: pair.master,
            writer: StdMutex::new(writer),
        };

        self.sessions.lock().await.insert(session_id.clone(), session.clone());
        self.outputs.lock().await.insert(session_id.clone(), tx);
        self.active_ptys.lock().await.insert(session_id.clone(), active);

        Ok(session)
    }

    pub async fn write_input(&self, session_id: &str, data: &str) -> Result<()> {
        let ptys = self.active_ptys.lock().await;
        let active = ptys.get(session_id).context("Session not found")?;

        let mut writer = active.writer.lock().unwrap();
        writer.write_all(data.as_bytes())?;
        writer.flush()?;
        Ok(())
    }

    pub async fn resize(&self, session_id: &str, cols: u16, rows: u16) -> Result<()> {
        let ptys = self.active_ptys.lock().await;
        let active = ptys.get(session_id).context("Session not found")?;

        active.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut sessions = self.sessions.lock().await;
        if let Some(s) = sessions.get_mut(session_id) {
            s.cols = cols;
            s.rows = rows;
        }
        Ok(())
    }

    pub async fn close_session(&self, session_id: &str) -> Result<()> {
        let pid = {
            let ptys = self.active_ptys.lock().await;
            ptys.get(session_id).map(|a| a.child_pid)
        };

        if let Some(pid) = pid {
            if pid > 0 {
                kill_process(pid);
            }
        }

        self.active_ptys.lock().await.remove(session_id);
        self.sessions.lock().await.remove(session_id);
        self.outputs.lock().await.remove(session_id);
        Ok(())
    }

    pub async fn list_sessions(&self) -> Vec<PtySession> {
        self.sessions.lock().await.values().cloned().collect()
    }

    pub async fn subscribe(&self, session_id: &str) -> Option<broadcast::Receiver<PtyOutput>> {
        self.outputs
            .lock()
            .await
            .get(session_id)
            .map(|tx| tx.subscribe())
    }
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(windows)]
fn kill_process(pid: u32) {
    use std::process::Command;
    let _ = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/F", "/T"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

#[cfg(not(windows))]
fn kill_process(pid: u32) {
    use std::process::Command;
    let _ = Command::new("kill")
        .args(["-9", &pid.to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

fn resolve_windows_shell() -> String {
    for candidate in &["pwsh.exe", "powershell.exe", "cmd.exe"] {
        if std::process::Command::new(candidate)
            .arg(if *candidate == "cmd.exe" { "/C" } else { "-Command" })
            .arg("exit 0")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok()
        {
            return candidate.to_string();
        }
    }
    std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
}

fn resolve_unix_shell() -> String {
    if let Ok(shell) = std::env::var("SHELL") {
        if !shell.is_empty() {
            return shell;
        }
    }
    for candidate in &["/bin/zsh", "/bin/bash", "/bin/sh"] {
        if std::path::Path::new(candidate).exists() {
            return candidate.to_string();
        }
    }
    "/bin/sh".to_string()
}

pub async fn execute_bash_command(
    command: &str,
    cwd: Option<&str>,
    timeout_secs: u64,
    env_vars: Option<HashMap<String, String>>,
) -> Result<String> {
    let cwd = cwd.unwrap_or(".");
    let is_win = cfg!(windows);

    let mut cmd = if is_win {
        let mut c = tokio::process::Command::new("powershell");
        c.args(["-Command", command]);
        c
    } else {
        let mut c = tokio::process::Command::new("bash");
        c.args(["-c", command]);
        c
    };

    cmd.current_dir(cwd)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);

    if let Some(env) = env_vars {
        let envs: Vec<(String, String)> = env
            .into_iter()
            .chain([
                ("LANG".to_string(), "en_US.UTF-8".to_string()),
                ("TERM".to_string(), "xterm-256color".to_string()),
            ])
            .collect();
        cmd.envs(envs);
    } else {
        cmd.env("LANG", "en_US.UTF-8");
        cmd.env("TERM", "xterm-256color");
    }

    let timeout = tokio::time::Duration::from_secs(timeout_secs);
    let result = tokio::time::timeout(timeout, cmd.output()).await;

    match result {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            let mut result = stdout.to_string();
            if !stderr.is_empty() {
                result.push_str(&format!("\nSTDERR: {}", stderr));
            }

            if !output.status.success() && stdout.is_empty() {
                result = format!(
                    "Command exited with code {:?}: {}",
                    output.status.code(),
                    stderr
                );
            }

            Ok(result)
        }
        Ok(Err(e)) => Err(anyhow::anyhow!("Failed to execute command: {}", e)),
        Err(_) => Err(anyhow::anyhow!(
            "Command timed out after {} seconds",
            timeout_secs
        )),
    }
}