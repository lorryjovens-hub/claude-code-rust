use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/* ── Bridge types ── */

#[derive(Debug, Serialize, Clone)]
pub struct BridgeInstanceInfo {
    pub running: bool,
    pub pid: Option<u32>,
    pub bot_name: Option<String>,
    pub app_id: Option<String>,
    pub version: Option<String>,
    pub started_at: Option<String>,
    pub config_path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FeishuCredentials {
    pub app_id: String,
    pub app_secret: String,
    pub tenant: String,
    pub admin_open_id: Option<String>,
}

fn lark_channel_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".lark-channel")
}

fn processes_path() -> PathBuf {
    lark_channel_dir().join("processes.json")
}

fn config_path() -> PathBuf {
    lark_channel_dir().join("config.json")
}

/* ── Bridge detection ── */

fn read_processes_json() -> Result<Vec<BridgeInstanceInfo>, String> {
    let content = std::fs::read_to_string(processes_path())
        .map_err(|e| format!("Failed to read processes.json: {}", e))?;

    let parsed: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse processes.json: {}", e))?;

    let entries = parsed.get("entries").and_then(|v| v.as_array())
        .ok_or_else(|| "No entries in processes.json".to_string())?;

    Ok(entries.iter().map(|entry| {
        let pid = entry.get("pid").and_then(|v| v.as_u64()).map(|v| v as u32);
        BridgeInstanceInfo {
            running: pid.map(|p| process_alive(p)).unwrap_or(false),
            pid,
            bot_name: entry.get("botName").and_then(|v| v.as_str()).map(String::from),
            app_id: entry.get("appId").and_then(|v| v.as_str()).map(String::from),
            version: entry.get("version").and_then(|v| v.as_str()).map(String::from),
            started_at: entry.get("startedAt").and_then(|v| v.as_str()).map(String::from),
            config_path: entry.get("configPath").and_then(|v| v.as_str()).map(String::from),
        }
    }).collect())
}

