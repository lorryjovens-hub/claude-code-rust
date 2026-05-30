pub mod routes;
pub mod interceptor;

use crate::clipboard::ClipboardManager;
use crate::config::{AppConfig, ConfigManager};
use crate::cost_tracker::CostTracker;
use crate::db::DbManager;
use crate::engine::EnginePool;
use crate::fs::FileOperations;
use crate::git::GitIntegration;
use crate::logger::Logger;
use crate::mcp::{McpServerManager, McpServerConfig};
use crate::memory::{MemExClient, MemoryConfig, MemorySearchRequest, MemoryIngestRequest, MemoryItem, MemoryStats, ContextManager};
use crate::native_engine::NativeEngine;
use crate::native_engine::provider_manager::ProviderManager;
use crate::notification::NotificationManager;
use crate::permissions::{AuditLogger, PermissionManager};
use crate::preview_engine::PreviewEngine;
use crate::commands::SystemStatus;
use crate::process::ProcessManager;
use crate::research::ResearchEvent;
use crate::orchestration::{MultiAgentOrchestrator, OrchestratorConfigFile, AgentStreamEvent};
use crate::skills::SkillsManager;
use crate::streaming::{StreamManager, SSE_IDLE_TIMEOUT_SECS, SSE_MAX_DURATION_SECS};
use crate::task::{TaskExecutor, TaskRequest, TaskResult};
use crate::terminal::PtyManager;
use crate::updater::AutoUpdater;
use crate::watcher::FileWatcher;
use crate::bridge::interceptor::{MetricsInterceptor, RequestInterceptor};
use crate::analytics::{AnalyticsStore, DashboardStats, TrackEventRequest};
use crate::memory::CavemanRTKStats;
use anyhow::Result;
use axum::{
    extract::{Path, Query, State, Multipart},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use tokio::time::Duration;
use tower_http::cors::{CorsLayer, AllowOrigin};
use axum::extract::Request;
use axum::http::header::{HeaderName, ORIGIN, CONTENT_TYPE, AUTHORIZATION, ACCEPT};
use axum::http::Method;
use axum::middleware::{self, Next};

use crate::tools::{execute_tool, get_tool_definitions, ToolDefinition};
use crate::cache::FileCache;

const SSE_CONTENT_TYPE: &str = "text/event-stream; charset=utf-8";

pub fn set_sse_content_type(response: &mut axum::response::Response) {
    if let Ok(header_value) = SSE_CONTENT_TYPE.parse::<axum::http::HeaderValue>() {
        response.headers_mut().insert(CONTENT_TYPE, header_value);
    }
}

pub struct CsrfTokenStore {
    tokens: Arc<Mutex<HashMap<String, String>>>,
}

impl CsrfTokenStore {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn generate(&self, session_id: &str) -> String {
        let token = uuid::Uuid::new_v4().to_string();
        let mut tokens = self.tokens.lock().await;
        tokens.insert(session_id.to_string(), token.clone());
        token
    }

    pub async fn validate(&self, session_id: &str, token: &str) -> bool {
        let tokens = self.tokens.lock().await;
        match tokens.get(session_id) {
            Some(expected) => expected == token,
            None => false,
        }
    }

    pub async fn remove(&self, session_id: &str) {
        let mut tokens = self.tokens.lock().await;
        tokens.remove(session_id);
    }
}

pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
    max_tokens: u64,
    refill_rate: u64,
}

struct TokenBucket {
    tokens: u64,
    last_refill: std::time::Instant,
}

impl RateLimiter {
    pub fn new(max_tokens: u64, refill_rate: u64) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            max_tokens,
            refill_rate,
        }
    }

    pub async fn check(&self, key: &str) -> bool {
        let mut buckets = self.buckets.lock().await;
        let now = std::time::Instant::now();

        let bucket = buckets.entry(key.to_string()).or_insert_with(|| TokenBucket {
            tokens: self.max_tokens,
            last_refill: now,
        });

        let elapsed = now.saturating_duration_since(bucket.last_refill);
        let tokens_to_add = (elapsed.as_secs() * self.refill_rate).min(self.max_tokens);
        bucket.tokens = (bucket.tokens + tokens_to_add).min(self.max_tokens);
        bucket.last_refill = now;

        if bucket.tokens > 0 {
            bucket.tokens -= 1;
            true
        } else {
            false
        }
    }
}

#[derive(Serialize)]
pub struct GitBashStatus {
    pub required: bool,
    pub found: bool,
    pub path: Option<String>,
}

#[derive(Deserialize)]
pub struct StreamQuery {
    pub conversation_id: String,
    pub model: String,
    pub user_mode: Option<String>,
    pub env_token: Option<String>,
    pub env_base_url: Option<String>,
    pub research_mode: Option<bool>,
    pub messages: Option<String>,
}

pub struct ResearchTask {
    pub handle: tokio::task::JoinHandle<()>,
    pub event_tx: broadcast::Sender<ResearchEvent>,
}

#[derive(Deserialize)]
pub struct ToolRequest {
    pub name: String,
    pub input: serde_json::Value,
    pub cwd: Option<String>,
}

#[derive(Deserialize)]
pub struct GitRequest {
    pub cwd: Option<String>,
    pub file: Option<String>,
    pub message: Option<String>,
    pub remote: Option<String>,
    pub branch: Option<String>,
    pub force: Option<bool>,
}

/// Bridge AppState — typed as the named struct from `api::state`.
pub type AppState = crate::api::state::AppState;

/// Alias for the tuple-based state, used by `crate::api::state::AppState::from_tuple`.
pub type AppStateTuple = (
    Arc<Mutex<EnginePool>>,
    Arc<McpServerManager>,
    Arc<Mutex<StreamManager>>,
    Arc<Mutex<HashMap<String, bool>>>,
    Arc<Mutex<Option<ConfigManager>>>,
    Arc<Mutex<SkillsManager>>,
    Arc<DbManager>,
    Arc<Mutex<Option<TaskExecutor>>>,
    Arc<Mutex<ProcessManager>>,
    Arc<Mutex<PtyManager>>,
    Arc<Mutex<FileWatcher>>,
    Arc<Mutex<ClipboardManager>>,
    Arc<Mutex<NotificationManager>>,
    Arc<Mutex<Logger>>,
    Arc<Mutex<Option<NativeEngine>>>,
    Arc<Mutex<HashMap<String, ResearchTask>>>,
    Arc<Mutex<Option<crate::orchestration::MultiAgentOrchestrator>>>,
    Arc<MemExClient>,
    String,
    Arc<RateLimiter>,
    Arc<CostTracker>,
    Arc<PreviewEngine>,
    Arc<AnalyticsStore>,
    Arc<Mutex<ContextManager>>,
);

pub struct BridgeServer {
    engine_pool: Arc<Mutex<EnginePool>>,
    native_engine: Arc<Mutex<Option<NativeEngine>>>,
    mcp_server_manager: Arc<McpServerManager>,
    stream_manager: Arc<Mutex<StreamManager>>,
    research_mode: Arc<Mutex<HashMap<String, bool>>>,
    config_manager: Arc<Mutex<Option<ConfigManager>>>,
    skill_manager: Arc<Mutex<SkillsManager>>,
    db_manager: Arc<DbManager>,
    task_executor: Arc<Mutex<Option<TaskExecutor>>>,
    process_manager: Arc<Mutex<ProcessManager>>,
    terminal_manager: Arc<Mutex<PtyManager>>,
    file_watcher: Arc<Mutex<FileWatcher>>,
    clipboard_manager: Arc<Mutex<ClipboardManager>>,
    notification_manager: Arc<Mutex<NotificationManager>>,
    logger: Arc<Mutex<Logger>>,
    active_research: Arc<Mutex<HashMap<String, ResearchTask>>>,
    orchestrator: Arc<Mutex<Option<MultiAgentOrchestrator>>>,
    memex_client: Arc<MemExClient>,
    api_key: String,
    rate_limiter: Arc<RateLimiter>,
    cost_tracker: Arc<CostTracker>,
    preview_engine: Arc<PreviewEngine>,
    analytics_store: Arc<AnalyticsStore>,
    context_manager: Arc<Mutex<ContextManager>>,
}

impl BridgeServer {
    pub fn new(data_dir: PathBuf, db_manager: Arc<DbManager>) -> Self {
        let _skills_dir = data_dir.join("skills");
        let log_dir = data_dir.join("logs");
        let analytics_dir = data_dir.join("analytics");

        let skill_manager = SkillsManager::new();
        if let Err(e) = skill_manager.install_bundled_skills() {
            tracing::error!(module = "Bridge", error = %e);
        }

        tracing::info!(module = "Bridge", "Database initialized at {:?}", data_dir.join("claude_desktop.db"));
        tracing::info!(module = "Bridge", "Running migration check...");
        {
            let data_dir_ref = &data_dir;
            db_manager.with_conn(|conn| {
                if let Err(e) = crate::db::migration::check_and_migrate(data_dir_ref, conn) {
                    tracing::warn!(module = "Bridge", error = %e);
                }
            }).ok();
        }
        tracing::info!(module = "Bridge", "Migration check completed");
        let logger = Logger::new(log_dir);
        let file_watcher = FileWatcher::new();

        let config_dir = data_dir.clone();
        let config_manager = ConfigManager::new(config_dir.clone());
        tracing::info!(module = "Bridge", "ConfigManager initialized at {:?}", data_dir.display());
        let config_manager = Arc::new(Mutex::new(Some(config_manager)));

        let provider_manager = Arc::new(Mutex::new(ProviderManager::new(
            data_dir.join("providers.json")
        )));
        let task_executor = TaskExecutor::new_with_provider_manager(
            provider_manager.clone(),
            db_manager.clone(),
        );

        let audit_logger = Arc::new(AuditLogger::new(1000));
        let permission_manager = Arc::new(PermissionManager::new(audit_logger));

        // 注册 always_allow 检查器，使 PermissionManager 能够通过数据库检查始终允许规则
        let db_for_checker = db_manager.clone();
        permission_manager.set_always_allow_checker(Box::new(move |tool_name: &str, action: &str| -> bool {
            let tool = tool_name.to_string();
            let act = action.to_string();
            db_for_checker.with_conn(|conn| {
                crate::db::permission_repo::check_always_allowed(conn, &tool, &act)
            }).unwrap_or(false)
        }));

        let file_cache = Arc::new(FileCache::new());
        tracing::info!(module = "Bridge", "FileCache initialized");

        let native_engine = Arc::new(Mutex::new(Some(NativeEngine::new(
            provider_manager,
            db_manager.clone(),
            data_dir.join("workspaces"),
            permission_manager,
            file_cache.clone(),
        ))));
        tracing::info!(module = "Bridge", "NativeEngine initialized");

        let config_path = std::path::Path::new("config/orchestration.toml");
        let orchestrator_config = if config_path.exists() {
            OrchestratorConfigFile::load_or_default(config_path)
        } else {
            OrchestratorConfigFile::default()
        };
        let orchestrator = MultiAgentOrchestrator::new(
            (&orchestrator_config).into(),
            &data_dir,
        );
        let orchestrator = Arc::new(Mutex::new(Some(orchestrator)));
        tracing::info!(module = "Bridge", "MultiAgentOrchestrator initialized");

        let memex_client = Arc::new(MemExClient::new(None));
        tracing::info!(module = "Bridge", "MemExClient initialized (backend: {})", memex_client.base_url);

        let api_key = uuid::Uuid::new_v4().to_string();
        tracing::info!(module = "Bridge", "API Key generated for bridge authentication");

        let cost_tracker = Arc::new(CostTracker::new(data_dir.join("costs")));
        tracing::info!(module = "Bridge", "CostTracker initialized");

        let preview_engine = Arc::new(PreviewEngine::new(None));
        tracing::info!(module = "Bridge", "PreviewEngine initialized");

        let analytics_store = Arc::new(AnalyticsStore::new(analytics_dir));
        tracing::info!(module = "Bridge", "AnalyticsStore initialized");

        let context_manager = Arc::new(Mutex::new(ContextManager::new(memex_client.clone())));
        tracing::info!(module = "Bridge", "ContextManager with CavemanRTK initialized");

        Self {
            engine_pool: Arc::new(Mutex::new(EnginePool::new())),
            native_engine,
            mcp_server_manager: Arc::new(McpServerManager::new(config_dir.join("mcp-servers.json"))),
            stream_manager: Arc::new(Mutex::new(StreamManager::new())),
            research_mode: Arc::new(Mutex::new(HashMap::new())),
            config_manager,
            skill_manager: Arc::new(Mutex::new(skill_manager)),
            db_manager,
            task_executor: Arc::new(Mutex::new(Some(task_executor))),
            process_manager: Arc::new(Mutex::new(ProcessManager::new())),
            terminal_manager: Arc::new(Mutex::new(PtyManager::new())),
            file_watcher: Arc::new(Mutex::new(file_watcher)),
            clipboard_manager: Arc::new(Mutex::new(ClipboardManager::new())),
            notification_manager: Arc::new(Mutex::new(NotificationManager::new())),
            logger: Arc::new(Mutex::new(logger)),
            active_research: Arc::new(Mutex::new(HashMap::new())),
            orchestrator,
            memex_client,
            api_key,
            rate_limiter: Arc::new(RateLimiter::new(60, 60)),
            cost_tracker,
            preview_engine,
            analytics_store,
            context_manager,
        }
    }

