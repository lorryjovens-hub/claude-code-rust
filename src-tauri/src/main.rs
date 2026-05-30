#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod bridge;
mod api;
mod commands;
mod app_studio;
mod engine;

mod tools;
mod research;
mod prompt;
mod mcp;
mod streaming;
mod task;
mod skills;
mod git;
mod config;
mod fs;
mod terminal;
mod process;
mod watcher;
mod clipboard;
mod notification;
mod logger;
mod updater;
mod worktree;
mod ide;
mod analytics;
mod slash_commands;
mod cost_tracker;
mod native_engine;
mod upload;
mod project;
mod computer_use;
mod ask_user;
mod document;
mod sandbox;
mod github;
mod db;
mod multiagent;
mod orchestration;
mod permissions;
mod memory;
mod remotion;
mod diff;
mod scheduler;
mod im_integration;
mod cache;
mod prefetch;
mod metrics;
mod agent_bus;
mod preview_engine;
mod knowledge;
mod secure_storage;

use bridge::BridgeServer;
use native_engine::engine_core::NativeEngine;
use native_engine::provider_manager::ProviderManager;
use native_engine::session_manager::SessionManager;
use mcp::McpServerManager;
use permissions::PermissionManager;
use memory::MemExClient;
use tauri::Manager;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::sync::Mutex;
use tokio::process::Child;

fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, Registry};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false);

    let log_dir = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("logs");
    let _ = std::fs::create_dir_all(&log_dir);

    let file_appender = tracing_appender::rolling::daily(log_dir, "app.log");
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .json();

    let subscriber = Registry::default()
        .with(filter)
        .with(fmt_layer)
        .with(file_layer);

    if std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").is_ok() {
        if let Ok(tracer) = init_opentelemetry() {
            let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
            let subscriber = subscriber.with(otel_layer);
            let _ = tracing::subscriber::set_global_default(subscriber);
            return;
        }
    }

    let _ = tracing::subscriber::set_global_default(subscriber);
}

fn init_opentelemetry() -> Result<opentelemetry_sdk::trace::Tracer, Box<dyn std::error::Error>> {
    use opentelemetry_otlp::WithExportConfig;

    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")?);

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    Ok(tracer)
}

async fn cleanup_all_processes(app_handle: &tauri::AppHandle) {
    tracing::info!(module = "Shutdown", "Cleaning up all child processes...");

    if let Some(mcp_state) = app_handle.try_state::<Arc<Mutex<Option<Arc<Mutex<McpServerManager>>>>>>() {
        if let Some(mcp_manager) = mcp_state.lock().await.clone() {
            let manager = mcp_manager.lock().await;
            if let Err(e) = manager.shutdown_all().await {
                tracing::error!(module = "Shutdown", error = %e);
            }
        }
    }

    if let Some(memex_state) = app_handle.try_state::<Arc<Mutex<Option<Child>>>>() {
        if let Some(mut child) = memex_state.lock().await.take() {
            tracing::info!(module = "Shutdown", "Killing MemEx Python backend...");
            let _ = child.kill().await;
        }
    }

    tracing::info!(module = "Shutdown", "All processes cleaned up.");
}