fn process_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        std::process::Command::new("kill")
            .arg("-0").arg(pid.to_string())
            .status().map(|s| s.success()).unwrap_or(false)
    }
    #[cfg(windows)]
    {
        std::process::Command::new("tasklist")
            .args(&["/FI", &format!("PID eq {}", pid), "/NH"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
            .unwrap_or(false)
    }
}

/* ── Credential resolution ── */

/// Read bridge config and attempt to resolve the app_secret
/// Uses the bridge's secrets-getter if available, otherwise tries plaintext
fn read_bridge_credentials() -> Result<Option<FeishuCredentials>, String> {
    let config_content = std::fs::read_to_string(config_path())
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let config: serde_json::Value = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    let accounts = config.get("accounts").and_then(|v| v.get("app"))
        .ok_or_else(|| "No 'app' account in config".to_string())?;

    let app_id = accounts.get("id").and_then(|v| v.as_str())
        .ok_or_else(|| "No app id in config".to_string())?.to_string();

    let tenant = accounts.get("tenant").and_then(|v| v.as_str())
        .unwrap_or("feishu").to_string();

    let admin_open_id = config
        .get("preferences")
        .and_then(|p| p.get("access"))
        .and_then(|a| a.get("admins"))
        .and_then(|a| a.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .map(String::from);

    // Try to decrypt secret using bridge's secrets-getter
    let secrets_getter = lark_channel_dir().join("secrets-getter");
    let app_secret = if secrets_getter.exists() {
        let output = std::process::Command::new(&secrets_getter)
            .arg("get")
            .arg(format!("app-{}", app_id))
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .output()
            .ok();
        match output {
            Some(o) if o.status.success() => {
                String::from_utf8_lossy(&o.stdout).trim().to_string()
            }
            _ => return Ok(None),
        }
    } else {
        // Try plaintext secret (some configs store it directly)
        match accounts.get("secret") {
            Some(s) if s.is_string() => s.as_str().unwrap().to_string(),
            _ => return Ok(None),
        }
    };

    Ok(Some(FeishuCredentials {
        app_id,
        app_secret,
        tenant,
        admin_open_id,
    }))
}

/* ── QR code auth ── */

/// Tracks the auth flow state
pub struct AuthFlow {
    pub verification_url: String,
    pub device_code: String,
    pub status: String, // "waiting" | "completed" | "failed"
}

pub static AUTH_FLOW: once_cell::sync::Lazy<Arc<Mutex<Option<AuthFlow>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

/// Launch the bridge registration wizard and capture the QR URL.
/// Returns the verification URL that should be displayed as QR code.
async fn run_registration_wizard() -> Result<String, String> {
    // Run: npx -y lark-channel-bridge@latest run --wizard-only
    // The bridge's wizard will output QR info to stderr
    // We capture the verification_url from the output
    let mut child = std::process::Command::new("npx")
        .args(&["-y", "lark-channel-bridge@latest", "run", "--wizard-only"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start auth wizard: {}", e))?;

    let _pid = child.id();

    // Wait briefly for the QR URL to appear in output
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // Check if child is still running
    match child.try_wait() {
        Ok(Some(status)) => {
            return Err(format!("Auth wizard exited early with status: {}", status));
        }
        Ok(None) => {
            // Still running — we can read PID and user can scan QR
            // The wizard prints to stdout/stderr
            // For now, just track the PID and inform the user
            // Kill after timeout
            tokio::time::sleep(tokio::time::Duration::from_secs(600)).await;
            let _ = child.kill();
            return Err("Auth timeout (10 min)".to_string());
        }
        Err(e) => {
            let _ = child.kill();
            return Err(format!("Error waiting for auth wizard: {}", e));
        }
    }
}

/* ── Tauri Commands ── */

#[tauri::command]
pub async fn bridge_detect() -> Result<Vec<BridgeInstanceInfo>, String> {
    read_processes_json()
}

#[tauri::command]
pub async fn bridge_get_status(id: Option<String>) -> Result<BridgeInstanceInfo, String> {
    let instances = read_processes_json()?;

    if let Some(target_id) = id {
        instances.into_iter().find(|i| {
            i.app_id.as_deref() == Some(&target_id)
                || i.pid.map(|p| p.to_string()) == Some(target_id.clone())
        }).ok_or_else(|| format!("No bridge instance found matching: {}", target_id))
    } else {
        instances.into_iter().next().ok_or_else(|| "No bridge instance found".to_string())
    }
}

#[tauri::command]
pub async fn bridge_start() -> Result<String, String> {
    let child = std::process::Command::new("npx")
        .args(&["-y", "lark-channel-bridge@latest", "start"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start bridge: {}", e))?;

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    Ok(format!("Bridge start initiated (pid: {})", child.id()))
}

#[tauri::command]
pub async fn bridge_stop(id: Option<String>) -> Result<(), String> {
    let instances = read_processes_json()?;
    let instance = if let Some(target_id) = id {
        instances.iter().find(|i| {
            i.app_id.as_deref() == Some(&target_id)
                || i.pid.map(|p| p.to_string()) == Some(target_id.clone())
        })
    } else {
        instances.first()
    };

    match instance {
        Some(inst) => {
            std::process::Command::new("npx")
                .args(&["-y", "lark-channel-bridge@latest", "kill", &inst.pid.map(|p| p.to_string()).unwrap_or_default()])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map_err(|e| format!("Failed to stop bridge: {}", e))?;
            Ok(())
        }
        None => Err("No running bridge instance found".to_string()),
    }
}

/// Read bridge config and return resolved credentials (or null if not available)
#[tauri::command]
pub async fn bridge_get_credentials() -> Result<Option<FeishuCredentials>, String> {
    read_bridge_credentials()
}

/// Launch QR code auth wizard and return the verification URL
#[tauri::command]
pub async fn bridge_start_auth() -> Result<String, String> {
    // Step 1: Try using lark-cli auth login --no-wait --json
    let output = std::process::Command::new("lark-cli")
        .args(&["auth", "login", "--no-wait", "--json"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to start lark-cli auth: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("lark-cli auth failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse lark-cli output: {}", e))?;

    let verification_url = parsed.get("verification_url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "No verification_url in lark-cli output".to_string())?
        .to_string();

    let device_code = parsed.get("device_code")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "No device_code in lark-cli output".to_string())?
        .to_string();

    // Store auth flow state for completion
    let mut flow = AUTH_FLOW.lock().await;
    *flow = Some(AuthFlow {
        verification_url: verification_url.clone(),
        device_code,
        status: "waiting".to_string(),
    });

    Ok(verification_url)
}

/// Poll auth flow status
#[tauri::command]
pub async fn bridge_poll_auth() -> Result<serde_json::Value, String> {
    let flow = AUTH_FLOW.lock().await;
    match flow.as_ref() {
        Some(state) => {
            Ok(serde_json::json!({
                "status": state.status,
                "verification_url": state.verification_url,
            }))
        }
        None => {
            Ok(serde_json::json!({
                "status": "no_auth_in_progress",
            }))
        }
    }
}

/// Complete the auth flow by checking credentials availability
#[tauri::command]
pub async fn bridge_complete_auth() -> Result<Option<FeishuCredentials>, String> {
    // Try to read the credentials from bridge config
    // The lark-cli auth flow writes to the bridge's config
    let creds = read_bridge_credentials()?;
    if creds.is_some() {
        let mut flow = AUTH_FLOW.lock().await;
        if let Some(ref mut f) = *flow {
            f.status = "completed".to_string();
        }
    }
    Ok(creds)
}