    pub fn get_api_key(&self) -> &str {
        &self.api_key
    }

    pub async fn start(&self, port: u16) -> Result<()> {
        if let Err(e) = self.mcp_server_manager.initialize().await {
            tracing::error!(module = "Bridge", "Failed to initialize MCP server manager: {}", e);
        }

        let csrf_token_store = Arc::new(CsrfTokenStore::new());
        let request_interceptor = Arc::new(RequestInterceptor::new());
        let metrics_interceptor = Arc::new(MetricsInterceptor::new());

        let state: AppState = AppState {
            engine_pool: self.engine_pool.clone(),
            mcp_manager: self.mcp_server_manager.clone(),
            stream_manager: self.stream_manager.clone(),
            research_mode: self.research_mode.clone(),
            config_manager: self.config_manager.clone(),
            skills_manager: self.skill_manager.clone(),
            db_manager: self.db_manager.clone(),
            task_executor: self.task_executor.clone(),
            process_manager: self.process_manager.clone(),
            terminal_manager: self.terminal_manager.clone(),
            file_watcher: self.file_watcher.clone(),
            clipboard_manager: self.clipboard_manager.clone(),
            notification_manager: self.notification_manager.clone(),
            logger: self.logger.clone(),
            native_engine: self.native_engine.clone(),
            active_research: self.active_research.clone(),
            orchestrator: self.orchestrator.clone(),
            memex_client: self.memex_client.clone(),
            api_key: self.api_key.clone(),
            rate_limiter: self.rate_limiter.clone(),
            cost_tracker: self.cost_tracker.clone(),
            preview_engine: self.preview_engine.clone(),
            analytics_store: self.analytics_store.clone(),
            context_manager: self.context_manager.clone(),
            git_integration: Arc::new(GitIntegration::new(
                std::env::current_dir().unwrap_or_default(),
            )),
            permission_manager: Arc::new(PermissionManager::new(
                Arc::new(crate::permissions::AuditLogger::new(1000)),
            )),
            csrf_token_store: csrf_token_store.clone(),
            request_interceptor: request_interceptor.clone(),
            metrics_interceptor: metrics_interceptor.clone(),
        };
        let allowed_origins: Vec<axum::http::HeaderValue> = [
            "tauri://localhost",
            "https://tauri.localhost",
            "http://tauri.localhost",
            "http://localhost:1420",
            "http://localhost:3456",
            "http://localhost:5173",
            "http://localhost:5180",
            "http://127.0.0.1:1420",
            "http://127.0.0.1:3456",
            "http://127.0.0.1:5173",
            "http://127.0.0.1:5180",
            "http://127.0.0.1:5190",
            "http://localhost:5190",
            "http://127.0.0.1:5200",
            "http://localhost:5200",
            "null",
        ]
        .iter()
        .filter_map(|s| s.parse::<axum::http::HeaderValue>().ok())
        .collect();

        let cors = CorsLayer::new()
            .allow_origin(AllowOrigin::list(allowed_origins))
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE, Method::OPTIONS])
            .allow_headers([
                CONTENT_TYPE,
                AUTHORIZATION,
                ACCEPT,
                ORIGIN,
                HeaderName::from_static("x-conversation-id"),
                HeaderName::from_static("x-api-key"),
                HeaderName::from_static("x-csrf-token"),
                HeaderName::from_static("x-session-id"),
            ]);

        let api_key_for_middleware = self.api_key.clone();
        let csrf_store_for_middleware = csrf_token_store.clone();
        let auth_middleware = middleware::from_fn(move |req: Request<axum::body::Body>, next: Next| {
            let key = api_key_for_middleware.clone();
            let csrf = csrf_store_for_middleware.clone();
            async move {
                let path = req.uri().path();
                let method = req.method().as_str();

                // CORS preflight — always pass
                if method == "OPTIONS" {
                    return next.run(req).await;
                }

                // Public endpoints (no auth required)
                if path == "/health" || path == "/metrics" || path == "/api/csrf-token" || path.starts_with("/api/im/webhook/") {
                    return next.run(req).await;
                }

                // Workspace config and system status — public read
                if (path == "/api/system-status" || path == "/api/workspace-config") && method == "GET" {
                    return next.run(req).await;
                }

                // Validate x-api-key header
                if let Some(val) = req.headers().get(HeaderName::from_static("x-api-key")) {
                    if let Ok(val) = val.to_str() {
                        if val == key {
                            // CSRF check for state-changing requests
                            if method == "POST" || method == "PUT" || method == "PATCH" || method == "DELETE" {
                                if let Some(csrf_token) = req.headers().get(HeaderName::from_static("x-csrf-token")) {
                                    if let Ok(token_val) = csrf_token.to_str() {
                                        if let Some(session_id) = req.headers().get(HeaderName::from_static("x-session-id")) {
                                            if let Ok(sid) = session_id.to_str() {
                                                if csrf.validate(sid, token_val).await {
                                                    return next.run(req).await;
                                                }
                                            }
                                        }
                                    }
                                }
                                let body = axum::Json(json!({"error": "Invalid or missing CSRF token"}));
                                return (StatusCode::FORBIDDEN, body).into_response();
                            }
                            return next.run(req).await;
                        }
                    }
                }

                // Validate Authorization: Bearer <token>
                if let Some(val) = req.headers().get(AUTHORIZATION) {
                    if let Ok(val) = val.to_str() {
                        if let Some(token) = val.strip_prefix("Bearer ") {
                            if token == key {
                                return next.run(req).await;
                            }
                        }
                    }
                }

                let body = axum::Json(json!({"error": "Invalid or missing API key"}));
                (StatusCode::UNAUTHORIZED, body).into_response()
            }
        });

        let rate_limiter_for_middleware = self.rate_limiter.clone();
        let rate_limit_middleware = middleware::from_fn(move |req: Request<axum::body::Body>, next: Next| {
            let limiter = rate_limiter_for_middleware.clone();
            async move {
                let path = req.uri().path();
                if path == "/health" || path == "/metrics" {
                    return next.run(req).await;
                }
                let client_key = req.headers()
                    .get(HeaderName::from_static("x-api-key"))
                    .and_then(|v| v.to_str().ok())
                    .or_else(|| req.headers().get(AUTHORIZATION)
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.strip_prefix("Bearer ")))
                    .unwrap_or("anonymous");
                if !limiter.check(client_key).await {
                    return (StatusCode::TOO_MANY_REQUESTS,
                        axum::Json(json!({"error": "Rate limit exceeded. Please retry later."})))
                        .into_response();
                }
                next.run(req).await
            }
        });

        let interceptor_middleware = middleware::from_fn(interceptor::interceptor_middleware);

        let app = routes::build_all_routes()
            .layer(cors)
            .layer(rate_limit_middleware)
            .layer(auth_middleware)
            .layer(interceptor_middleware)
            .with_state(state);

        let mut bound_port = None;
        for attempt_port in [port, port + 1, port + 2, port + 3, port + 4] {
            match tokio::net::TcpListener::bind(format!("127.0.0.1:{}", attempt_port)).await {
                Ok(l) => {
                    bound_port = Some((l, attempt_port));
                    break;
                }
                Err(e) => {
                    tracing::warn!(module = "Bridge", "Port {} bind failed: {}, trying next...", attempt_port, e);
                }
            }
        }
        let (listener, actual_port) = bound_port.ok_or_else(|| anyhow::anyhow!("Failed to bind any port in range {}-{}", port, port + 4))?;
        if actual_port != port {
            tracing::warn!(module = "Bridge", "Original port {} unavailable, using port {} instead", port, actual_port);
        }
        tracing::info!(module = "Bridge", "Server running on http://127.0.0.1:{}", actual_port);
        axum::serve(listener, app).await?;
        Ok(())
    }
}

async fn system_status() -> Json<SystemStatus> {
    let platform = std::env::consts::OS.to_string();

    Json(SystemStatus {
        platform,
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let db_healthy = {
        let db = &state.db_manager;
        db.with_conn(|conn| {
            conn.execute_batch("SELECT 1")
        }).is_ok()
    };

    let active_sse = crate::metrics::ACTIVE_SSE_CONNECTIONS.get();

    let status = if db_healthy { "healthy" } else { "unhealthy" };
    let http_status = if db_healthy { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };

    let body = serde_json::json!({
        "status": status,
        "database": db_healthy,
        "active_sse_connections": active_sse,
    });

    (http_status, Json(body))
}

async fn csrf_token_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let session_id = headers
        .get(HeaderName::from_static("x-session-id"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("default")
        .to_string();
    let token = state.csrf_token_store.generate(&session_id).await;
    Json(json!({ "csrf_token": token, "session_id": session_id }))
}

async fn interceptor_stats_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let snapshot = state.metrics_interceptor.snapshot().await;
    Json(snapshot)
}

async fn interceptor_blocked_paths_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let blocked = state.request_interceptor.blocked_paths.read().await;
    Json(json!({ "blocked_paths": blocked.clone() }))
}

async fn interceptor_block_path_handler(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let path = body.get("path").and_then(|v| v.as_str()).unwrap_or("");
    if !path.is_empty() {
        state.request_interceptor.add_blocked_path(path).await;
        Json(json!({ "ok": true, "path": path }))
    } else {
        Json(json!({ "error": "path is required" }))
    }
}

async fn interceptor_unblock_path_handler(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let path = body.get("path").and_then(|v| v.as_str()).unwrap_or("");
    if !path.is_empty() {
        state.request_interceptor.remove_blocked_path(path).await;
        Json(json!({ "ok": true, "path": path }))
    } else {
        Json(json!({ "error": "path is required" }))
    }
}

async fn metrics_handler() -> impl IntoResponse {
    let metrics = crate::metrics::gather_metrics();
    (
        StatusCode::OK,
        [("Content-Type", "text/plain; version=0.0.4; charset=utf-8")],
        metrics,
    )
}

#[derive(Serialize)]
struct WorkspaceConfig {
    default_dir: String,
}

async fn workspace_config_get() -> Json<WorkspaceConfig> {
    let default_dir = dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());
    Json(WorkspaceConfig { default_dir })
}

