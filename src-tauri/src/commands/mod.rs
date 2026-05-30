use crate::computer_use::{ComputerUseManager, ComputerAction, MouseButton};
use crate::db::DbManager;
use crate::db::diff_repo::CodeDiffRow;
use crate::db::h5_repo::H5TokenRow;
use crate::db::permission_repo::{PermissionApprovalRow, AlwaysAllowRuleRow};
use crate::db::task_repo::{ScheduledTaskRow, TaskRunRow};
use crate::diff::{DiffResult, generate_diff as diff_generate, apply_diff_to_content};
use crate::im_integration::{
    ImIntegrationManager, ImPlatformConfig, ImConnectionInfo, ImConnectionStatusResult,
    ImMessageStatsResult, ImErrorLogInfo,
};
use crate::im_integration::permission_manager::{PermissionMode as ImPermissionMode, UserPermission};
use crate::mcp::{McpServerConfig, McpServerStatus, McpTool};
use crate::native_engine::engine_core::ChatRequest as NativeChatRequest;
use crate::native_engine::provider_manager::{Provider, ProviderManager};
use crate::native_engine::session_manager::{Conversation, Message, SessionManager};
use crate::permissions::{DangerousTool, PermissionMode};
use crate::slash_commands::{SlashCommand, SlashCommandRegistry};
use crate::cost_tracker::{CostSummary, CostTracker, SessionCost};

pub mod bridge;
pub mod feishu_chat;
pub use bridge::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

// Bug #16 fix: Cache SlashCommandRegistry to avoid recreating on every call
static SLASH_COMMAND_REGISTRY: std::sync::OnceLock<SlashCommandRegistry> = std::sync::OnceLock::new();

fn get_slash_command_registry() -> &'static SlashCommandRegistry {
    SLASH_COMMAND_REGISTRY.get_or_init(SlashCommandRegistry::new)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub platform: String,
    pub version: String,
}

#[tauri::command]
pub fn get_platform() -> String {
    std::env::consts::OS.to_string()
}

#[tauri::command]
pub async fn select_directory() -> Result<Option<String>, String> {
    
    // This is a placeholder - actual implementation would use the dialog plugin
    Ok(None)
}

