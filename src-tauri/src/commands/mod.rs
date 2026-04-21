use serde::Serialize;
use tauri::{AppHandle, Manager};

#[derive(Serialize)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
    pub is_electron: bool,
}

#[tauri::command]
pub async fn get_platform() -> Result<PlatformInfo, String> {
    Ok(PlatformInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        is_electron: false,
    })
}

#[tauri::command]
pub async fn get_app_path(app: AppHandle) -> Result<String, String> {
    let path = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn select_directory(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::{DialogExt, FilePath};
    use tokio::sync::oneshot;

    let (tx, rx) = oneshot::channel::<Option<FilePath>>();

    #[cfg(not(mobile))]
    {
        app.dialog().file().pick_folder(move |dir| {
            let _ = tx.send(dir);
        });
    }

    #[cfg(mobile)]
    {
        app.dialog().file().pick_file(move |file| {
            let _ = tx.send(file);
        });
    }

    let dir = rx.await.map_err(|e| e.to_string())?;
    Ok(dir.map(|p| p.to_string()))
}

#[tauri::command]
pub async fn show_item_in_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", path))
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(
                std::path::Path::new(&path)
                    .parent()
                    .unwrap_or(std::path::Path::new(".")),
            )
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn open_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn open_external_url(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resize_window(app: AppHandle, width: f64, height: f64) -> Result<(), String> {
    #[cfg(not(mobile))]
    {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window
                .set_size(tauri::LogicalSize::new(width, height));
        }
    }
    let _ = (width, height);
    Ok(())
}

#[tauri::command]
pub async fn export_workspace(
    app: AppHandle,
    _workspace_id: String,
    context_markdown: String,
    default_filename: String,
) -> Result<String, String> {
    let download_dir = app.path().download_dir().map_err(|e| e.to_string())?;
    let output_path = download_dir.join(&default_filename);

    std::fs::write(&output_path, context_markdown).map_err(|e| e.to_string())?;

    Ok(output_path.to_string_lossy().to_string())
}

#[derive(Serialize)]
pub struct SystemStatusResult {
    pub platform: String,
    pub git_bash: GitBashStatusResult,
}

#[derive(Serialize)]
pub struct GitBashStatusResult {
    pub required: bool,
    pub found: bool,
    pub path: Option<String>,
}

#[tauri::command]
pub async fn get_system_status() -> Result<SystemStatusResult, String> {
    let platform = std::env::consts::OS.to_string();
    let git_bash_path = find_git_bash();

    Ok(SystemStatusResult {
        platform,
        git_bash: GitBashStatusResult {
            required: cfg!(target_os = "windows"),
            found: git_bash_path.is_some(),
            path: git_bash_path,
        },
    })
}

fn find_git_bash() -> Option<String> {
    if cfg!(target_os = "windows") {
        let candidates = [
            r"C:\Program Files\Git\bin\bash.exe",
            r"C:\Program Files (x86)\Git\bin\bash.exe",
        ];
        for path in &candidates {
            if std::path::Path::new(path).exists() {
                return Some(path.to_string());
            }
        }
    }
    None
}

#[tauri::command]
pub async fn chat_send(
    _conversation_id: String,
    messages: Vec<serde_json::Value>,
    model: String,
    user_mode: Option<String>,
    env_token: Option<String>,
    env_base_url: Option<String>,
) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;
    let api_url = resolve_api_url(&user_mode, &env_base_url.unwrap_or_default());

    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "max_tokens": 8192,
        "stream": false,
    });

    let mut request_builder = client
        .post(&api_url)
        .header("Content-Type", "application/json")
        .json(&body);

    if let Some(token) = env_token {
        if !token.is_empty() {
            if api_url.contains("anthropic") {
                request_builder = request_builder
                    .header("x-api-key", &token)
                    .header("anthropic-version", "2023-06-01");
            } else {
                request_builder = request_builder.bearer_auth(&token);
            }
        }
    }

    let response = request_builder.send().await.map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("API error {}: {}", status, text));
    }

    response.json::<serde_json::Value>().await.map_err(|e| e.to_string())
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
        Some("clawparrot") => "https://clawparrot.com/api/v1/messages".to_string(),
        _ => "https://api.anthropic.com/v1/messages".to_string(),
    }
}

#[tauri::command]
pub async fn chat_stream() -> Result<String, String> {
    Ok("streaming_placeholder".to_string())
}

#[tauri::command]
pub async fn execute_tool(
    name: String,
    input: serde_json::Value,
    cwd: Option<String>,
) -> Result<serde_json::Value, String> {
    let cwd = cwd.unwrap_or_else(|| ".".to_string());
    crate::tools::execute_tool(&name, input, &cwd).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_update(_app: AppHandle) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({ "has_update": false }))
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    app.restart();
}