#[derive(Deserialize)]
struct WorkspaceConfigUpdate {
    dir: String,
}

async fn workspace_config_set(
    Json(body): Json<WorkspaceConfigUpdate>,
) -> Json<serde_json::Value> {
    let dir = body.dir.clone();
    let valid = std::path::Path::new(&dir).is_dir();
    tracing::info!(module = "WorkspaceConfig", "Set workspace dir to: {}", dir);
    Json(serde_json::json!({ "ok": valid, "dir": dir }))
}

async fn performance_overview_handler(_state: State<AppState>) -> Json<serde_json::Value> {
    use crate::metrics::{
        ACTIVE_SSE_CONNECTIONS, TOOL_CALLS_TOTAL, TOKENS_CONSUMED,
        TOKENS_INPUT, TOKENS_OUTPUT, TOKENS_SAVED_COMPRESSION,
        TOKENS_PROCESSED, MEMORY_SEGMENTS,
    };

    let active_models = {
        crate::metrics::ACTIVE_SSE_CONNECTIONS.get() as usize
    };

    let rate_info = serde_json::json!({
        "enabled": true,
    });

    Json(serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "status": "operational",
        "connections": {
            "active_sse": ACTIVE_SSE_CONNECTIONS.get(),
            "active_models": active_models,
        },
        "tokens": {
            "total_consumed": TOKENS_CONSUMED.get(),
            "input_total": TOKENS_INPUT.get(),
            "output_total": TOKENS_OUTPUT.get(),
            "saved_by_compression": TOKENS_SAVED_COMPRESSION.get(),
            "processed_through_compressor": TOKENS_PROCESSED.get(),
        },
        "tools": {
            "total_calls": TOOL_CALLS_TOTAL.get(),
            "memory_segments": MEMORY_SEGMENTS.get(),
        },
        "rate_limiter": rate_info,
        "system": {
            "pid": std::process::id(),
        },
    }))
}



async fn preview_list_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let preview_engine = state.preview_engine.clone();
    let contents = preview_engine.list_contents().await;

    let result: Vec<serde_json::Value> = contents.iter().map(|c| {
        serde_json::json!({
            "id": c.id,
            "content_type": c.content_type,
            "last_updated": c.last_updated,
        })
    }).collect();

    Json(serde_json::json!({ "success": true, "items": result }))
}

async fn preview_get_handler(Path(id): Path<String>, State(state): State<AppState>) -> Json<serde_json::Value> {
    let preview_engine = state.preview_engine.clone();

    match preview_engine.get_content(&id).await {
        Ok(Some(content)) => Json(serde_json::json!({
            "success": true,
            "id": content.id,
            "content": content.content,
            "content_type": content.content_type,
            "last_updated": content.last_updated,
        })),
        Ok(None) => Json(serde_json::json!({ "success": false, "error": "Preview content not found" })),
        Err(e) => Json(serde_json::json!({ "success": false, "error": format!("{}", e) })),
    }
}

#[derive(Deserialize)]
struct PreviewSetRequest {
    content: String,
    content_type: String,
}

async fn preview_set_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(req): Json<PreviewSetRequest>,
) -> Json<serde_json::Value> {
    let preview_engine = state.preview_engine.clone();

    match preview_engine.set_content(&id, &req.content, &req.content_type).await {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "success": false, "error": format!("{}", e) })),
    }
}

async fn preview_delete_handler(Path(id): Path<String>, State(state): State<AppState>) -> Json<serde_json::Value> {
    let preview_engine = state.preview_engine.clone();

    match preview_engine.remove_content(&id).await {
        Ok(true) => Json(serde_json::json!({ "success": true })),
        Ok(false) => Json(serde_json::json!({ "success": false, "error": "Preview content not found" })),
        Err(e) => Json(serde_json::json!({ "success": false, "error": format!("{}", e) })),
    }
}

async fn preview_events_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let preview_engine = state.preview_engine.clone();

    let stream = async_stream::stream! {
        loop {
            preview_engine.wait_for_update().await;

            match preview_engine.get_content(&id).await {
                Ok(Some(content)) => {
                    let data = serde_json::json!({
                        "type": "update",
                        "id": content.id,
                        "content": content.content,
                        "content_type": content.content_type,
                        "last_updated": content.last_updated,
                    });
                    yield Ok::<Event, Infallible>(Event::default().data(data.to_string()));
                }
                Ok(None) => {
                    let data = serde_json::json!({
                        "type": "deleted",
                        "id": id.clone(),
                    });
                    yield Ok::<Event, Infallible>(Event::default().data(data.to_string()));
                    break;
                }
                Err(e) => {
                    let data = serde_json::json!({
                        "type": "error",
                        "message": e.to_string(),
                    });
                    yield Ok::<Event, Infallible>(Event::default().data(data.to_string()));
                }
            }
        }
    };

    let mut response = Sse::new(stream)
        .keep_alive(KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"))
        .into_response();
    set_sse_content_type(&mut response);
    Ok(response)
}

async fn costs_dashboard_handler(State(state): State<AppState>) -> impl IntoResponse {
    let tracker = &state.cost_tracker;
    let sessions: Vec<crate::cost_tracker::SessionCost> = tracker.get_all_sessions().await;
    let daily_records: Vec<crate::cost_tracker::DailyUsageRecord> = tracker.get_daily_records(30).await;
    let usage_stats = tracker.get_usage_stats().await;

    let mut model_breakdown: HashMap<String, serde_json::Value> = HashMap::new();
    for session in &sessions {
        let entry = model_breakdown.entry(session.model.clone())
            .or_insert_with(|| serde_json::json!({
                "total_tokens": 0u64,
                "total_cost": 0.0f64,
                "session_count": 0u32,
            }));
        let tokens = entry.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0) + session.total_input_tokens + session.total_output_tokens;
        let cost = entry.get("total_cost").and_then(|v| v.as_f64()).unwrap_or(0.0) + session.estimated_cost;
        let count = entry.get("session_count").and_then(|v| v.as_u64()).unwrap_or(0) + 1;
        *entry = serde_json::json!({
            "total_tokens": tokens,
            "total_cost": cost,
            "session_count": count,
        });
    }

    let daily_trend: Vec<serde_json::Value> = daily_records.iter().map(|r| {
        serde_json::json!({
            "date": r.date,
            "total_tokens": r.total_tokens,
            "total_cost": r.total_cost,
        })
    }).collect();

    let total_tokens: u64 = sessions.iter().map(|s| s.total_input_tokens + s.total_output_tokens).sum();
    let total_cost: f64 = sessions.iter().map(|s| s.estimated_cost).sum();

    let body = serde_json::json!({
        "total_tokens": total_tokens,
        "total_cost": total_cost,
        "session_count": sessions.len(),
        "model_breakdown": model_breakdown,
        "daily_trend": daily_trend,
        "usage_stats": usage_stats,
    });

    (StatusCode::OK, Json(body))
}

async fn costs_budget_get_handler(State(state): State<AppState>) -> impl IntoResponse {
    let tracker = &state.cost_tracker;
    let stats = tracker.get_usage_stats().await;
    (StatusCode::OK, Json(stats))
}

#[derive(Deserialize)]
struct BudgetRequest {
    daily_budget: Option<u64>,
    monthly_budget: Option<u64>,
}

async fn costs_budget_set_handler(
    State(state): State<AppState>,
    Json(req): Json<BudgetRequest>,
) -> impl IntoResponse {
    let tracker = &state.cost_tracker;
    if let Some(daily) = req.daily_budget {
        tracker.set_daily_budget(if daily == 0 { None } else { Some(daily) }).await;
    }
    if let Some(monthly) = req.monthly_budget {
        tracker.set_monthly_budget(if monthly == 0 { None } else { Some(monthly) }).await;
    }
    let stats = tracker.get_usage_stats().await;
    (StatusCode::OK, Json(stats))
}

async fn costs_usage_handler(State(state): State<AppState>) -> impl IntoResponse {
    let tracker = &state.cost_tracker;
    let stats = tracker.get_usage_stats().await;
    (StatusCode::OK, Json(stats))
}