fn main() {
    init_tracing();

    std::panic::set_hook(Box::new(|info| {
        let location = info.location().map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column())).unwrap_or_else(|| "unknown".to_string());
        let message = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };

        // 收集更详细的崩溃信息
        let backtrace = std::backtrace::Backtrace::capture();

        tracing::error!(
            module = "PANIC",
            location = %location,
            message = %message,
            backtrace = %backtrace,
            "Application panic occurred"
        );

        // 写入崩溃日志到文件
        if let Ok(log_dir) = std::env::current_dir().map(|d| d.join("logs")) {
            let _ = std::fs::create_dir_all(&log_dir);
            let crash_log_path = log_dir.join("crash.log");
            let timestamp = chrono::Utc::now().to_rfc3339();
            let crash_info = format!(
                "[{}] PANIC at {}:\n{}\nBacktrace:\n{}\n\n",
                timestamp, location, message, backtrace
            );
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(crash_log_path)
                .map(|mut f| std::io::Write::write_all(&mut f, crash_info.as_bytes()));
        }
    }));

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_secure_storage::init())
        .manage(Arc::new(Mutex::new(None::<NativeEngine>)))
        .manage(Arc::new(Mutex::new(None::<Arc<Mutex<ProviderManager>>>)))
        .manage(Arc::new(Mutex::new(None::<Arc<Mutex<McpServerManager>>>)))
        .manage(Arc::new(std::sync::Mutex::new(None::<String>)))
        .manage(Arc::new(Mutex::new(None::<Child>)))
        .manage(Arc::new(Mutex::new(None::<SessionManager>)))
        .manage(Arc::new(Mutex::new(None::<Arc<PermissionManager>>)))
        .setup(|app| {
            let data_dir = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
            tracing::info!(module = "App", "Data dir: {:?}", data_dir);

            // Initialise global orchestration config from config/orchestration.toml
            if let Err(e) = crate::config::OrchestrationConfig::init_global() {
                tracing::warn!("Failed to load orchestration.toml, using defaults: {}", e);
            }
            // Initialize SQLite database with persistent storage
            let db_path = data_dir.join("claude_desktop.db");
            let db_manager = match crate::db::DbManager::new(db_path.clone()) {
                Ok(db) => Arc::new(db),
                Err(e) => {
                    tracing::error!(module = "FATAL", error = %e);
                    std::process::exit(1);
                }
            };
            if let Err(e) = db_manager.init() {
                tracing::error!(module = "FATAL", error = %e);
                std::process::exit(1);
            }
            app.manage(db_manager.clone());
            tracing::info!(module = "SQLite", "Database initialized at {:?}", db_path);

            let im_manager = Arc::new(
                crate::im_integration::ImIntegrationManager::new(db_manager.clone())
            );
            tauri::async_runtime::block_on(async {
                if let Err(e) = im_manager.initialize().await {
                    tracing::error!(module = "IM", error = %e);
                }
            });
            app.manage(im_manager);
            tracing::info!(module = "IM", "IM Integration Manager initialized");

            // Initialize Knowledge Base
            let kb_dir = data_dir.join("knowledge");
            let kb = Arc::new(crate::knowledge::KnowledgeBase::new(kb_dir));
            tauri::async_runtime::block_on(async { kb.load().await; });
            app.manage(kb.clone());

            // Initialize Knowledge Flow Engine
            let flow_engine = Arc::new(crate::knowledge::FlowEngine {
                flow: crate::knowledge::KnowledgeFlow::new(kb.clone()),
            });
            app.manage(flow_engine);

            // Initialize Project Intelligence Engine
            let intel_engine = Arc::new(crate::knowledge::IntelEngine {
                intel: crate::knowledge::ProjectIntel::new(kb.clone()),
            });
            app.manage(intel_engine);

            // Initialize GitHub Hub
            let github_hub = Arc::new(crate::knowledge::GitHubHub::new());
            app.manage(github_hub);
            tracing::info!(module = "KB", "Knowledge Base + Flow + Intel + GitHub initialized");

            // Start IM message processing pipeline
            {
                use tokio::sync::mpsc;
                use crate::im_integration::message_router::{UnifiedMessage, MessageType, ReplyPayload};
                use crate::db::DbManager;
                use crate::native_engine::NativeEngine;

                let (msg_tx, mut msg_rx) = mpsc::unbounded_channel::<UnifiedMessage>();
                let im_manager_ref = app.state::<Arc<crate::im_integration::ImIntegrationManager>>().inner().clone();
                let db_for_im = app.state::<Arc<DbManager>>().inner().clone();
                let native_engine_state = app.state::<Arc<Mutex<Option<NativeEngine>>>>().inner().clone();
                let _ = db_for_im;

                tauri::async_runtime::block_on(async {
                    im_manager_ref.set_message_channel(msg_tx).await;
                });

                tauri::async_runtime::spawn(async move {
                    while let Some(msg) = msg_rx.recv().await {
                        tracing::info!(module = "IM_Processor", "Processing {} msg from chat_id={}: {}",
                            msg.platform, msg.chat_id, &msg.content[..msg.content.len().min(80)]);

                        // Only process Feishu messages with NativeEngine
                        if msg.platform != "feishu" {
                            continue;
                        }

                        // Get or create conversation mapping
                        let mapping = match crate::commands::feishu_chat::feishu_get_or_create_conversation_inner(
                            &db_for_im, msg.chat_id.clone(), None
                        ).await {
                            Ok(m) => m,
                            Err(e) => {
                                tracing::error!(module = "IM_Processor", "Failed to create conversation: {}", e);
                                continue;
                            }
                        };
                        let cid = mapping.conversation_id;

                        tracing::info!(module = "IM_Processor", "Feishu msg routed to conv={}", cid);

                        // Lock NativeEngine and send message
                        let mut engine_guard = native_engine_state.lock().await;
                        let engine = match engine_guard.as_mut() {
                            Some(e) => e,
                            None => {
                                tracing::error!(module = "IM_Processor", "NativeEngine not initialized");
                                let _ = im_manager_ref.message_router.send_reply(ReplyPayload {
                                    platform: "feishu".to_string(), chat_id: msg.chat_id.clone(),
                                    content: "❌ AI 引擎未初始化，请先配置模型".to_string(),
                                    message_type: MessageType::Text, thread_id: msg.thread_id.clone(), extra: None,
                                }).await;
                                continue;
                            }
                        };

                        // Build chat request with user's message
                        let chat_req = crate::native_engine::engine_core::ChatRequest {
                            conversation_id: cid.clone(),
                            messages: vec![serde_json::json!({
                                "role": "user",
                                "content": msg.content.clone()
                            })],
                            model: "claude-sonnet-4-6".to_string(),
                            system_prompt: None,
                            max_tokens: None,
                            workspace_path: None,
                            temperature: None,
                            top_p: None,
                            reasoning_mode: None,
                        };

                        // Send initial "thinking" card
                        let card_start = serde_json::json!({
                            "config": { "wide_screen_mode": true },
                            "header": { "title": { "tag": "plain_text", "content": "Claude" }, "template": "blue" },
                            "elements": [
                                { "tag": "markdown", "content": "🤔 *正在思考...*" },
                                { "tag": "note", "elements": [{ "tag": "plain_text", "content": "流式输出中..." }] }
                            ]
                        });
                        let card_str = serde_json::to_string(&card_start).unwrap_or_default();
                        let _ = im_manager_ref.message_router.send_reply(ReplyPayload {
                            platform: "feishu".to_string(), chat_id: msg.chat_id.clone(),
                            content: card_str,
                            message_type: MessageType::Card, thread_id: msg.thread_id.clone(), extra: None,
                        }).await;

                        // Call NativeEngine
                        match engine.send_message(chat_req).await {
                            Ok(mut rx) => {
                                let mut full_text = String::new();
                                let mut total_input_tokens: u64 = 0;
                                let mut total_output_tokens: u64 = 0;
                                let mut model_name = String::new();
                                use tokio::time::{sleep, Duration};
                                let mut last_update = std::time::Instant::now();
                                let min_update_interval = Duration::from_millis(300);

                                loop {
                                    match rx.recv().await {
                                        Some(crate::native_engine::tool_loop::EngineEvent::Text(delta)) => {
                                            full_text.push_str(&delta);
                                            // Rate-limit card updates: at most every 300ms
                                            if last_update.elapsed() >= min_update_interval {
                                                let preview = if full_text.len() > 1500 {
                                                    format!("{}...\n\n_(继续生成中...)_", &full_text[..1500])
                                                } else {
                                                    full_text.clone()
                                                };
                                                let card_update = serde_json::json!({
                                                    "config": { "wide_screen_mode": true },
                                                    "header": { "title": { "tag": "plain_text", "content": "Claude" }, "template": "blue" },
                                                    "elements": [
                                                        { "tag": "markdown", "content": preview },
                                                        { "tag": "note", "elements": [{ "tag": "plain_text", "content": "流式输出中..." }] }
                                                    ]
                                                });
                                                let update_str = serde_json::to_string(&card_update).unwrap_or_default();
                                                let _ = im_manager_ref.message_router.send_reply(ReplyPayload {
                                                    platform: "feishu".to_string(), chat_id: msg.chat_id.clone(),
                                                    content: update_str,
                                                    message_type: MessageType::Card, thread_id: msg.thread_id.clone(), extra: None,
                                                }).await;
                                                last_update = std::time::Instant::now();
                                            }
                                        }
                                        Some(crate::native_engine::tool_loop::EngineEvent::Thinking(_)) => {}
                                        Some(crate::native_engine::tool_loop::EngineEvent::Usage(usage)) => {
                                            // Track tokens from Usage events
                                            if let Some(input) = usage.get("input_tokens").and_then(|v| v.as_u64()) {
                                                total_input_tokens += input;
                                            }
                                            if let Some(output) = usage.get("output_tokens").and_then(|v| v.as_u64()) {
                                                total_output_tokens += output;
                                            }
                                        }
                                        Some(crate::native_engine::tool_loop::EngineEvent::MessageStart { model }) => {
                                            model_name = model;
                                        }
                                        Some(crate::native_engine::tool_loop::EngineEvent::MessageStop { .. }) => {
                                            // Finalize card
                                            let card_final = serde_json::json!({
                                                "config": { "wide_screen_mode": true },
                                                "header": { "title": { "tag": "plain_text", "content": "Claude" }, "template": "blue" },
                                                "elements": [
                                                    { "tag": "markdown", "content": &if full_text.is_empty() { "_(空回复)_" } else { &full_text } }
                                                ]
                                            });
                                            let final_str = serde_json::to_string(&card_final).unwrap_or_default();
                                            let _ = im_manager_ref.message_router.send_reply(ReplyPayload {
                                                platform: "feishu".to_string(), chat_id: msg.chat_id.clone(),
                                                content: final_str,
                                                message_type: MessageType::Card, thread_id: msg.thread_id.clone(), extra: None,
                                            }).await;
                                            // Track tokens consumed in this Feishu conversation
                                            if total_input_tokens > 0 || total_output_tokens > 0 {
                                                let track_body = serde_json::json!({
                                                    "event_type": "tokens_used",
                                                    "properties": {
                                                        "input_tokens": total_input_tokens,
                                                        "output_tokens": total_output_tokens,
                                                        "model": model_name,
                                                        "source": "feishu_bridge"
                                                    }
                                                });
                                                let _ = reqwest::Client::new()
                                                    .post("http://127.0.0.1:30085/api/analytics/track")
                                                    .json(&track_body)
                                                    .timeout(std::time::Duration::from_secs(5))
                                                    .send()
                                                    .await;
                                            }
                                            break;
                                        }
                                        Some(crate::native_engine::tool_loop::EngineEvent::Error(err)) => {
                                            let _ = im_manager_ref.message_router.send_reply(ReplyPayload {
                                                platform: "feishu".to_string(), chat_id: msg.chat_id.clone(),
                                                content: format!("❌ 错误: {}", err),
                                                message_type: MessageType::Text, thread_id: msg.thread_id.clone(), extra: None,
                                            }).await;
                                            break;
                                        }
                                        None => break,
                                        _ => {}
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!(module = "IM_Processor", "NativeEngine send_message failed: {}", e);
                                let _ = im_manager_ref.message_router.send_reply(ReplyPayload {
                                    platform: "feishu".to_string(), chat_id: msg.chat_id.clone(),
                                    content: format!("❌ 引擎调用失败: {}", e),
                                    message_type: MessageType::Text, thread_id: msg.thread_id.clone(), extra: None,
                                }).await;
                            }
                        }
                    }
                });
            }

            // Initialize SessionManager
            let sessions_path = data_dir.join("sessions.json");
            let workspaces_dir = data_dir.join("workspaces");
            let session_mgr = SessionManager::new(sessions_path, workspaces_dir);
            tauri::async_runtime::block_on(async {
                *app.state::<Arc<Mutex<Option<SessionManager>>>>().lock().await = Some(session_mgr);
            });
            tracing::info!(module = "SessionManager", "Initialized");
            let bridge_ready = Arc::new(Notify::new());
            let _bridge_ready_clone = bridge_ready.clone();

            let mcp_config_path = data_dir.join("mcp-servers.json");
            let mcp_manager = Arc::new(Mutex::new(McpServerManager::new(mcp_config_path)));

            {
                let mcp_manager_ref = mcp_manager.clone();
                tauri::async_runtime::block_on(async move {
                    let manager = mcp_manager_ref.lock().await;
                    if let Err(e) = manager.initialize().await {
                        tracing::error!(module = "MCP", error = %e);
                    } else {
                        tracing::info!(module = "MCP", "Initialized successfully");
                    }
                });
            }

            {
                let app_handle = app.handle().clone();
                let mcp_manager_clone = mcp_manager.clone();
                tauri::async_runtime::block_on(async move {
                    *app_handle.state::<Arc<Mutex<Option<Arc<Mutex<McpServerManager>>>>>>().lock().await = Some(mcp_manager_clone);
                });
            }

            let api_key_store = app.state::<Arc<std::sync::Mutex<Option<String>>>>().inner().clone();
            let db_manager_for_bridge = match app.try_state::<Arc<crate::db::DbManager>>() {
                Some(state) => state.inner().clone(),
                None => {
                    tracing::error!(module = "Main", "DbManager not found in state, creating new one for Bridge");
                    let new_db = match crate::db::DbManager::new(db_path.clone()) {
                        Ok(db) => Arc::new(db),
                        Err(e) => {
                            tracing::error!(module = "FATAL", error = %e);
                            std::process::exit(1);
                        }
                    };
                    if let Err(e) = new_db.init() {
                        tracing::error!(module = "FATAL", error = %e);
                        std::process::exit(1);
                    }
                    new_db
                }
            };

            let bridge_ready = Arc::new(tokio::sync::Notify::new());
            let bridge_ready_for_bridge = bridge_ready.clone();
            let bridge_ready_for_window = bridge_ready.clone();

            tauri::async_runtime::spawn(async move {
                let bridge = BridgeServer::new(data_dir, db_manager_for_bridge);
                let api_key = bridge.get_api_key().to_string();
                if let Ok(mut store) = api_key_store.lock() {
                    *store = Some(api_key);
                }
                tracing::info!(module = "Bridge", "Starting server on port 30085...");
                match bridge.start(30085).await {
                    Ok(()) => {
                        bridge_ready_for_bridge.notify_waiters();
                        tracing::info!(module = "Bridge", "Server stopped.");
                    }
                    Err(e) => {
                        bridge_ready_for_bridge.notify_waiters();
                        tracing::error!(module = "Bridge", "Server failed to start: {}", e);
                    }
                }
            });

            if let Some(window) = app.webview_windows().get("main") {
                let window = window.clone();
                tauri::async_runtime::spawn(async move {
                    // Wait briefly for webview to initialize
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    let _ = window.open_devtools();
                });
            }

            let memex_client = MemExClient::new(None);
            let memex_base_url = memex_client.base_url.clone();
            let memex_child_store = app.state::<Arc<Mutex<Option<Child>>>>().inner().clone();

            tauri::async_runtime::spawn(async move {
                tracing::info!(module = "MemEx", "Attempting to start Python backend on port 8765...");

                let is_running = reqwest::Client::new()
                    .get(format!("{}/health", memex_base_url))
                    .send()
                    .await
                    .map(|r| r.status().is_success())
                    .unwrap_or(false);

                if is_running {
                    tracing::info!(module = "MemEx", "Backend already running on port 8765");
                    return;
                }

                let memex_script = std::env::current_dir()
                    .unwrap_or_default()
                    .join("memex")
                    .join("api.py");

                if !memex_script.exists() {
                    tracing::warn!(module = "MemEx", "Python backend script not found at {:?}. Memory features will be disabled.", memex_script);
                    return;
                }

                let python_cmd = if cfg!(windows) { "python" } else { "python3" };
                match tokio::process::Command::new(python_cmd)
                    .arg(&memex_script)
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .kill_on_drop(true)
                    .spawn()
                {
                    Ok(child) => {
                        tracing::info!(module = "MemEx", "Python backend started on port 8765");
                        memex_child_store.lock().await.replace(child);
                        for i in 0..10 {
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                            if reqwest::Client::new()
                                .get(format!("{}/health", memex_base_url))
                                .send()
                                .await
                                .map(|r| r.status().is_success())
                                .unwrap_or(false)
                            {
                                tracing::info!(module = "MemEx", "Backend healthy after {} seconds", i + 1);
                                return;
                            }
                        }
                        tracing::warn!(module = "MemEx", "Backend health check timeout, memory features may be limited");
                    }
                    Err(e) => {
                        tracing::error!(module = "MemEx", error = %e);
                    }
                }
            });

            let db_manager_for_scheduler = match app.try_state::<Arc<crate::db::DbManager>>() {
                Some(state) => state.inner().clone(),
                None => {
                    tracing::error!(module = "Scheduler", "DbManager not found, scheduler disabled");
                    return Ok(());
                }
            };

            tauri::async_runtime::spawn(async move {
                tracing::info!(module = "Scheduler", "Task scheduler started, checking every 30 seconds");
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;

                    let now = chrono::Utc::now().to_rfc3339();
                    let now_dt = chrono::Utc::now();
                    let db = db_manager_for_scheduler.clone();

                    let due_tasks = match tokio::task::spawn_blocking(move || {
                        db.with_conn(|conn| {
                            crate::db::task_repo::get_due_tasks(conn, &now)
                        })
                    })
                    .await
                    {
                        Ok(Ok(Ok(tasks))) => tasks,
                        Ok(Ok(Err(e))) | Ok(Err(e)) => {
                            tracing::error!(module = "Scheduler", error = %e);
                            continue;
                        }
                        Err(e) => {
                            tracing::error!(module = "Scheduler", error = %e);
                            continue;
                        }
                    };

                    for task in &due_tasks {
                        tracing::info!(module = "Scheduler", "Executing due task: id={}, name={}, type={}",
                            task.id, task.name, task.task_type
                        );

                        let task_id = task.id.clone();
                        let task_name = task.name.clone();
                        let task_type = task.task_type.clone();
                        let task_config = task.task_config.clone();
                        let task_cron = task.cron_expression.clone();
                        let now_str = chrono::Utc::now().to_rfc3339();
                        let db2 = db_manager_for_scheduler.clone();

                        let (_id1, now1, task_name1, task_type1, task_config1) = (
                            task_id.clone(),
                            now_str.clone(),
                            task_name.clone(),
                            task_type.clone(),
                            task_config.clone(),
                        );

                        let execution_output = tokio::task::spawn_blocking(move || {
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            format!(
                                "Task '{}' (type: {}) executed at {}. Config: {}",
                                task_name1, task_type1, now1, task_config1
                            )
                        })
                        .await
                        .unwrap_or_else(|e| format!("Spawn error: {}", e));

                        let next_run = crate::task::cron::calc_next_run(&task_cron, &now_dt)
                            .unwrap_or_default();

                        let (id2, now2) = (task_id.clone(), now_str.clone());
                        let next_run_opt = if next_run.is_empty() { None } else { Some(next_run.clone()) };

                        match tokio::task::spawn_blocking(move || {
                            db2.with_conn(|conn| {
                                crate::db::task_repo::update_task_run_result(
                                    conn,
                                    &id2,
                                    &now2,
                                    "success",
                                    Some(&execution_output),
                                    next_run_opt.as_deref(),
                                )
                            })
                        })
                        .await
                        {
                            Ok(Ok(Ok(()))) => {
                                tracing::info!(module = "Scheduler", "Task '{}' completed, next run at: {}",
                                    task_name, next_run
                                );
                            }
                            Ok(Ok(Err(e))) | Ok(Err(e)) => {
                                tracing::error!(module = "Scheduler", "Failed to update task '{}' result: {}",
                                    task_id, e
                                );
                            }
                            Err(e) => {
                                tracing::error!(module = "Scheduler", error = %e);
                            }
                        }
                    }

                    if due_tasks.is_empty() {
                        continue;
                    }
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let app_handle = window.app_handle().clone();
                let window_clone = window.clone();
                tauri::async_runtime::spawn(async move {
                    cleanup_all_processes(&app_handle).await;
                    window_clone.destroy().unwrap_or_else(|e| {
                        tracing::error!(module = "Shutdown", error = %e);
                    });
                });
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_platform,
            commands::select_directory,
            commands::show_item_in_folder,
            commands::open_folder,
            commands::open_external_url,
            commands::resize_window,
            commands::show_main_window,
            commands::export_workspace,
            commands::get_system_status,
            commands::chat_send,
            commands::chat_stream,
            commands::execute_tool,
            commands::get_app_path,
            commands::check_update,
            commands::install_update,
            commands::list_slash_commands,
            commands::search_slash_commands,
            commands::get_slash_command_categories,
            commands::get_cost_summary,
            commands::get_all_session_costs,
            commands::native_engine_init,
            commands::native_chat,
            commands::native_create_conversation,
            commands::native_list_conversations,
            commands::native_delete_conversation,
            commands::native_get_messages,
            commands::native_list_providers,
            commands::native_update_provider,
            commands::native_delete_provider,
            commands::mcp_list_servers,
            commands::mcp_start_server,
            commands::mcp_stop_server,
            commands::mcp_restart_server,
            commands::mcp_add_server,
            commands::mcp_update_server,
            commands::mcp_remove_server,
            commands::mcp_toggle_server,
            commands::mcp_list_tools,
            commands::get_bridge_api_key,
            // Diff commands
            commands::generate_diff,
            commands::apply_diff,
            commands::reject_diff,
            commands::get_code_diffs,
            commands::generate_h5_token,
            commands::revoke_h5_token,
            commands::list_h5_tokens,
            commands::validate_h5_token,
            commands::cleanup_expired_h5_tokens,
            // Remotion commands
            remotion::remotion_create_project,
            remotion::remotion_install_deps,
            remotion::remotion_start_studio,
            remotion::remotion_render,
            remotion::remotion_list_compositions,
            remotion::remotion_scan_projects,
            remotion::remotion_open_in_editor,
            remotion::remotion_still,
            // Scheduled task commands
            commands::create_scheduled_task,
            commands::list_scheduled_tasks,
            commands::update_scheduled_task,
            commands::delete_scheduled_task,
            commands::execute_task_now,
            commands::get_task_runs,
            // Permission approval commands
            commands::request_permission_approval,
            commands::approve_permission,
            commands::reject_permission,
            commands::get_pending_approvals,
            commands::always_allow_permission,
            commands::get_dangerous_tools_list,
            commands::im_connect_platform,
            commands::im_disconnect_platform,
            commands::im_list_connections,
            commands::im_send_message,
            commands::im_get_config,
            commands::im_update_config,
            // New IM commands
            commands::im_generate_qr_code,
            commands::im_check_auth_status,
            commands::im_get_connection_status,
            commands::im_get_message_stats,
            commands::im_set_permission_mode,
            commands::im_get_permission_mode,
            commands::im_generate_pairing_code,
            commands::im_get_pending_pairing_requests,
            commands::im_approve_pairing_request,
            commands::im_reject_pairing_request,
            commands::im_get_error_logs,
            // Lark Bridge commands
            commands::bridge::bridge_detect,
            commands::bridge::bridge_get_status,
            commands::bridge::bridge_start,
            commands::bridge::bridge_stop,
            commands::bridge::bridge_get_credentials,
            commands::bridge::bridge_start_auth,
            commands::bridge::bridge_poll_auth,
            commands::bridge::bridge_complete_auth,
            commands::add_always_allow_rule,
            commands::remove_always_allow_rule,
            commands::get_always_allow_rules,
            commands::set_permission_mode,
            // Computer Use commands
            commands::computer_use_screenshot,
            commands::computer_use_mouse_click,
            commands::computer_use_keyboard_type,
            commands::computer_use_keyboard_key,
            commands::computer_use_mouse_scroll,
            commands::computer_use_get_screen_info,
            // Secure Storage commands
            secure_storage::secure_get_api_key,
            secure_storage::secure_set_api_key,
            secure_storage::secure_delete_api_key,
            secure_storage::secure_get_gateway_user,
            secure_storage::secure_set_gateway_user,
            secure_storage::secure_delete_gateway_user,
            secure_storage::secure_get_gateway_quota,
            secure_storage::secure_set_gateway_quota,
            secure_storage::secure_delete_gateway_quota,
            secure_storage::secure_get_auth_token,
            secure_storage::secure_set_auth_token,
            secure_storage::secure_delete_auth_token,
            secure_storage::secure_clear_all,
            // Feishu multi-window chat commands
            commands::feishu_chat::feishu_get_or_create_conversation,
            commands::feishu_chat::feishu_list_conversations,
            commands::feishu_chat::feishu_delete_conversation,
            // Knowledge Base commands
            crate::knowledge::kb_list,
            crate::knowledge::kb_get,
            crate::knowledge::kb_search,
            crate::knowledge::kb_add,
            crate::knowledge::kb_delete,
            crate::knowledge::kb_graph,
            crate::knowledge::kb_import,
            // Knowledge Flow commands
            crate::knowledge::kb_ingest_conversation,
            crate::knowledge::kb_prepare_context,
            crate::knowledge::kb_digest,
            crate::knowledge::kb_patterns,
            // Project Intelligence commands
            crate::knowledge::intel_analyze,
            crate::knowledge::intel_find_fusions,
            crate::knowledge::intel_generate_skill,
            // GitHub Hub commands
            crate::knowledge::gh_trending,
            crate::knowledge::gh_search,
            crate::knowledge::gh_user_repos,
            crate::knowledge::gh_set_token,
            crate::knowledge::gh_watch,
            crate::knowledge::gh_get_watched,
            crate::knowledge::gh_oauth_url,
            // App Studio commands
            commands::app_studio_generate_project,
            commands::get_context_size,
        ]);

    #[cfg(mobile)]
    {
        builder = builder
            .plugin(tauri_plugin_haptics::init())
            .plugin(tauri_plugin_barcode_scanner::init());
    }

    if let Err(e) = builder.run(tauri::generate_context!()) {
        tracing::error!(module = "FATAL", error = %e);
        std::process::exit(1);
    }
}