#[tauri::command]
pub fn show_item_in_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .args(["/select,", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path])
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
pub fn open_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
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
pub async fn resize_window(window: tauri::Window, width: f64, height: f64) -> Result<(), String> {
    window.set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn show_main_window(app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn export_workspace(app_handle: tauri::AppHandle) -> Result<String, String> {
    let data_dir = app_handle.path().app_data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."));
    let workspace_dir = data_dir.join("workspaces");
    Ok(workspace_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_system_status() -> SystemStatus {
    SystemStatus {
        platform: std::env::consts::OS.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[tauri::command]
pub async fn chat_send(
    _app_handle: tauri::AppHandle,
    _request: crate::native_engine::engine_core::ChatRequest,
) -> Result<serde_json::Value, String> {
    // Placeholder - actual implementation would use the bridge
    Err("chat_send not implemented in commands".to_string())
}

#[tauri::command]
pub async fn chat_stream(
    _app_handle: tauri::AppHandle,
    _request: crate::native_engine::engine_core::ChatRequest,
) -> Result<serde_json::Value, String> {
    // Placeholder - actual implementation would use the bridge
    Err("chat_stream not implemented in commands".to_string())
}

#[tauri::command]
pub async fn execute_tool(
    name: String,
    input: serde_json::Value,
    cwd: Option<String>,
) -> Result<serde_json::Value, String> {
    let cwd_str = cwd.unwrap_or_else(|| std::env::current_dir().unwrap_or_default().to_string_lossy().to_string());
    match crate::tools::execute_tool(&name, input, &cwd_str) {
        Ok(result) => Ok(result),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn get_app_path(app_handle: tauri::AppHandle) -> Result<String, String> {
    app_handle.path().app_data_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_update(app_handle: tauri::AppHandle) -> Result<Option<crate::updater::UpdateInfo>, String> {
    let data_dir = app_handle.path().app_data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."));
    let updater = crate::updater::AutoUpdater::new(
        "https://api.claude-desktop.app/updates",
        env!("CARGO_PKG_VERSION"),
        data_dir.join("updates"),
    );
    updater.check_for_updates().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn install_update(_app_handle: tauri::AppHandle) -> Result<(), String> {
    Err("install_update not implemented".to_string())
}

#[tauri::command]
pub fn list_slash_commands() -> Vec<SlashCommand> {
    let registry = get_slash_command_registry();
    registry.list_commands().into_iter().cloned().collect()
}

#[tauri::command]
pub fn search_slash_commands(query: String) -> Vec<SlashCommand> {
    let registry = get_slash_command_registry();
    registry.search_commands(&query).into_iter().cloned().collect()
}

#[tauri::command]
pub fn get_slash_command_categories() -> Vec<String> {
    let registry = get_slash_command_registry();
    registry.get_categories()
}

#[tauri::command]
pub async fn get_cost_summary(
    app_handle: tauri::AppHandle,
    conversation_id: String,
) -> Result<CostSummary, String> {
    // Bug #18 fix: Try to use app-level CostTracker from state, fall back to creating one
    let tracker = match app_handle.try_state::<Arc<CostTracker>>() {
        Some(state) => state.inner().clone(),
        None => {
            let data_dir = app_handle.path().app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            Arc::new(CostTracker::new(data_dir.join("costs")))
        }
    };
    Ok(tracker.get_conversation_cost(&conversation_id).await)
}

#[tauri::command]
pub async fn get_all_session_costs(
    app_handle: tauri::AppHandle,
) -> Result<Vec<SessionCost>, String> {
    // Bug #18 fix: Try to use app-level CostTracker from state, fall back to creating one
    let tracker = match app_handle.try_state::<Arc<CostTracker>>() {
        Some(state) => state.inner().clone(),
        None => {
            let data_dir = app_handle.path().app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            Arc::new(CostTracker::new(data_dir.join("costs")))
        }
    };
    Ok(tracker.get_all_sessions().await)
}

#[tauri::command]
pub async fn native_engine_init(
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let engine_state = app_handle.state::<Arc<Mutex<Option<crate::native_engine::NativeEngine>>>>();
    let mut engine_guard = engine_state.lock().await;
    if engine_guard.is_none() {
        let data_dir = app_handle.path().app_data_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."));
        let provider_manager = Arc::new(Mutex::new(ProviderManager::new(
            data_dir.join("providers.json"),
        )));
        let db_manager = match app_handle.try_state::<Arc<DbManager>>() {
            Some(state) => state.inner().clone(),
            None => return Err("DbManager not found".to_string()),
        };
        let permission_manager = Arc::new(crate::permissions::PermissionManager::new(
            Arc::new(crate::permissions::AuditLogger::new(1000)),
        ));
        let engine = crate::native_engine::NativeEngine::new(
            provider_manager,
            db_manager,
            data_dir.join("workspaces"),
            permission_manager,
            Arc::new(crate::cache::FileCache::new()),
        );
        *engine_guard = Some(engine);
    }
    Ok(())
}

#[tauri::command]
pub async fn native_chat(
    _app_handle: tauri::AppHandle,
    _request: NativeChatRequest,
) -> Result<serde_json::Value, String> {
    Err("native_chat not implemented in commands".to_string())
}

#[tauri::command]
pub async fn native_create_conversation(
    app_handle: tauri::AppHandle,
    model: String,
    title: Option<String>,
) -> Result<Conversation, String> {
    let session_state = app_handle.state::<Arc<Mutex<Option<SessionManager>>>>();
    let mut session_guard = session_state.lock().await;
    if let Some(session_mgr) = session_guard.as_mut() {
        let conversation = session_mgr.create_conversation(model, title, false);
        if let Err(e) = session_mgr.save() {
            tracing::warn!(module = "Commands", "Failed to save session: {}", e);
        }
        Ok(conversation)
    } else {
        Err("SessionManager not initialized".to_string())
    }
}

#[tauri::command]
pub async fn native_list_conversations(
    app_handle: tauri::AppHandle,
) -> Result<Vec<Conversation>, String> {
    let session_state = app_handle.state::<Arc<Mutex<Option<SessionManager>>>>();
    let session_guard = session_state.lock().await;
    if let Some(session_mgr) = session_guard.as_ref() {
        Ok(session_mgr.list_conversations().to_vec())
    } else {
        Err("SessionManager not initialized".to_string())
    }
}

#[tauri::command]
pub async fn native_delete_conversation(
    app_handle: tauri::AppHandle,
    conversation_id: String,
) -> Result<(), String> {
    let session_state = app_handle.state::<Arc<Mutex<Option<SessionManager>>>>();
    let mut session_guard = session_state.lock().await;
    if let Some(session_mgr) = session_guard.as_mut() {
        session_mgr.delete_conversation(&conversation_id);
        if let Err(e) = session_mgr.save() {
            tracing::warn!(module = "Commands", "Failed to save session: {}", e);
        }
        Ok(())
    } else {
        Err("SessionManager not initialized".to_string())
    }
}

#[tauri::command]
pub async fn native_get_messages(
    app_handle: tauri::AppHandle,
    conversation_id: String,
) -> Result<Vec<Message>, String> {
    let session_state = app_handle.state::<Arc<Mutex<Option<SessionManager>>>>();
    let session_guard = session_state.lock().await;
    if let Some(session_mgr) = session_guard.as_ref() {
        Ok(session_mgr.get_messages(&conversation_id).into_iter().cloned().collect())
    } else {
        Err("SessionManager not initialized".to_string())
    }
}

#[tauri::command]
pub async fn native_list_providers(
    app_handle: tauri::AppHandle,
) -> Result<Vec<Provider>, String> {
    let data_dir = app_handle.path().app_data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."));
    let provider_manager = ProviderManager::new(data_dir.join("providers.json"));
    Ok(provider_manager.list_providers().to_vec())
}

#[tauri::command]
pub async fn native_update_provider(
    app_handle: tauri::AppHandle,
    provider: Provider,
) -> Result<(), String> {
    let data_dir = app_handle.path().app_data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."));
    let mut provider_manager = ProviderManager::new(data_dir.join("providers.json"));
    provider_manager.update_provider(&provider.id.clone(), provider);
    Ok(())
}

#[tauri::command]
pub async fn native_delete_provider(
    app_handle: tauri::AppHandle,
    provider_id: String,
) -> Result<(), String> {
    let data_dir = app_handle.path().app_data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."));
    let mut provider_manager = ProviderManager::new(data_dir.join("providers.json"));
    provider_manager.delete_provider(&provider_id);
    Ok(())
}

#[tauri::command]
pub async fn mcp_list_servers(
    app_handle: tauri::AppHandle,
) -> Result<Vec<McpServerStatus>, String> {
    let mcp_state = app_handle.state::<Arc<Mutex<Option<Arc<Mutex<crate::mcp::McpServerManager>>>>>>();
    let mcp_guard = mcp_state.lock().await;
    if let Some(mcp_manager) = mcp_guard.as_ref() {
        let manager = mcp_manager.lock().await;
        Ok(manager.list_servers().await)
    } else {
        Ok(Vec::new())
    }
}

#[tauri::command]
pub async fn mcp_start_server(
    app_handle: tauri::AppHandle,
    id: String,
) -> Result<(), String> {
    let mcp_state = app_handle.state::<Arc<Mutex<Option<Arc<Mutex<crate::mcp::McpServerManager>>>>>>();
    let mcp_guard = mcp_state.lock().await;
    if let Some(mcp_manager) = mcp_guard.as_ref() {
        let manager = mcp_manager.lock().await;
        manager.start_server(&id).await.map_err(|e| e.to_string())
    } else {
        Err("MCP manager not initialized".to_string())
    }
}

#[tauri::command]
pub async fn mcp_stop_server(
    app_handle: tauri::AppHandle,
    id: String,
) -> Result<(), String> {
    let mcp_state = app_handle.state::<Arc<Mutex<Option<Arc<Mutex<crate::mcp::McpServerManager>>>>>>();
    let mcp_guard = mcp_state.lock().await;
    if let Some(mcp_manager) = mcp_guard.as_ref() {
        let manager = mcp_manager.lock().await;
        manager.stop_server(&id).await.map_err(|e| e.to_string())
    } else {
        Err("MCP manager not initialized".to_string())
    }
}

#[tauri::command]
pub async fn mcp_restart_server(
    app_handle: tauri::AppHandle,
    id: String,
) -> Result<(), String> {
    let mcp_state = app_handle.state::<Arc<Mutex<Option<Arc<Mutex<crate::mcp::McpServerManager>>>>>>();
    let mcp_guard = mcp_state.lock().await;
    if let Some(mcp_manager) = mcp_guard.as_ref() {
        let manager = mcp_manager.lock().await;
        manager.restart_server(&id).await.map_err(|e| e.to_string())
    } else {
        Err("MCP manager not initialized".to_string())
    }
}

#[tauri::command]
pub async fn mcp_add_server(
    app_handle: tauri::AppHandle,
    config: McpServerConfig,
) -> Result<(), String> {
    let mcp_state = app_handle.state::<Arc<Mutex<Option<Arc<Mutex<crate::mcp::McpServerManager>>>>>>();
    let mcp_guard = mcp_state.lock().await;
    if let Some(mcp_manager) = mcp_guard.as_ref() {
        let manager = mcp_manager.lock().await;
        manager.add_server(config).await.map_err(|e| e.to_string())
    } else {
        Err("MCP manager not initialized".to_string())
    }
}

#[tauri::command]
pub async fn mcp_update_server(
    app_handle: tauri::AppHandle,
    id: String,
    config: McpServerConfig,
) -> Result<(), String> {
    let mcp_state = app_handle.state::<Arc<Mutex<Option<Arc<Mutex<crate::mcp::McpServerManager>>>>>>();
    let mcp_guard = mcp_state.lock().await;
    if let Some(mcp_manager) = mcp_guard.as_ref() {
        let manager = mcp_manager.lock().await;
        manager.update_server(&id, config).await.map_err(|e| e.to_string())
    } else {
        Err("MCP manager not initialized".to_string())
    }
}

#[tauri::command]
pub async fn mcp_remove_server(
    app_handle: tauri::AppHandle,
    id: String,
) -> Result<(), String> {
    let mcp_state = app_handle.state::<Arc<Mutex<Option<Arc<Mutex<crate::mcp::McpServerManager>>>>>>();
    let mcp_guard = mcp_state.lock().await;
    if let Some(mcp_manager) = mcp_guard.as_ref() {
        let manager = mcp_manager.lock().await;
        manager.remove_server(&id).await.map_err(|e| e.to_string())
    } else {
        Err("MCP manager not initialized".to_string())
    }
}

#[tauri::command]
pub async fn mcp_toggle_server(
    app_handle: tauri::AppHandle,
    id: String,
    enabled: Option<bool>,
) -> Result<(), String> {
    let mcp_state = app_handle.state::<Arc<Mutex<Option<Arc<Mutex<crate::mcp::McpServerManager>>>>>>();
    let mcp_guard = mcp_state.lock().await;
    if let Some(mcp_manager) = mcp_guard.as_ref() {
        let manager = mcp_manager.lock().await;
        if let Some(enabled_val) = enabled {
            manager.set_server_enabled(&id, enabled_val).await.map_err(|e| e.to_string())
        } else {
            manager.toggle_server(&id).await.map_err(|e| e.to_string())
        }
    } else {
        Err("MCP manager not initialized".to_string())
    }
}

#[tauri::command]
pub async fn mcp_list_tools(
    app_handle: tauri::AppHandle,
) -> Result<Vec<McpTool>, String> {
    let mcp_state = app_handle.state::<Arc<Mutex<Option<Arc<Mutex<crate::mcp::McpServerManager>>>>>>();
    let mcp_guard = mcp_state.lock().await;
    if let Some(mcp_manager) = mcp_guard.as_ref() {
        let manager = mcp_manager.lock().await;
        Ok(manager.get_all_tools().await)
    } else {
        Ok(Vec::new())
    }
}

#[tauri::command]
pub fn get_bridge_api_key(app_handle: tauri::AppHandle) -> Result<String, String> {
    let api_key_store = app_handle.state::<Arc<std::sync::Mutex<Option<String>>>>();
    let guard = api_key_store.lock().map_err(|e| e.to_string())?;
    guard.clone().ok_or_else(|| "API key not available".to_string())
}

#[tauri::command]
pub fn generate_diff(original: String, modified: String) -> DiffResult {
    diff_generate(&original, &modified)
}

#[tauri::command]
pub fn apply_diff(content: String, diff_text: String) -> Result<String, String> {
    apply_diff_to_content(&content, &diff_text)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reject_diff(
    app_handle: tauri::AppHandle,
    diff_id: String,
) -> Result<(), String> {
    let db_manager = match app_handle.try_state::<Arc<DbManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("DbManager not found".to_string()),
    };
    tokio::task::spawn_blocking(move || {
        db_manager.with_conn(|conn| {
            crate::db::diff_repo::update_diff_status(conn, &diff_id, "rejected", None)
        })?
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_code_diffs(
    app_handle: tauri::AppHandle,
    conversation_id: String,
) -> Result<Vec<CodeDiffRow>, String> {
    let db_manager = match app_handle.try_state::<Arc<DbManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("DbManager not found".to_string()),
    };
    tokio::task::spawn_blocking(move || {
        db_manager.with_conn(|conn| {
            crate::db::diff_repo::get_diffs_by_conversation(conn, &conversation_id)
        })?
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn generate_h5_token(
    app_handle: tauri::AppHandle,
    conversation_id: String,
    expires_in_hours: Option<i64>,
) -> Result<String, String> {
    let db_manager = match app_handle.try_state::<Arc<DbManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("DbManager not found".to_string()),
    };
    let id = uuid::Uuid::new_v4().to_string();
    let token = format!("h5_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
    let hours = expires_in_hours.unwrap_or(24);
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(hours);
    let now = chrono::Utc::now().to_rfc3339();
    let expires_str = expires_at.to_rfc3339();
    let token_clone = token.clone();

    tokio::task::spawn_blocking(move || {
        db_manager.with_conn(|conn| {
            crate::db::h5_repo::insert_h5_token(conn, &id, &token_clone, &conversation_id, &expires_str, &now)
        })?
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(token)
}

#[tauri::command]
pub async fn revoke_h5_token(
    app_handle: tauri::AppHandle,
    token_id: String,
) -> Result<(), String> {
    let db_manager = match app_handle.try_state::<Arc<DbManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("DbManager not found".to_string()),
    };
    tokio::task::spawn_blocking(move || {
        db_manager.with_conn(|conn| {
            crate::db::h5_repo::revoke_h5_token(conn, &token_id)
        })?
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_h5_tokens(
    app_handle: tauri::AppHandle,
    conversation_id: String,
) -> Result<Vec<H5TokenRow>, String> {
    let db_manager = match app_handle.try_state::<Arc<DbManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("DbManager not found".to_string()),
    };
    tokio::task::spawn_blocking(move || {
        db_manager.with_conn(|conn| {
            crate::db::h5_repo::list_h5_tokens_by_conversation(conn, &conversation_id)
        })?
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn validate_h5_token(
    app_handle: tauri::AppHandle,
    token: String,
) -> Result<Option<H5TokenRow>, String> {
    let db_manager = match app_handle.try_state::<Arc<DbManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("DbManager not found".to_string()),
    };
    tokio::task::spawn_blocking(move || {
        db_manager.with_conn(|conn| {
            crate::db::h5_repo::validate_h5_token(conn, &token)
        })?
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cleanup_expired_h5_tokens(
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let db_manager = match app_handle.try_state::<Arc<DbManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("DbManager not found".to_string()),
    };
    tokio::task::spawn_blocking(move || {
        db_manager.with_conn(|conn| {
            crate::db::h5_repo::cleanup_expired_tokens(conn)
        })?
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_scheduled_task(
    app_handle: tauri::AppHandle,
    name: String,
    description: Option<String>,
    cron_expression: String,
    task_type: String,
    task_config: String,
    conversation_id: Option<String>,
    is_enabled: bool,
) -> Result<ScheduledTaskRow, String> {
    let db_manager = match app_handle.try_state::<Arc<DbManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("DbManager not found".to_string()),
    };
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let next_run = crate::task::cron::calc_next_run(&cron_expression, &chrono::Utc::now())
        .unwrap_or_default();
    let next_run_opt = if next_run.is_empty() { None } else { Some(next_run) };

    let id_clone = id.clone();
    let name_clone = name.clone();
    let description_clone = description.clone();
    let cron_clone = cron_expression.clone();
    let task_type_clone = task_type.clone();
    let task_config_clone = task_config.clone();
    let conversation_id_clone = conversation_id.clone();
    let next_run_opt_clone = next_run_opt.clone();
    let now_clone = now.clone();

    tokio::task::spawn_blocking(move || {
        db_manager.with_conn(|conn| {
            crate::db::task_repo::insert_scheduled_task(
                conn, &id_clone, &name_clone, description_clone.as_deref(), &cron_clone,
                &task_type_clone, &task_config_clone, conversation_id_clone.as_deref(),
                is_enabled, next_run_opt_clone.as_deref(), &now_clone, &now_clone,
            )
        })?
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(ScheduledTaskRow {
        id,
        name,
        description,
        cron_expression,
        task_type,
        task_config,
        conversation_id,
        is_enabled,
        last_run_at: None,
        last_run_status: None,
        last_run_output: None,
        next_run_at: next_run_opt,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub async fn list_scheduled_tasks(
    app_handle: tauri::AppHandle,
) -> Result<Vec<ScheduledTaskRow>, String> {
    let db_manager = match app_handle.try_state::<Arc<DbManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("DbManager not found".to_string()),
    };
    tokio::task::spawn_blocking(move || {
        db_manager.with_conn(|conn| {
            crate::db::task_repo::list_scheduled_tasks(conn)
        })?
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_scheduled_task(
    app_handle: tauri::AppHandle,
    task_id: String,
    name: Option<String>,
    description: Option<String>,
    cron_expression: Option<String>,
    task_type: Option<String>,
    task_config: Option<String>,
    conversation_id: Option<String>,
    is_enabled: Option<bool>,
) -> Result<(), String> {
    let db_manager = match app_handle.try_state::<Arc<DbManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("DbManager not found".to_string()),
    };
    let now = chrono::Utc::now().to_rfc3339();
    let next_run_opt = if let Some(ref cron) = cron_expression {
        let next = crate::task::cron::calc_next_run(cron, &chrono::Utc::now()).unwrap_or_default();
        if next.is_empty() { None } else { Some(next) }
    } else {
        None
    };

    tokio::task::spawn_blocking(move || {
        db_manager.with_conn(|conn| {
            crate::db::task_repo::update_scheduled_task(
                conn, &task_id,
                name.as_deref(), description.as_deref(),
                cron_expression.as_deref(), task_type.as_deref(),
                task_config.as_deref(), conversation_id.as_deref(),
                is_enabled, next_run_opt.as_deref(), &now,
            )
        })?
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_scheduled_task(
    app_handle: tauri::AppHandle,
    task_id: String,
) -> Result<(), String> {
    let db_manager = match app_handle.try_state::<Arc<DbManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("DbManager not found".to_string()),
    };
    tokio::task::spawn_blocking(move || {
        db_manager.with_conn(|conn| {
            crate::db::task_repo::delete_scheduled_task(conn, &task_id)
        })?
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn execute_task_now(
    app_handle: tauri::AppHandle,
    task_id: String,
) -> Result<String, String> {
    let db_manager = match app_handle.try_state::<Arc<DbManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("DbManager not found".to_string()),
    };
    let task = tokio::task::spawn_blocking({
        let db = db_manager.clone();
        move || {
            db.with_conn(|conn| {
                crate::db::task_repo::get_scheduled_task(conn, &task_id)
            })?
        }
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    if let Some(task) = task {
        let scheduler = crate::scheduler::ScheduledTaskScheduler::new(db_manager);
        let result = scheduler.execute_task(&task).await;
        Ok(format!("Task executed: success={}, output={:?}, error={:?}",
            result.success, result.output, result.error))
    } else {
        Err("Task not found".to_string())
    }
}

#[tauri::command]
pub async fn get_task_runs(
    app_handle: tauri::AppHandle,
    task_id: String,
    limit: Option<usize>,
) -> Result<Vec<TaskRunRow>, String> {
    let db_manager = match app_handle.try_state::<Arc<DbManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("DbManager not found".to_string()),
    };
    let limit = limit.unwrap_or(50);
    tokio::task::spawn_blocking(move || {
        db_manager.with_conn(|conn| {
            crate::db::task_repo::get_task_runs(conn, &task_id, limit)
        })?
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn request_permission_approval(
    app_handle: tauri::AppHandle,
    conversation_id: String,
    message_id: String,
    tool_name: String,
    action: String,
    risk_level: String,
) -> Result<String, String> {
    let db_manager = match app_handle.try_state::<Arc<DbManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("DbManager not found".to_string()),
    };
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let id_clone = id.clone();

    tokio::task::spawn_blocking(move || {
        db_manager.with_conn(|conn| {
            crate::db::permission_repo::insert_permission_approval(
                conn, &id_clone, &conversation_id, &message_id,
                &tool_name, &action, &risk_level, "pending", &now,
            )
        })?
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(id)
}

#[tauri::command]
pub async fn approve_permission(
    app_handle: tauri::AppHandle,
    approval_id: String,
) -> Result<(), String> {
    let db_manager = match app_handle.try_state::<Arc<DbManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("DbManager not found".to_string()),
    };
    let now = chrono::Utc::now().to_rfc3339();

    tokio::task::spawn_blocking(move || {
        db_manager.with_conn(|conn| {
            crate::db::permission_repo::update_approval_status(
                conn, &approval_id, "approved", Some("approved"), None, &now,
            )
        })?
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reject_permission(
    app_handle: tauri::AppHandle,
    approval_id: String,
    reason: Option<String>,
) -> Result<(), String> {
    let db_manager = match app_handle.try_state::<Arc<DbManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("DbManager not found".to_string()),
    };
    let now = chrono::Utc::now().to_rfc3339();

    tokio::task::spawn_blocking(move || {
        db_manager.with_conn(|conn| {
            crate::db::permission_repo::update_approval_status(
                conn, &approval_id, "rejected", Some("rejected"), reason.as_deref(), &now,
            )
        })?
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_pending_approvals(
    app_handle: tauri::AppHandle,
    conversation_id: String,
) -> Result<Vec<PermissionApprovalRow>, String> {
    let db_manager = match app_handle.try_state::<Arc<DbManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("DbManager not found".to_string()),
    };
    tokio::task::spawn_blocking(move || {
        db_manager.with_conn(|conn| {
            crate::db::permission_repo::get_pending_approvals(conn, &conversation_id)
        })?
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn always_allow_permission(
    app_handle: tauri::AppHandle,
    tool_name: String,
    action: String,
) -> Result<(), String> {
    let db_manager = match app_handle.try_state::<Arc<DbManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("DbManager not found".to_string()),
    };
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let rule_pattern = format!("{}:{}", tool_name, action);

    tokio::task::spawn_blocking(move || {
        db_manager.with_conn(|conn| {
            crate::db::permission_repo::insert_always_allow_rule(
                conn, &id, &rule_pattern, "combined", &now, &now,
            )
        })?
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_dangerous_tools_list() -> Vec<DangerousTool> {
    crate::permissions::get_dangerous_tools()
}

#[tauri::command]
pub async fn im_connect_platform(
    app_handle: tauri::AppHandle,
    platform: String,
    config: ImPlatformConfig,
) -> Result<ImConnectionInfo, String> {
    let im_manager = match app_handle.try_state::<Arc<ImIntegrationManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("ImIntegrationManager not found".to_string()),
    };
    im_manager.connect_platform(&platform, config).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn im_disconnect_platform(
    app_handle: tauri::AppHandle,
    platform: String,
) -> Result<(), String> {
    let im_manager = match app_handle.try_state::<Arc<ImIntegrationManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("ImIntegrationManager not found".to_string()),
    };
    im_manager.disconnect_platform(&platform).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn im_list_connections(
    app_handle: tauri::AppHandle,
) -> Result<Vec<ImConnectionInfo>, String> {
    let im_manager = match app_handle.try_state::<Arc<ImIntegrationManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("ImIntegrationManager not found".to_string()),
    };
    Ok(im_manager.list_connections().await)
}

#[tauri::command]
pub async fn im_send_message(
    app_handle: tauri::AppHandle,
    platform: String,
    chat_id: String,
    message: String,
) -> Result<(), String> {
    let im_manager = match app_handle.try_state::<Arc<ImIntegrationManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("ImIntegrationManager not found".to_string()),
    };
    im_manager.send_message(&platform, &chat_id, &message).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn im_get_config(
    app_handle: tauri::AppHandle,
    platform: String,
) -> Result<Option<ImConnectionInfo>, String> {
    let im_manager = match app_handle.try_state::<Arc<ImIntegrationManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("ImIntegrationManager not found".to_string()),
    };
    Ok(im_manager.get_connection(&platform).await)
}

#[tauri::command]
pub async fn im_update_config(
    app_handle: tauri::AppHandle,
    platform: String,
    config: ImPlatformConfig,
) -> Result<ImConnectionInfo, String> {
    let im_manager = match app_handle.try_state::<Arc<ImIntegrationManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("ImIntegrationManager not found".to_string()),
    };
    im_manager.connect_platform(&platform, config).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn im_generate_qr_code(
    app_handle: tauri::AppHandle,
    platform: String,
) -> Result<String, String> {
    let im_manager = match app_handle.try_state::<Arc<ImIntegrationManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("ImIntegrationManager not found".to_string()),
    };
    im_manager.generate_qr_code_url(&platform).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn im_check_auth_status(
    app_handle: tauri::AppHandle,
    platform: String,
) -> Result<(bool, String), String> {
    let im_manager = match app_handle.try_state::<Arc<ImIntegrationManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("ImIntegrationManager not found".to_string()),
    };
    im_manager.check_auth_status(&platform).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn im_get_connection_status(
    app_handle: tauri::AppHandle,
    platform: String,
) -> Result<ImConnectionStatusResult, String> {
    let im_manager = match app_handle.try_state::<Arc<ImIntegrationManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("ImIntegrationManager not found".to_string()),
    };
    im_manager.get_connection_status(&platform).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn im_get_message_stats(
    app_handle: tauri::AppHandle,
    platform: Option<String>,
) -> Result<ImMessageStatsResult, String> {
    let im_manager = match app_handle.try_state::<Arc<ImIntegrationManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("ImIntegrationManager not found".to_string()),
    };
    let platform_ref = platform.as_deref();
    im_manager.get_message_stats(platform_ref).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn im_set_permission_mode(
    app_handle: tauri::AppHandle,
    platform: String,
    mode: String,
) -> Result<(), String> {
    let im_manager = match app_handle.try_state::<Arc<ImIntegrationManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("ImIntegrationManager not found".to_string()),
    };
    let permission_mode = ImPermissionMode::from_str(&mode)
        .ok_or_else(|| format!("Invalid permission mode: {}", mode))?;
    im_manager.set_permission_mode(&platform, permission_mode).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn im_get_permission_mode(
    app_handle: tauri::AppHandle,
    platform: String,
) -> Result<String, String> {
    let im_manager = match app_handle.try_state::<Arc<ImIntegrationManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("ImIntegrationManager not found".to_string()),
    };
    let mode = im_manager.get_permission_mode(&platform).await;
    Ok(mode.as_str().to_string())
}

#[tauri::command]
pub async fn im_generate_pairing_code(
    app_handle: tauri::AppHandle,
    platform: String,
    user_id: String,
) -> Result<String, String> {
    let im_manager = match app_handle.try_state::<Arc<ImIntegrationManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("ImIntegrationManager not found".to_string()),
    };
    im_manager.generate_pairing_code(&platform, &user_id).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn im_get_pending_pairing_requests(
    app_handle: tauri::AppHandle,
    platform: String,
) -> Result<Vec<UserPermission>, String> {
    let im_manager = match app_handle.try_state::<Arc<ImIntegrationManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("ImIntegrationManager not found".to_string()),
    };
    im_manager.get_pending_pairing_requests(&platform).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn im_approve_pairing_request(
    app_handle: tauri::AppHandle,
    platform: String,
    user_id: String,
) -> Result<(), String> {
    let im_manager = match app_handle.try_state::<Arc<ImIntegrationManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("ImIntegrationManager not found".to_string()),
    };
    im_manager.approve_pairing_request(&platform, &user_id).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn im_reject_pairing_request(
    app_handle: tauri::AppHandle,
    platform: String,
    user_id: String,
) -> Result<(), String> {
    let im_manager = match app_handle.try_state::<Arc<ImIntegrationManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("ImIntegrationManager not found".to_string()),
    };
    im_manager.reject_pairing_request(&platform, &user_id).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn im_get_error_logs(
    app_handle: tauri::AppHandle,
    platform: Option<String>,
) -> Result<Vec<ImErrorLogInfo>, String> {
    let im_manager = match app_handle.try_state::<Arc<ImIntegrationManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("ImIntegrationManager not found".to_string()),
    };
    let platform_ref = platform.as_deref();
    im_manager.get_error_logs(platform_ref).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_always_allow_rule(
    app_handle: tauri::AppHandle,
    rule_pattern: String,
    rule_type: String,
) -> Result<(), String> {
    let db_manager = match app_handle.try_state::<Arc<DbManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("DbManager not found".to_string()),
    };
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    tokio::task::spawn_blocking(move || {
        db_manager.with_conn(|conn| {
            crate::db::permission_repo::insert_always_allow_rule(
                conn, &id, &rule_pattern, &rule_type, &now, &now,
            )
        })?
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_always_allow_rule(
    app_handle: tauri::AppHandle,
    rule_id: String,
) -> Result<(), String> {
    let db_manager = match app_handle.try_state::<Arc<DbManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("DbManager not found".to_string()),
    };
    tokio::task::spawn_blocking(move || {
        db_manager.with_conn(|conn| {
            crate::db::permission_repo::delete_always_allow_rule(conn, &rule_id)
        })?
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_always_allow_rules(
    app_handle: tauri::AppHandle,
) -> Result<Vec<AlwaysAllowRuleRow>, String> {
    let db_manager = match app_handle.try_state::<Arc<DbManager>>() {
        Some(state) => state.inner().clone(),
        None => return Err("DbManager not found".to_string()),
    };
    tokio::task::spawn_blocking(move || {
        db_manager.with_conn(|conn| {
            crate::db::permission_repo::get_always_allow_rules(conn)
        })?
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_permission_mode(
    app_handle: tauri::AppHandle,
    mode: String,
) -> Result<(), String> {
    let permission_state = app_handle.state::<Arc<Mutex<Option<Arc<crate::permissions::PermissionManager>>>>>();
    let permission_guard = permission_state.lock().await;
    if let Some(perm_mgr) = permission_guard.as_ref() {
        let mode_enum = PermissionMode::from_str(&mode);
        perm_mgr.set_mode(mode_enum);
        Ok(())
    } else {
        Err("PermissionManager not initialized".to_string())
    }
}

#[tauri::command]
pub async fn computer_use_screenshot() -> Result<String, String> {
    crate::computer_use::take_screenshot_powershell()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn computer_use_mouse_click(
    x: i32,
    y: i32,
    button: Option<String>,
) -> Result<serde_json::Value, String> {
    let config = crate::computer_use::ComputerUseConfig::default();
    let manager = ComputerUseManager::new(config);
    let mouse_button = match button.as_deref() {
        Some("right") => MouseButton::Right,
        Some("middle") => MouseButton::Middle,
        _ => MouseButton::Left,
    };
    let action = crate::computer_use::action_helpers::mouse_click(x, y, mouse_button);
    let result = manager.execute_action(action).await
        .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "success": result.success,
        "screenshot": result.screenshot,
        "error": result.error,
    }))
}

#[tauri::command]
pub async fn computer_use_keyboard_type(
    text: String,
) -> Result<serde_json::Value, String> {
    let config = crate::computer_use::ComputerUseConfig::default();
    let manager = ComputerUseManager::new(config);
    let action = crate::computer_use::action_helpers::type_text(&text);
    let result = manager.execute_action(action).await
        .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "success": result.success,
        "screenshot": result.screenshot,
        "error": result.error,
    }))
}

#[tauri::command]
pub async fn computer_use_keyboard_key(
    key: String,
) -> Result<serde_json::Value, String> {
    let config = crate::computer_use::ComputerUseConfig::default();
    let manager = ComputerUseManager::new(config);
    let action = crate::computer_use::action_helpers::key_press(&key);
    let result = manager.execute_action(action).await
        .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "success": result.success,
        "screenshot": result.screenshot,
        "error": result.error,
    }))
}

#[tauri::command]
pub async fn computer_use_mouse_scroll(
    scroll_y: i32,
    scroll_x: Option<i32>,
) -> Result<serde_json::Value, String> {
    let config = crate::computer_use::ComputerUseConfig::default();
    let manager = ComputerUseManager::new(config);
    let action = ComputerAction {
        action_type: crate::computer_use::ComputerActionType::MouseScroll,
        coordinate: None,
        button: None,
        key: None,
        text: None,
        scroll_y: Some(scroll_y),
        scroll_x,
        duration_ms: None,
    };
    let result = manager.execute_action(action).await
        .map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "success": result.success,
        "screenshot": result.screenshot,
        "error": result.error,
    }))
}

#[tauri::command]
pub async fn computer_use_get_screen_info() -> Result<serde_json::Value, String> {
    let config = crate::computer_use::ComputerUseConfig::default();
    let manager = ComputerUseManager::new(config);
    let info = manager.get_screen_info();
    Ok(serde_json::json!({
        "width": info.width,
        "height": info.height,
        "scale_factor": info.scale_factor,
    }))
}

#[tauri::command]
pub async fn app_studio_generate_project(
    spec: crate::app_studio::AppProjectSpec,
) -> Result<crate::app_studio::ProjectStructure, String> {
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("claude-desktop")
        .join("app-studio");
    let studio = crate::app_studio::AppStudio::new(data_dir);
    studio.generate_project(&spec)
}

#[tauri::command]
pub async fn get_context_size(conversation_id: String) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:30085/api/conversations/{}/context-size", conversation_id);
    let resp = client.get(&url).send().await.map_err(|e| format!("Bridge error: {}", e))?;
    let data: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;
    Ok(data)
}