async fn tools_handler(
    Json(req): Json<ToolRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let cwd = req.cwd.clone().unwrap_or_else(|| ".".to_string());
    let name = req.name.clone();
    let input = req.input.clone();

    let result = tokio::task::spawn_blocking(move || {
        execute_tool(&name, input, &cwd)
    }).await;

    match result {
        Ok(Ok(result)) => Ok(Json(result)),
        _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn tool_execute_handler(
    _state: State<AppState>,
    Json(req): Json<ToolRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    crate::metrics::TOOL_CALLS_TOTAL.inc();
    let _tool_timer = crate::metrics::TOOL_CALL_DURATION.start_timer();

    let cwd = req.cwd.clone().unwrap_or_else(|| ".".to_string());
    let name = req.name.clone();
    let name_log = name.clone();
    let input_log = serde_json::to_string(&req.input).unwrap_or_default();

    let start_time = std::time::Instant::now();
    let result = tokio::task::spawn_blocking(move || {
        execute_tool(&name, req.input, &cwd)
    }).await;

    let duration_ms = start_time.elapsed().as_millis() as u64;

    match result {
        Ok(Ok(ref result_value)) => {
            tracing::info!(
                module = "Audit",
                "Tool: {} | Success: true | Duration: {}ms | Input: {}",
                name_log, duration_ms, input_log
            );
            Ok(Json(result_value.clone()))
        }
        _ => {
            tracing::error!(
                module = "Audit",
                "Tool: {} | Success: false | Duration: {}ms | Input: {}",
                name_log, duration_ms, input_log
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn tools_list_handler() -> Json<Vec<ToolDefinition>> {
    Json(get_tool_definitions())
}

async fn projects_list(State(state): State<AppState>) -> Json<serde_json::Value> {
    let db = state.db_manager.clone();
    let result = tokio::task::spawn_blocking(move || {
        db.with_conn(|conn| crate::db::project_repo::list_projects(conn))
    }).await;
    match result {
        Ok(Ok(Ok(projects))) => Json(serde_json::json!({ "projects": projects })),
        _ => Json(serde_json::json!({ "projects": [] })),
    }
}

#[derive(Deserialize)]
struct ProjectCreateRequest {
    name: String,
    description: Option<String>,
    instructions: Option<String>,
    workspace_path: Option<String>,
}

async fn projects_create(State(state): State<AppState>, Json(req): Json<ProjectCreateRequest>) -> Json<serde_json::Value> {
    let id = uuid::Uuid::new_v4().to_string();
    let req_name = req.name.clone();
    let db = state.db_manager.clone();
    let id_for_db = id.clone();
    let name_for_db = req_name.clone();
    let desc_clone = req.description.clone();
    let instructions_clone = req.instructions.clone();
    let workspace_clone = req.workspace_path.clone();
    let result = tokio::task::spawn_blocking(move || {
        db.with_conn(|conn| {
            let now = chrono::Utc::now().to_rfc3339();
            crate::db::project_repo::insert_project(
                conn,
                &id_for_db,
                &name_for_db,
                desc_clone.as_deref(),
                instructions_clone.as_deref(),
                workspace_clone.as_deref(),
                false,
                &now,
                &now,
            )
        })
    }).await;
    match result {
        Ok(Ok(Ok(()))) => Json(serde_json::json!({ "id": id.clone(), "name": req_name })),
        _ => {
            tracing::error!(module = "Projects", "Failed to create project {}", id);
            Json(serde_json::json!({ "error": "Failed to create project" }))
        }
    }
}

static UPLOAD_DIR: once_cell::sync::Lazy<std::sync::Mutex<Option<PathBuf>>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(None));

fn get_upload_dir() -> PathBuf {
    let guard = UPLOAD_DIR.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(dir) = guard.as_ref() {
        return dir.clone();
    }
    drop(guard);
    let default_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
        .join("claude-desktop")
        .join("uploads");
    let mut guard = UPLOAD_DIR.lock().unwrap_or_else(|e| e.into_inner());
    *guard = Some(default_dir.clone());
    default_dir
}

async fn upload_handler(mut multipart: Multipart) -> Result<Json<serde_json::Value>, StatusCode> {
    let upload_dir = get_upload_dir();
    std::fs::create_dir_all(&upload_dir).map_err(|e| {
        tracing::error!(module = "Upload", "Failed to create upload dir: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!(module = "Upload", "Multipart error: {}", e);
        StatusCode::BAD_REQUEST
    })? {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let file_name = field.file_name()
                .unwrap_or("unnamed")
                .to_string();
            let content_type = field.content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;

            let file_size = data.len();
            let file_id = uuid::Uuid::new_v4().to_string();
            let ext = std::path::Path::new(&file_name)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            let file_type = if content_type.starts_with("image/") {
                "image"
            } else if content_type.starts_with("video/") {
                "video"
            } else if content_type.starts_with("audio/") {
                "audio"
            } else if content_type == "application/pdf" || ext == "pdf" {
                "document"
            } else if content_type.starts_with("text/") || matches!(ext, "txt" | "md" | "csv" | "json" | "xml" | "yaml" | "yml") {
                "text"
            } else {
                "document"
            };

            let dest_path = upload_dir.join(&file_id);
            tokio::fs::write(&dest_path, &data).await.map_err(|e| {
                tracing::error!(module = "Upload", "Failed to save file: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            tracing::info!(module = "Upload", "File saved: {} ({} bytes, type: {})", file_name, file_size, file_type);

            return Ok(Json(serde_json::json!({
                "fileId": file_id,
                "fileName": file_name,
                "fileType": file_type,
                "mimeType": content_type,
                "size": file_size,
            })));
        }
    }

    Err(StatusCode::BAD_REQUEST)
}

use axum::body::Body;
use axum::response::Response;
use axum::http::header;

async fn upload_get_handler(Path(id): Path<String>) -> Result<Response<Body>, StatusCode> {
    // Bug #1 fix: Prevent path traversal - reject ids containing path separators or parent refs
    if id.contains("..") || id.contains('/') || id.contains('\\') {
        tracing::warn!(module = "Upload", "Rejected path traversal attempt: {:?}", id);
        return Err(StatusCode::BAD_REQUEST);
    }

    let upload_dir = get_upload_dir();
    let file_path = upload_dir.join(&id);

    // Ensure the resolved path is still under the upload directory
    match file_path.canonicalize() {
        Ok(canonical) => {
            if let Ok(upload_canonical) = upload_dir.canonicalize() {
                if !canonical.starts_with(&upload_canonical) {
                    tracing::warn!(module = "Upload", "Path escapes upload directory: {:?}", canonical);
                    return Err(StatusCode::FORBIDDEN);
                }
            }
        }
        Err(_) => return Err(StatusCode::NOT_FOUND),
    }

    if !file_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    let data = tokio::fs::read(&file_path).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let mime_type = match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "md" => "text/markdown",
        "json" => "application/json",
        "xml" => "application/xml",
        "html" => "text/html",
        "css" => "text/css",
        "js" => "text/javascript",
        "csv" => "text/csv",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        "avi" => "video/x-msvideo",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "m4a" => "audio/mp4",
        _ => "application/octet-stream",
    };

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type)
        .header(header::CONTENT_LENGTH, data.len())
        .header(header::CACHE_CONTROL, "public, max-age=31536000")
        .body(Body::from(data))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(response)
}

async fn upload_delete_handler(Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    let upload_dir = get_upload_dir();
    let file_path = upload_dir.join(&id);

    if file_path.exists() {
        tokio::fs::remove_file(&file_path).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        tracing::info!(module = "Upload", "File deleted: {}", id);
    }

    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn config_get(State(state): State<AppState>) -> Json<serde_json::Value> {
    let config_manager = state.config_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, Option<ConfigManager>> = config_manager.lock().await;
    if let Some(m) = manager.as_ref() {
        return Json(serde_json::to_value(m.get_config()).unwrap_or_default());
    }
    Json(serde_json::json!({}))
}

async fn config_update(State(state): State<AppState>, Json(config): Json<AppConfig>) -> Json<serde_json::Value> {
    let config_manager = state.config_manager.clone();
    let mut manager: tokio::sync::MutexGuard<'_, Option<ConfigManager>> = config_manager.lock().await;
    if let Some(m) = manager.as_mut() {
        let _ = m.update_config(|c| *c = config);
    }
    Json(serde_json::json!({ "ok": true }))
}

#[derive(Deserialize)]
pub struct TaskExecuteRequest {
    pub task_id: String,
    pub prompt: String,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub context: Option<Vec<serde_json::Value>>,
}

async fn task_execute(
    State(state): State<AppState>,
    Json(req): Json<TaskExecuteRequest>,
) -> Result<Json<TaskResult>, StatusCode> {
    let task_executor = state.task_executor.clone();
    let executor: tokio::sync::MutexGuard<'_, Option<TaskExecutor>> = task_executor.lock().await;
    if let Some(e) = executor.as_ref() {
        let task_request = TaskRequest {
            task_id: req.task_id,
            prompt: req.prompt,
            model: req.model,
            max_tokens: req.max_tokens,
            context: req.context,
            tools: None,
        };

        match e.execute_task(task_request).await {
            Ok(result) => Ok(Json(result)),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

async fn task_status(Path(id): Path<String>, State(state): State<AppState>) -> Json<serde_json::Value> {
    let task_executor = state.task_executor.clone();
    let executor: tokio::sync::MutexGuard<'_, Option<TaskExecutor>> = task_executor.lock().await;
    if let Some(e) = executor.as_ref() {
        if let Some(status) = e.get_task_status(&id).await {
            return Json(serde_json::json!({ "status": format!("{:?}", status) }));
        }
    }
    Json(serde_json::json!({ "status": "not_found" }))
}

async fn task_cancel(Path(id): Path<String>, State(state): State<AppState>) -> Json<serde_json::Value> {
    let task_executor = state.task_executor.clone();
    let executor: tokio::sync::MutexGuard<'_, Option<TaskExecutor>> = task_executor.lock().await;
    if let Some(e) = executor.as_ref() {
        let cancelled = e.cancel_task(&id).await;
        return Json(serde_json::json!({ "cancelled": cancelled }));
    }
    Json(serde_json::json!({ "cancelled": false }))
}

async fn mcp_servers_list(State(state): State<AppState>) -> Json<serde_json::Value> {
    let mcp_server_manager = state.mcp_manager.clone();
    let servers: Vec<crate::mcp::McpServerStatus> = mcp_server_manager.list_servers().await;
    let servers_json: Vec<serde_json::Value> = servers
        .iter()
        .map(|s| serde_json::json!({
            "id": s.id,
            "name": s.name,
            "command": s.command,
            "args": s.args,
            "enabled": s.enabled,
            "running": s.running,
            "pid": s.pid,
            "tools_count": s.tools_count,
            "resources_count": s.resources_count,
            "error": s.error,
            "transport_type": s.transport_type
        }))
        .collect();

    Json(serde_json::json!({ "servers": servers_json }))
}

async fn mcp_servers_update(State(state): State<AppState>, Json(servers): Json<Vec<McpServerConfig>>) -> Json<serde_json::Value> {
    let mcp_server_manager = state.mcp_manager.clone();

    for server in servers {
        if let Err(e) = mcp_server_manager.add_server(server).await {
            tracing::error!(module = "Bridge", "Failed to add MCP server: {}", e);
        }
    }

    Json(serde_json::json!({ "ok": true }))
}

async fn mcp_tools_list(Path(name): Path<String>, State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let mcp_server_manager = state.mcp_manager.clone();
    let tools: Vec<crate::mcp::McpTool> = mcp_server_manager.get_all_tools().await;

    let tools_json: Vec<serde_json::Value> = tools
        .iter()
        .filter(|t| t.server_name == name)
        .map(|t| serde_json::json!({
            "name": t.name,
            "description": t.description,
            "input_schema": t.input_schema
        }))
        .collect();

    Ok(Json(serde_json::json!({ "tools": tools_json })))
}

async fn mcp_resources_list(Path(_name): Path<String>, State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let mcp_server_manager = state.mcp_manager.clone();
    let resources: Vec<crate::mcp::McpResource> = mcp_server_manager.get_all_resources().await;

    let resources_json: Vec<serde_json::Value> = resources
        .iter()
        .map(|r| serde_json::json!({
            "uri": r.uri,
            "name": r.name,
            "mime_type": r.mime_type
        }))
        .collect();

    Ok(Json(serde_json::json!({ "resources": resources_json })))
}

async fn mcp_resource_read(Path((name, uri)): Path<(String, String)>, State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let mcp_server_manager = state.mcp_manager.clone();

    match mcp_server_manager.read_resource(&name, &uri, None).await {
        Ok(content) => Ok(Json(serde_json::json!({
            "uri": content.uri,
            "content": content.content,
            "content_type": content.content_type,
            "metadata": content.metadata
        }))),
        Err(e) => {
            tracing::error!(module = "Bridge", "Failed to read resource: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn mcp_resource_monitor(Path((name, uri)): Path<(String, String)>, State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let mcp_server_manager = state.mcp_manager.clone();

    match mcp_server_manager.monitor_resource(&name, &uri, true).await {
        Ok(enabled) => Ok(Json(serde_json::json!({
            "uri": uri,
            "enabled": enabled
        }))),
        Err(e) => {
            tracing::error!(module = "Bridge", "Failed to monitor resource: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn mcp_connect_handler(Path(name): Path<String>, State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let mcp_server_manager = state.mcp_manager.clone();

    match mcp_server_manager.start_server(&name).await {
        Ok(_) => {
            if let Some(status) = mcp_server_manager.get_server(&name).await {
                Ok(Json(serde_json::json!({
                    "ok": true,
                    "name": status.name,
                    "status": if status.running { "running" } else { "ready" },
                    "tools_count": status.tools_count,
                    "resources_count": status.resources_count
                })))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        },
        Err(e) => {
            tracing::error!(module = "Bridge", "Failed to connect MCP server: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn mcp_disconnect_handler(Path(name): Path<String>, State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let mcp_server_manager = state.mcp_manager.clone();

    match mcp_server_manager.stop_server(&name).await {
        Ok(_) => Ok(Json(serde_json::json!({ "ok": true }))),
        Err(e) => {
            tracing::error!(module = "Bridge", "Failed to disconnect MCP server: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn engine_status_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    let pool = state.engine_pool.clone();
    let pool_guard: tokio::sync::MutexGuard<'_, EnginePool> = pool.lock().await;
    let engines: Vec<serde_json::Value> = pool_guard.list_engines()
        .iter()
        .map(|e| serde_json::json!({
            "conv_id": e.conv_id,
            "pid": e.pid,
            "model": e.model,
            "session_id": e.session_id,
            "state": format!("{:?}", e.state),
            "workspace": e.workspace.to_string_lossy()
        }))
        .collect();

    Json(serde_json::json!({
        "engines": engines,
        "workspace": pool_guard.get_workspace().to_string_lossy()
    }))
}

#[derive(Deserialize)]
pub struct SpawnRequest {
    pub conv_id: String,
    pub model: String,
    pub cwd: Option<String>,
}

async fn engine_spawn_handler(State(state): State<AppState>, Json(req): Json<SpawnRequest>) -> Result<Json<serde_json::Value>, StatusCode> {
    let pool = state.engine_pool.clone();
    let mut pool_guard: tokio::sync::MutexGuard<'_, EnginePool> = pool.lock().await;
    match pool_guard.spawn_engine(&req.conv_id, &req.model, req.cwd).await {
        Ok(handle) => Ok(Json(serde_json::json!({
            "ok": true,
            "conv_id": handle.conv_id,
            "session_id": handle.session_id,
            "pid": handle.pid
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn engine_kill_handler(Path(conv_id): Path<String>, State(state): State<AppState>) -> Json<serde_json::Value> {
    let pool = state.engine_pool.clone();
    let mut pool_guard: tokio::sync::MutexGuard<'_, EnginePool> = pool.lock().await;
    pool_guard.remove_engine(&conv_id).await;
    Json(serde_json::json!({ "ok": true }))
}

async fn stream_events_handler(Path(conv_id): Path<String>, State(state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    let stream_manager = state.stream_manager.clone();
    let mut manager: tokio::sync::MutexGuard<'_, StreamManager> = stream_manager.lock().await;

    let receiver = manager.add_listener(&conv_id)
        .ok_or_else(|| StatusCode::NOT_FOUND)?;

    let idle_timeout = Duration::from_secs(SSE_IDLE_TIMEOUT_SECS);
    let max_duration = Duration::from_secs(SSE_MAX_DURATION_SECS);
    let mut last_activity = std::time::Instant::now();

    crate::metrics::ACTIVE_SSE_CONNECTIONS.inc();
    let stream = async_stream::stream! {
        let mut rx = receiver;
        let max_sleep = tokio::time::sleep(max_duration);
        tokio::pin!(max_sleep);

        loop {
            let idle_sleep = tokio::time::sleep(idle_timeout);
            tokio::pin!(idle_sleep);

            tokio::select! {
                _ = &mut max_sleep => {
                    tracing::error!(module = "SSE_stream_events", "Max duration reached, closing stream");
                    break;
                }
                _ = &mut idle_sleep => {
                    if last_activity.elapsed() >= idle_timeout {
                        tracing::error!(module = "SSE_stream_events", "Idle timeout, closing stream");
                        break;
                    }
                }
                result = rx.recv() => {
                    match result {
                        Ok(event) => {
                            last_activity = std::time::Instant::now();
                            let event_name = event.event_type;
                            let data = serde_json::to_string(&event.data).unwrap_or_default();
                            yield Ok::<Event, Infallible>(Event::default()
                                .event(&event_name)
                                .data(data));
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            tracing::error!(module = "SSE_stream_events", "Receiver lagged, dropped {} events", n);
                            last_activity = std::time::Instant::now();
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
            }
        }

        tracing::info!(module = "SSE_stream_events", "Stream ended for conv_id={}", conv_id);
        crate::metrics::ACTIVE_SSE_CONNECTIONS.dec();
    };

    let mut response = Sse::new(stream).keep_alive(KeepAlive::default()).into_response();
    set_sse_content_type(&mut response);
    Ok(response)
}

async fn computer_use_screen_info() -> Json<serde_json::Value> {
    let manager = crate::computer_use::ComputerUseManager::new(crate::computer_use::ComputerUseConfig::default());
    let info = manager.get_screen_info();
    Json(serde_json::json!({
        "width": info.width,
        "height": info.height,
        "scaleFactor": info.scale_factor,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ComputerUseRequest {
    action_type: String,
    coordinate: Option<[i32; 2]>,
    button: Option<String>,
    key: Option<String>,
    text: Option<String>,
    scroll_y: Option<i32>,
    scroll_x: Option<i32>,
    duration_ms: Option<u64>,
}

async fn computer_use_execute(Json(req): Json<ComputerUseRequest>) -> Json<serde_json::Value> {
    // Bug #11 fix: Use a shared ComputerUseManager to preserve state (action_history, pressed_keys, etc.)
    // across requests instead of creating a new one each time.
    static SHARED_COMPUTER_USE: once_cell::sync::Lazy<tokio::sync::Mutex<crate::computer_use::ComputerUseManager>> =
        once_cell::sync::Lazy::new(|| {
            tokio::sync::Mutex::new(crate::computer_use::ComputerUseManager::new(
                crate::computer_use::ComputerUseConfig::default()
            ))
        });
    let mut guard = SHARED_COMPUTER_USE.lock().await;
    let manager = &mut *guard;
    let action = crate::computer_use::ComputerAction {
        action_type: match req.action_type.as_str() {
            "mouse_move" => crate::computer_use::ComputerActionType::MouseMove,
            "mouse_click" => crate::computer_use::ComputerActionType::MouseClick,
            "mouse_down" => crate::computer_use::ComputerActionType::MouseDown,
            "mouse_up" => crate::computer_use::ComputerActionType::MouseUp,
            "mouse_scroll" => crate::computer_use::ComputerActionType::MouseScroll,
            "key_press" => crate::computer_use::ComputerActionType::KeyPress,
            "key_down" => crate::computer_use::ComputerActionType::KeyDown,
            "key_up" => crate::computer_use::ComputerActionType::KeyUp,
            "type_text" => crate::computer_use::ComputerActionType::TypeText,
            "screenshot" => crate::computer_use::ComputerActionType::Screenshot,
            "wait" => crate::computer_use::ComputerActionType::Wait,
            _ => crate::computer_use::ComputerActionType::Wait,
        },
        coordinate: req.coordinate.map(|c| crate::computer_use::ScreenCoordinate { x: c[0], y: c[1] }),
        button: req.button.map(|b| match b.as_str() {
            "right" => crate::computer_use::MouseButton::Right,
            "middle" => crate::computer_use::MouseButton::Middle,
            _ => crate::computer_use::MouseButton::Left,
        }),
        key: req.key,
        text: req.text,
        scroll_y: req.scroll_y,
        scroll_x: req.scroll_x,
        duration_ms: req.duration_ms,
    };
    match manager.execute_action(action).await {
        Ok(result) => Json(serde_json::json!({
            "ok": result.success,
            "screenshot": result.screenshot,
            "error": result.error,
            "durationMs": result.duration_ms,
        })),
        Err(e) => Json(serde_json::json!({ "ok": false, "error": format!("{}", e) })),
    }
}

async fn computer_use_screenshot() -> Json<serde_json::Value> {
    let manager = crate::computer_use::ComputerUseManager::new(crate::computer_use::ComputerUseConfig::default());
    let action = crate::computer_use::ComputerAction {
        action_type: crate::computer_use::ComputerActionType::Screenshot,
        coordinate: None,
        button: None,
        key: None,
        text: None,
        scroll_y: None,
        scroll_x: None,
        duration_ms: None,
    };
    match manager.execute_action(action).await {
        Ok(result) => Json(serde_json::json!({ "ok": result.success, "screenshot": result.screenshot, "error": result.error })),
        Err(e) => Json(serde_json::json!({ "ok": false, "error": format!("{}", e) })),
    }
}

async fn git_status_handler(_state: State<AppState>, Query(query): Query<GitRequest>) -> Json<serde_json::Value> {
    let git = GitIntegration::with_cwd(query.cwd);
    match git.get_status() {
        Ok(status) => Json(serde_json::json!({ "status": status })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

async fn git_log_handler(_state: State<AppState>, Query(query): Query<GitRequest>) -> Json<serde_json::Value> {
    let git = GitIntegration::with_cwd(query.cwd);
    match git.get_commits(Some(10), None) {
        Ok(commits) => Json(serde_json::json!({ "commits": commits })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

async fn git_diff_handler(_state: State<AppState>, Query(query): Query<GitRequest>) -> Json<serde_json::Value> {
    let git = GitIntegration::with_cwd(query.cwd);
    match git.get_file_diff(query.file.as_deref()) {
        Ok(diff) => Json(serde_json::json!({ "diff": diff })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

async fn git_commit_handler(_state: State<AppState>, Json(req): Json<GitRequest>) -> Result<Json<serde_json::Value>, StatusCode> {
    let git = GitIntegration::with_cwd(req.cwd);
    let message = req.message.ok_or_else(|| StatusCode::BAD_REQUEST)?;

    match git.commit(&message) {
        Ok(_) => Ok(Json(serde_json::json!({ "ok": true }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn git_push_handler(_state: State<AppState>, Json(req): Json<GitRequest>) -> Result<Json<serde_json::Value>, StatusCode> {
    let git = GitIntegration::with_cwd(req.cwd);
    match git.push(req.remote.as_deref(), req.branch.as_deref(), req.force.unwrap_or(false)) {
        Ok(output) => Ok(Json(serde_json::json!({ "ok": true, "output": output }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn git_pull_handler(_state: State<AppState>, Json(req): Json<GitRequest>) -> Result<Json<serde_json::Value>, StatusCode> {
    let git = GitIntegration::with_cwd(req.cwd);
    match git.pull(req.remote.as_deref(), req.branch.as_deref()) {
        Ok(output) => Ok(Json(serde_json::json!({ "ok": true, "output": output }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
pub struct TerminalCreateRequest {
    pub cwd: Option<String>,
    pub shell: Option<String>,
}

async fn terminal_create(State(state): State<AppState>, Json(req): Json<TerminalCreateRequest>) -> Json<serde_json::Value> {
    let terminal_manager = state.terminal_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, PtyManager> = terminal_manager.lock().await;
    match manager.create_session(req.cwd, req.shell).await {
        Ok(session) => Json(serde_json::to_value(session).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

#[derive(Deserialize)]
pub struct TerminalWriteRequest {
    pub session_id: String,
    pub data: String,
}

async fn terminal_write(State(state): State<AppState>, Json(req): Json<TerminalWriteRequest>) -> Json<serde_json::Value> {
    let terminal_manager = state.terminal_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, PtyManager> = terminal_manager.lock().await;
    match manager.write_input(&req.session_id, &req.data).await {
        Ok(_) => Json(serde_json::json!({ "ok": true })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

#[derive(Deserialize)]
pub struct TerminalResizeRequest {
    pub session_id: String,
    pub cols: u16,
    pub rows: u16,
}

async fn terminal_resize(State(state): State<AppState>, Json(req): Json<TerminalResizeRequest>) -> Json<serde_json::Value> {
    let terminal_manager = state.terminal_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, PtyManager> = terminal_manager.lock().await;
    match manager.resize(&req.session_id, req.cols, req.rows).await {
        Ok(_) => Json(serde_json::json!({ "ok": true })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

async fn terminal_close(State(state): State<AppState>, Json(session_id): Json<String>) -> Json<serde_json::Value> {
    let terminal_manager = state.terminal_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, PtyManager> = terminal_manager.lock().await;
    match manager.close_session(&session_id).await {
        Ok(_) => Json(serde_json::json!({ "ok": true })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

async fn terminal_list(State(state): State<AppState>) -> Json<serde_json::Value> {
    let terminal_manager = state.terminal_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, PtyManager> = terminal_manager.lock().await;
    let sessions = manager.list_sessions().await;
    Json(serde_json::json!({ "sessions": sessions }))
}

async fn terminal_output_stream(Path(session_id): Path<String>, State(state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    let terminal_manager = state.terminal_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, PtyManager> = terminal_manager.lock().await;

    let receiver = manager.subscribe(&session_id)
        .await
        .ok_or_else(|| StatusCode::NOT_FOUND)?;

    let idle_timeout = Duration::from_secs(SSE_IDLE_TIMEOUT_SECS);
    let max_duration = Duration::from_secs(SSE_MAX_DURATION_SECS);
    let mut last_activity = std::time::Instant::now();

    let stream = async_stream::stream! {
        let mut rx = receiver;
        let max_sleep = tokio::time::sleep(max_duration);
        tokio::pin!(max_sleep);

        loop {
            let idle_sleep = tokio::time::sleep(idle_timeout);
            tokio::pin!(idle_sleep);

            tokio::select! {
                _ = &mut max_sleep => {
                    break;
                }
                _ = &mut idle_sleep => {
                    if last_activity.elapsed() >= idle_timeout {
                        break;
                    }
                }
                result = rx.recv() => {
                    match result {
                        Ok(output) => {
                            last_activity = std::time::Instant::now();
                            let data = serde_json::to_string(&output).unwrap_or_default();
                            yield Ok::<Event, Infallible>(Event::default()
                                .event("data")
                                .data(data));
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            tracing::error!(module = "SSE_terminal_output", "Receiver lagged, dropped {} events", n);
                            last_activity = std::time::Instant::now();
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            yield Ok::<Event, Infallible>(Event::default()
                                .event("exit")
                                .data(r#"{"code":0}"#));
                            break;
                        }
                    }
                }
            }
        }
    };

    let mut response = Sse::new(stream).keep_alive(KeepAlive::default()).into_response();
    set_sse_content_type(&mut response);
    Ok(response)
}

#[derive(Deserialize)]
struct ProcessSpawnRequest {
    command: String,
    cwd: Option<String>,
    env_vars: Option<HashMap<String, String>>,
}

async fn process_spawn(State(state): State<AppState>, Json(req): Json<ProcessSpawnRequest>) -> Json<serde_json::Value> {
    let process_manager = state.process_manager.clone();
    // Bug #12 fix: ProcessManager is internally thread-safe (Arc<Mutex<HashMap>>),
    // so we don't need to hold the outer MutexGuard during the async spawn.
    let manager = process_manager.lock().await;
    match manager.spawn(&req.command, req.cwd.as_deref(), req.env_vars).await {
        Ok(info) => Json(serde_json::to_value(info).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

async fn process_kill(Path(pid): Path<u32>, State(state): State<AppState>) -> Json<serde_json::Value> {
    let process_manager = state.process_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, ProcessManager> = process_manager.lock().await;
    match manager.kill(pid).await {
        Ok(_) => Json(serde_json::json!({ "ok": true })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

async fn process_list(State(state): State<AppState>) -> Json<serde_json::Value> {
    let process_manager = state.process_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, ProcessManager> = process_manager.lock().await;
    let processes = manager.list_processes().await;
    Json(serde_json::json!({ "processes": processes }))
}

async fn clipboard_read(State(state): State<AppState>) -> Json<serde_json::Value> {
    let clipboard_manager = state.clipboard_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, ClipboardManager> = clipboard_manager.lock().await;
    match manager.read() {
        Ok(content) => Json(serde_json::to_value(content).unwrap_or_default()),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

#[derive(Deserialize)]
pub struct ClipboardWriteRequest {
    pub text: Option<String>,
}

async fn clipboard_write(State(state): State<AppState>, Json(req): Json<ClipboardWriteRequest>) -> Json<serde_json::Value> {
    let clipboard_manager = state.clipboard_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, ClipboardManager> = clipboard_manager.lock().await;
    let content = crate::clipboard::ClipboardContent {
        text: req.text,
        html: None,
        image: None,
    };
    match manager.write(&content) {
        Ok(_) => Json(serde_json::json!({ "ok": true })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

#[derive(Deserialize)]
pub struct NotificationRequest {
    pub title: String,
    pub body: String,
    pub urgency: Option<String>,
}

async fn notification_show(State(state): State<AppState>, Json(req): Json<NotificationRequest>) -> Json<serde_json::Value> {
    let notification_manager = state.notification_manager.clone();
    let manager: tokio::sync::MutexGuard<'_, NotificationManager> = notification_manager.lock().await;
    let options = crate::notification::NotificationOptions {
        title: req.title,
        body: req.body,
        icon: None,
        silent: None,
        urgency: req.urgency,
        timeout: None,
    };
    match manager.show(&options) {
        Ok(_) => Json(serde_json::json!({ "ok": true })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

#[derive(Deserialize)]
pub struct LogsReadRequest {
    pub level: Option<String>,
    pub source: Option<String>,
    pub search: Option<String>,
    pub limit: Option<usize>,
}

async fn logs_read(State(state): State<AppState>, Query(req): Query<LogsReadRequest>) -> Json<serde_json::Value> {
    let logger = state.logger.clone();
    let logger_guard: tokio::sync::MutexGuard<'_, Logger> = logger.lock().await;
    let filter = crate::logger::LogFilter {
        level: req.level,
        source: req.source,
        from_time: None,
        to_time: None,
        search: req.search,
    };
    match logger_guard.read_logs(Some(filter), req.limit.unwrap_or(100)) {
        Ok(entries) => Json(serde_json::json!({ "logs": entries })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

#[derive(Deserialize)]
pub struct LogsClearRequest {
    pub days: Option<u32>,
}

async fn logs_clear(State(state): State<AppState>, Json(req): Json<LogsClearRequest>) -> Json<serde_json::Value> {
    let logger = state.logger.clone();
    let logger_guard: tokio::sync::MutexGuard<'_, Logger> = logger.lock().await;
    match logger_guard.clear_old_logs(req.days.unwrap_or(30)) {
        Ok(_) => Json(serde_json::json!({ "ok": true })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

#[derive(Deserialize)]
pub struct AnalyticsQuery {
    pub days: Option<u32>,
}

async fn analytics_dashboard(State(state): State<AppState>, Query(req): Query<AnalyticsQuery>) -> Json<DashboardStats> {
    let analytics_store = state.analytics_store.clone();
    let context_manager = state.context_manager.clone();

    let caveman_stats = {
        let cm_guard = context_manager.lock().await;
        Some(cm_guard.get_caveman_stats().await)
    };

    let stats = analytics_store.get_dashboard_stats(req.days.unwrap_or(30), caveman_stats).await;
    Json(stats)
}

async fn analytics_track_event(State(state): State<AppState>, Json(req): Json<TrackEventRequest>) -> Json<serde_json::Value> {
    let analytics_store = state.analytics_store.clone();
    match analytics_store.track_event(&req).await {
        Ok(_) => Json(serde_json::json!({ "ok": true })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

#[derive(Deserialize)]
pub struct CavemanAddRequest {
    pub content: String,
    pub role: String,
}

async fn caveman_add_memory(State(state): State<AppState>, Json(req): Json<CavemanAddRequest>) -> Json<serde_json::Value> {
    let context_manager = state.context_manager.clone();
    let cm_guard = context_manager.lock().await;
    match cm_guard.add_to_caveman_memory(&req.content, &req.role).await {
        Ok(id) => Json(serde_json::json!({ "id": id })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

#[derive(Deserialize)]
pub struct CavemanQueryRequest {
    pub query: String,
    pub max_segments: Option<usize>,
}

async fn caveman_query_memory(State(state): State<AppState>, Json(req): Json<CavemanQueryRequest>) -> Json<serde_json::Value> {
    let context_manager = state.context_manager.clone();
    let cm_guard = context_manager.lock().await;
    let context = cm_guard.get_caveman_context(&req.query, req.max_segments.unwrap_or(5)).await;
    Json(serde_json::json!({ "context": context }))
}

#[derive(Deserialize)]
pub struct CavemanRLMFeedback {
    pub memory_id: String,
    pub was_useful: bool,
    pub context: String,
}

async fn caveman_rlm_feedback(State(state): State<AppState>, Json(req): Json<CavemanRLMFeedback>) -> Json<serde_json::Value> {
    let context_manager = state.context_manager.clone();
    let cm_guard = context_manager.lock().await;
    cm_guard.record_rlm_feedback(&req.memory_id, req.was_useful, &req.context).await;
    crate::metrics::record_rlm_feedback();
    Json(serde_json::json!({ "ok": true }))
}

async fn caveman_rlm_iterate(State(state): State<AppState>) -> Json<serde_json::Value> {
    let context_manager = state.context_manager.clone();
    let cm_guard = context_manager.lock().await;
    let updates = cm_guard.run_rlm_iteration().await;
    Json(serde_json::json!({ "updates": updates }))
}

async fn caveman_stats(State(state): State<AppState>) -> Json<CavemanRTKStats> {
    let context_manager = state.context_manager.clone();
    let cm_guard = context_manager.lock().await;
    let stats = cm_guard.get_caveman_stats().await;
    Json(stats)
}

async fn watcher_start(State(state): State<AppState>) -> Json<serde_json::Value> {
    let file_watcher = state.file_watcher.clone();
    let watcher: tokio::sync::MutexGuard<'_, FileWatcher> = file_watcher.lock().await;
    match watcher.start().await {
        Ok(_) => Json(serde_json::json!({ "ok": true })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

#[derive(Deserialize)]
pub struct WatcherWatchRequest {
    pub path: String,
}

async fn watcher_watch(State(state): State<AppState>, Json(req): Json<WatcherWatchRequest>) -> Json<serde_json::Value> {
    let file_watcher = state.file_watcher.clone();
    let watcher: tokio::sync::MutexGuard<'_, FileWatcher> = file_watcher.lock().await;
    match watcher.watch(&req.path).await {
        Ok(_) => Json(serde_json::json!({ "ok": true })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

async fn watcher_unwatch(State(state): State<AppState>, Json(req): Json<WatcherWatchRequest>) -> Json<serde_json::Value> {
    let file_watcher = state.file_watcher.clone();
    let watcher: tokio::sync::MutexGuard<'_, FileWatcher> = file_watcher.lock().await;
    match watcher.unwatch(&req.path).await {
        Ok(_) => Json(serde_json::json!({ "ok": true })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

async fn update_check() -> Json<serde_json::Value> {
    let updates_url = crate::config::api_endpoints::resolve_api_url(Some("clawparrot"), "updates");
    let updater = AutoUpdater::new(
        if updates_url.is_empty() { "https://clawparrot.com/updates" } else { &updates_url },
        env!("CARGO_PKG_VERSION"),
        std::path::PathBuf::from(std::env::temp_dir()).join("claude-desktop-updates"),
    );
    match updater.check_for_updates().await {
        Ok(Some(info)) => Json(serde_json::to_value(info).unwrap_or_default()),
        Ok(None) => Json(serde_json::json!({ "up_to_date": true })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

#[derive(Deserialize)]
pub struct UpdateDownloadRequest {
    pub url: String,
}

async fn update_download(Json(req): Json<UpdateDownloadRequest>) -> Json<serde_json::Value> {
    let updates_url = crate::config::api_endpoints::resolve_api_url(Some("clawparrot"), "updates");
    let updater = AutoUpdater::new(
        if updates_url.is_empty() { "https://clawparrot.com/updates" } else { &updates_url },
        env!("CARGO_PKG_VERSION"),
        std::path::PathBuf::from(std::env::temp_dir()).join("claude-desktop-updates"),
    );
    match updater.download_update(&req.url, None).await {
        Ok(path) => Json(serde_json::json!({ "path": path.to_string_lossy() })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

use crate::worktree::{WorktreeManager, CreateWorktreeRequest, MergeWorktreeRequest};
use crate::ide::{IdeBridge, IdeConfig};

static WORKTREE_MANAGER: once_cell::sync::Lazy<tokio::sync::Mutex<Option<WorktreeManager>>> =
    once_cell::sync::Lazy::new(|| tokio::sync::Mutex::new(None));

static IDE_BRIDGE: once_cell::sync::Lazy<tokio::sync::Mutex<Option<IdeBridge>>> =
    once_cell::sync::Lazy::new(|| tokio::sync::Mutex::new(None));

async fn worktree_create(Json(req): Json<CreateWorktreeRequest>) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut guard = WORKTREE_MANAGER.lock().await;
    if guard.is_none() {
        *guard = Some(WorktreeManager::with_cwd(None));
    }
    if let Some(mgr) = guard.as_ref() {
        match mgr.create_worktree(req).await {
            Ok(info) => Ok(Json(serde_json::json!({ "success": true, "worktree": info }))),
            Err(e) => {
                tracing::error!(module = "Worktree", "Create failed: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

async fn worktree_list() -> Result<Json<serde_json::Value>, StatusCode> {
    let guard = WORKTREE_MANAGER.lock().await;
    if let Some(mgr) = guard.as_ref() {
        match mgr.list_worktrees().await {
            Ok(list) => Ok(Json(serde_json::json!({ "success": true, "worktrees": list }))),
            Err(e) => {
                tracing::error!(module = "Worktree", "List failed: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Ok(Json(serde_json::json!({ "success": true, "worktrees": [] })))
    }
}

async fn worktree_get(Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    let guard = WORKTREE_MANAGER.lock().await;
    if let Some(mgr) = guard.as_ref() {
        match mgr.get_worktree(&id).await {
            Some(info) => Ok(Json(serde_json::json!({ "success": true, "worktree": info }))),
            None => Err(StatusCode::NOT_FOUND),
        }
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn worktree_remove(Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    let guard = WORKTREE_MANAGER.lock().await;
    if let Some(mgr) = guard.as_ref() {
        match mgr.remove_worktree(&id).await {
            Ok(()) => Ok(Json(serde_json::json!({ "success": true }))),
            Err(e) => {
                tracing::error!(module = "Worktree", "Remove failed: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn worktree_merge(Json(req): Json<MergeWorktreeRequest>) -> Result<Json<serde_json::Value>, StatusCode> {
    let guard = WORKTREE_MANAGER.lock().await;
    if let Some(mgr) = guard.as_ref() {
        match mgr.merge_worktree(req).await {
            Ok(output) => Ok(Json(serde_json::json!({ "success": true, "output": output }))),
            Err(e) => {
                tracing::error!(module = "Worktree", "Merge failed: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

async fn worktree_sync() -> Result<Json<serde_json::Value>, StatusCode> {
    let mut guard = WORKTREE_MANAGER.lock().await;
    if guard.is_none() {
        *guard = Some(WorktreeManager::with_cwd(None));
    }
    if let Some(mgr) = guard.as_ref() {
        match mgr.sync_from_git().await {
            Ok(list) => Ok(Json(serde_json::json!({ "success": true, "worktrees": list }))),
            Err(e) => {
                tracing::error!(module = "Worktree", "Sync failed: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

async fn agent_list() -> Result<Json<serde_json::Value>, StatusCode> {
    let guard = WORKTREE_MANAGER.lock().await;
    if let Some(mgr) = guard.as_ref() {
        let agents = mgr.list_agents().await;
        Ok(Json(serde_json::json!({ "success": true, "agents": agents })))
    } else {
        Ok(Json(serde_json::json!({ "success": true, "agents": [] })))
    }
}

async fn agent_get(Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    let guard = WORKTREE_MANAGER.lock().await;
    if let Some(mgr) = guard.as_ref() {
        match mgr.get_agent(&id).await {
            Some(info) => Ok(Json(serde_json::json!({ "success": true, "agent": info }))),
            None => Err(StatusCode::NOT_FOUND),
        }
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn agent_cancel(Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    let guard = WORKTREE_MANAGER.lock().await;
    if let Some(mgr) = guard.as_ref() {
        match mgr.cancel_agent(&id).await {
            Ok(()) => Ok(Json(serde_json::json!({ "success": true }))),
            Err(e) => {
                tracing::error!(module = "Agent", "Cancel failed: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn ide_status() -> Result<Json<serde_json::Value>, StatusCode> {
    let guard = IDE_BRIDGE.lock().await;
    if let Some(bridge) = guard.as_ref() {
        let status = bridge.get_status().await;
        Ok(Json(serde_json::json!({ "success": true, "status": status })))
    } else {
        Ok(Json(serde_json::json!({
            "success": true,
            "status": { "server_running": false, "port": 0, "active_connections": 0, "total_connections": 0 }
        })))
    }
}

async fn ide_start() -> Result<Json<serde_json::Value>, StatusCode> {
    let mut guard = IDE_BRIDGE.lock().await;
    if guard.is_none() {
        *guard = Some(IdeBridge::new(IdeConfig::default()));
    }
    if let Some(bridge) = guard.as_ref() {
        match bridge.start_server().await {
            Ok(port) => Ok(Json(serde_json::json!({ "success": true, "port": port }))),
            Err(e) => {
                tracing::error!(module = "IDE", "Start failed: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

async fn ide_stop() -> Result<Json<serde_json::Value>, StatusCode> {
    let guard = IDE_BRIDGE.lock().await;
    if let Some(bridge) = guard.as_ref() {
        bridge.stop_server().await;
        Ok(Json(serde_json::json!({ "success": true })))
    } else {
        Ok(Json(serde_json::json!({ "success": true })))
    }
}

async fn ide_connections() -> Result<Json<serde_json::Value>, StatusCode> {
    let guard = IDE_BRIDGE.lock().await;
    if let Some(bridge) = guard.as_ref() {
        let conns = bridge.list_connections().await;
        Ok(Json(serde_json::json!({ "success": true, "connections": conns })))
    } else {
        Ok(Json(serde_json::json!({ "success": true, "connections": [] })))
    }
}

async fn ide_disconnect(Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    let guard = IDE_BRIDGE.lock().await;
    if let Some(bridge) = guard.as_ref() {
        match bridge.disconnect(&id).await {
            Ok(()) => Ok(Json(serde_json::json!({ "success": true }))),
            Err(e) => {
                tracing::error!(module = "IDE", "Disconnect failed: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// Bug #9 fix: Removed dead global ANALYTICS_STORE and analytics_track function.
// All analytics tracking now goes through AppState's analytics_store (state.analytics_store)
// via analytics_track_event, ensuring consistent data storage.

async fn analytics_daily(State(state): State<AppState>, Path(date): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    let store = state.analytics_store.clone();
    match store.get_daily_stats(&date).await {
        Some(stats) => Ok(Json(serde_json::json!({ "success": true, "stats": stats }))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn analytics_range(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<Json<serde_json::Value>, StatusCode> {
    let store = state.analytics_store.clone();
    let from = params.get("from").map(|s| s.as_str()).unwrap_or("2025-01-01");
    let to = params.get("to").map(|s| s.as_str()).unwrap_or("2099-12-31");
    let stats = store.get_stats_range(from, to).await;
    Ok(Json(serde_json::json!({ "success": true, "stats": stats })))
}
async fn analytics_summary(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<Json<serde_json::Value>, StatusCode> {
    let store = state.analytics_store.clone();
    let days: u32 = params.get("days").and_then(|d| d.parse().ok()).unwrap_or(30);
    let summary = store.get_usage_summary(days).await;
    Ok(Json(serde_json::json!({ "success": true, "summary": summary })))
}

async fn analytics_event_counts(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<Json<serde_json::Value>, StatusCode> {
    let store = state.analytics_store.clone();
    let days: u32 = params.get("days").and_then(|d| d.parse().ok()).unwrap_or(30);
    let counts = store.get_event_type_counts(days);
    Ok(Json(serde_json::json!({ "success": true, "counts": counts })))
}

async fn analytics_recent_events(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<Json<serde_json::Value>, StatusCode> {
    let store = state.analytics_store.clone();
    let limit: usize = params.get("limit").and_then(|d| d.parse().ok()).unwrap_or(50);
    let events = store.get_recent_events(limit);
    Ok(Json(serde_json::json!({ "success": true, "events": events })))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowExecuteRequest {
    pub goal: String,
    pub provider_id: Option<String>,
    pub model: Option<String>,
}

async fn workflow_execute(
    State(state): State<AppState>,
    Json(req): Json<WorkflowExecuteRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let config_manager: Arc<Mutex<Option<ConfigManager>>> = state.config_manager.clone();
    let orchestrator: Arc<Mutex<Option<crate::orchestration::MultiAgentOrchestrator>>> = state.orchestrator.clone();

    let orchestrator_guard: tokio::sync::MutexGuard<'_, Option<crate::orchestration::MultiAgentOrchestrator>> = orchestrator.lock().await;
    let orchestrator = orchestrator_guard.as_ref()
        .ok_or_else(|| {
            tracing::error!(module = "Bridge", "Orchestrator not initialized");
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    let config_guard: tokio::sync::MutexGuard<'_, Option<ConfigManager>> = config_manager.lock().await;
    let config = config_guard.as_ref()
        .ok_or_else(|| {
            tracing::error!(module = "Bridge", "Config manager not initialized");
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    let provider_config = match req.provider_id {
        Some(id) => config.get_provider(&id),
        None => config.get_default_provider(),
    };

    let provider_config = provider_config
        .ok_or_else(|| {
            tracing::error!(module = "Bridge", "No provider configured");
            StatusCode::BAD_REQUEST
        })?;

    let model_config = provider_config.models.first()
        .ok_or_else(|| {
            tracing::error!(module = "Bridge", "No model configured for provider");
            StatusCode::BAD_REQUEST
        })?;

    let api_format = if provider_config.provider_type.to_lowercase() == "anthropic" {
        crate::native_engine::provider_manager::ApiFormat::Anthropic
    } else {
        crate::native_engine::provider_manager::ApiFormat::OpenAI
    };

    let provider = crate::native_engine::provider_manager::Provider {
        id: provider_config.id.clone(),
        name: provider_config.name.clone(),
        base_url: provider_config.base_url.clone(),
        api_key: provider_config.api_key.clone().unwrap_or_default(),
        api_format,
        models: provider_config.models.iter().map(|m| crate::native_engine::provider_manager::ModelConfig {
            id: m.id.clone(),
            name: m.name.clone(),
            enabled: m.enabled,
            max_tokens: m.max_tokens,
            supports_vision: m.supports_vision,
            supports_web_search: false,
        }).collect(),
        enabled: provider_config.enabled,
        web_search_strategy: provider_config.web_search_strategy.clone(),
    };

    let model = crate::native_engine::provider_manager::ModelConfig {
        id: model_config.id.clone(),
        name: model_config.name.clone(),
        enabled: model_config.enabled,
        max_tokens: model_config.max_tokens,
        supports_vision: model_config.supports_vision,
        supports_web_search: false,
    };

    let resolved_provider = crate::native_engine::provider_manager::ResolvedProvider {
        provider,
        model,
    };

    match orchestrator.execute_workflow(&req.goal, &resolved_provider).await {
        Ok(result) => Ok(Json(serde_json::json!({ "success": true, "result": result }))),
        Err(e) => {
            tracing::error!(module = "Bridge", "Workflow execution failed: {}", e);
            Ok(Json(serde_json::json!({ "success": false, "error": format!("{}", e) })))
        }
    }
}

async fn workflow_execute_stream(
    State(state): State<AppState>,
    Json(req): Json<WorkflowExecuteRequest>,
) -> Result<Sse<impl futures::Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    let config_manager: Arc<Mutex<Option<ConfigManager>>> = state.config_manager.clone();
    let orchestrator: Arc<Mutex<Option<crate::orchestration::MultiAgentOrchestrator>>> = state.orchestrator.clone();

    let orchestrator_guard: tokio::sync::MutexGuard<'_, Option<crate::orchestration::MultiAgentOrchestrator>> = orchestrator.lock().await;
    let orchestrator = orchestrator_guard.as_ref()
        .ok_or_else(|| {
            tracing::error!(module = "Bridge", "Orchestrator not initialized");
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    let config_guard: tokio::sync::MutexGuard<'_, Option<ConfigManager>> = config_manager.lock().await;
    let config = config_guard.as_ref()
        .ok_or_else(|| {
            tracing::error!(module = "Bridge", "Config manager not initialized");
            StatusCode::SERVICE_UNAVAILABLE
        })?;

    let provider_config = match req.provider_id {
        Some(ref id) => config.get_provider(id),
        None => config.get_default_provider(),
    };

    let provider_config = provider_config
        .ok_or_else(|| {
            tracing::error!(module = "Bridge", "No provider configured");
            StatusCode::BAD_REQUEST
        })?;

    let model_config = provider_config.models.first()
        .ok_or_else(|| {
            tracing::error!(module = "Bridge", "No model configured for provider");
            StatusCode::BAD_REQUEST
        })?;

    let api_format = if provider_config.provider_type.to_lowercase() == "anthropic" {
        crate::native_engine::provider_manager::ApiFormat::Anthropic
    } else {
        crate::native_engine::provider_manager::ApiFormat::OpenAI
    };

    let provider = crate::native_engine::provider_manager::Provider {
        id: provider_config.id.clone(),
        name: provider_config.name.clone(),
        base_url: provider_config.base_url.clone(),
        api_key: provider_config.api_key.clone().unwrap_or_default(),
        api_format,
        models: provider_config.models.iter().map(|m| crate::native_engine::provider_manager::ModelConfig {
            id: m.id.clone(),
            name: m.name.clone(),
            enabled: m.enabled,
            max_tokens: m.max_tokens,
            supports_vision: m.supports_vision,
            supports_web_search: false,
        }).collect(),
        enabled: provider_config.enabled,
        web_search_strategy: provider_config.web_search_strategy.clone(),
    };

    let model = crate::native_engine::provider_manager::ModelConfig {
        id: model_config.id.clone(),
        name: model_config.name.clone(),
        enabled: model_config.enabled,
        max_tokens: model_config.max_tokens,
        supports_vision: model_config.supports_vision,
        supports_web_search: false,
    };

    let resolved_provider = crate::native_engine::provider_manager::ResolvedProvider {
        provider,
        model,
    };

    let goal = req.goal.clone();
    let (stream_tx, mut stream_rx) = broadcast::channel::<AgentStreamEvent>(256);

    let orchestrator_clone = orchestrator.clone();
    tokio::spawn(async move {
        if let Err(e) = orchestrator_clone.execute_workflow_streaming(&goal, &resolved_provider, stream_tx).await {
            tracing::error!(module = "Bridge", "Streaming workflow failed: {}", e);
        }
    });

    let sse_stream = async_stream::stream! {
        while let Ok(event) = stream_rx.recv().await {
            let data = serde_json::to_string(&event).unwrap_or_default();
            if event.event_type == "agent_done" {
                yield Ok::<Event, Infallible>(Event::default().data("[DONE]"));
                break;
            }
            yield Ok::<Event, Infallible>(Event::default().data(data).event("agent_event"));
        }
    };

    let sse = Sse::new(sse_stream)
        .keep_alive(KeepAlive::default());

    Ok(sse)
}

async fn workflow_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let orchestrator: Arc<Mutex<Option<crate::orchestration::MultiAgentOrchestrator>>> = state.orchestrator.clone();

    let orchestrator_guard: tokio::sync::MutexGuard<'_, Option<crate::orchestration::MultiAgentOrchestrator>> = orchestrator.lock().await;
    if let Some(orchestrator) = orchestrator_guard.as_ref() {
        let stats: serde_json::Value = orchestrator.get_scheduling_stats().await;
        Ok(Json(stats))
    } else {
        Ok(Json(serde_json::json!({ "success": false, "error": "Orchestrator not initialized" })))
    }
}

async fn workflow_config_get() -> Result<Json<serde_json::Value>, StatusCode> {
    let config_path = std::path::Path::new("config/orchestration.toml");
    let config = OrchestratorConfigFile::load_or_default(config_path);
    Ok(Json(serde_json::json!({ "success": true, "config": config })))
}

async fn workflow_config_set(
    Json(config): Json<OrchestratorConfigFile>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let config_path = std::path::Path::new("config/orchestration.toml");
    let config_dir = config_path.parent().unwrap_or(std::path::Path::new("."));
    std::fs::create_dir_all(config_dir).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    config.save_to_file(config_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "success": true })))
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Memory Handlers
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

async fn memory_search_handler(
    State(state): State<AppState>,
    Json(req): Json<MemorySearchRequest>,
) -> Result<Json<Vec<MemoryItem>>, (StatusCode, String)> {
    let memex: Arc<MemExClient> = state.memex_client.clone();
    match memex.search(&req.query, req.top_k).await {
        Ok(results) => Ok(Json(results)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn memory_ingest_handler(
    State(state): State<AppState>,
    Json(req): Json<MemoryIngestRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let memex: Arc<MemExClient> = state.memex_client.clone();
    match memex.ingest(&req.content, req.importance, req.metadata).await {
        Ok(()) => Ok(Json(json!({"status": "ok"}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn memory_stats_handler(
    State(state): State<AppState>,
) -> Result<Json<MemoryStats>, (StatusCode, String)> {
    let memex: Arc<MemExClient> = state.memex_client.clone();
    match memex.stats().await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

async fn memory_get_config_handler(
    State(state): State<AppState>,
) -> Json<MemoryConfig> {
    let memex: Arc<MemExClient> = state.memex_client.clone();
    Json(memex.get_config().await)
}

async fn memory_update_config_handler(
    State(state): State<AppState>,
    Json(config): Json<MemoryConfig>,
) -> Json<serde_json::Value> {
    let memex: Arc<MemExClient> = state.memex_client.clone();
    memex.update_config(config).await;
    Json(json!({"status": "updated"}))
}

async fn memory_clear_handler(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let memex: Arc<MemExClient> = state.memex_client.clone();
    match memex.clear().await {
        Ok(()) => Ok(Json(json!({"status": "cleared"}))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[derive(Deserialize)]
struct FsPathQuery {
    path: Option<String>,
}

#[derive(Deserialize)]
struct FsWriteRequest {
    path: String,
    content: String,
}

#[derive(Deserialize)]
struct FsCreateRequest {
    path: String,
    #[serde(default)]
    is_dir: bool,
}

fn build_tree(dir_path: &str, max_depth: u32, visited: &mut Vec<String>) -> Result<serde_json::Value, StatusCode> {
    if max_depth == 0 {
        return Ok(serde_json::json!([{"name": "... (max depth reached)", "path": "", "is_dir": false, "size": 0}]));
    }

    let canonical = std::path::Path::new(dir_path)
        .canonicalize()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let canonical_str = canonical.to_string_lossy().to_string();

    // Prevent symlink cycles
    if visited.contains(&canonical_str) {
        return Ok(serde_json::json!([{"name": "... (symlink cycle)", "path": "", "is_dir": false, "size": 0}]));
    }
    visited.push(canonical_str);

    let entries = FileOperations::list_directory(dir_path, false)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut children = Vec::new();
    for entry in entries {
        let mut node = serde_json::json!({
            "name": entry.name,
            "path": entry.path,
            "is_dir": entry.is_dir,
            "size": entry.size,
        });

        if entry.is_dir {
            node["children"] = build_tree(&entry.path, max_depth - 1, visited).unwrap_or(serde_json::json!([]));
        }

        children.push(node);
    }

    Ok(serde_json::json!(children))
}

async fn fs_tree(Query(query): Query<FsPathQuery>) -> Result<Json<serde_json::Value>, StatusCode> {
    let path = query.path.unwrap_or_else(|| {
        dirs::home_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string())
    });

    if !FileOperations::is_directory(&path) && !FileOperations::exists(&path) {
        return Err(StatusCode::NOT_FOUND);
    }

    let tree = build_tree(&path, 10, &mut Vec::new()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "success": true, "path": path, "tree": tree })))
}

async fn fs_read(Query(query): Query<FsPathQuery>) -> Result<Json<serde_json::Value>, StatusCode> {
    let path = query.path.ok_or(StatusCode::BAD_REQUEST)?;

    let content = FileOperations::read_file(&path, None, None)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "success": true, "path": path, "content": content })))
}

async fn fs_read_encoding(Query(query): Query<FsPathQuery>) -> Result<Json<serde_json::Value>, StatusCode> {
    let path = query.path.ok_or(StatusCode::BAD_REQUEST)?;
    let result = FileOperations::read_file_with_encoding(&path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({
        "success": true, "path": path,
        "content": result["content"], "encoding": result["encoding"], "size": result["size"]
    })))
}

async fn fs_download(Query(query): Query<FsPathQuery>) -> Result<Json<serde_json::Value>, StatusCode> {
    let path = query.path.ok_or(StatusCode::BAD_REQUEST)?;
    let result = FileOperations::read_file_base64(&path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(serde_json::json!({ "success": true, "data": result })))
}

async fn fs_write(Json(req): Json<FsWriteRequest>) -> Result<Json<serde_json::Value>, StatusCode> {
    FileOperations::write_file(&req.path, &req.content)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "success": true, "path": req.path })))
}

async fn fs_create(Json(req): Json<FsCreateRequest>) -> Result<Json<serde_json::Value>, StatusCode> {
    if req.is_dir {
        FileOperations::create_directory(&req.path)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    } else {
        FileOperations::write_file(&req.path, "")
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(serde_json::json!({ "success": true, "path": req.path })))
}

async fn fs_delete(Json(req): Json<FsCreateRequest>) -> Result<Json<serde_json::Value>, StatusCode> {
    FileOperations::delete_file(&req.path)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "success": true, "path": req.path })))
}
// Dead-code H5 and IM handlers removed (compiled versions in bridge/routes/h5.rs and bridge/routes/im.rs)

#[cfg(test)]
mod tests;
