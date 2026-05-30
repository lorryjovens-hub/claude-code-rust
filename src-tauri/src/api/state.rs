//! Structured application state for the Axum HTTP bridge.
//!
//! Replaces the 24-element tuple `AppState` with named fields,
//! so handlers reference `state.db_manager` instead of `state.6`.

use crate::analytics::AnalyticsStore;
use crate::clipboard::ClipboardManager;
use crate::config::ConfigManager;
use crate::cost_tracker::CostTracker;
use crate::db::DbManager;
use crate::engine::EnginePool;
use crate::git::GitIntegration;
use crate::logger::Logger;
use crate::mcp::McpServerManager;
use crate::memory::{ContextManager, MemExClient};
use crate::native_engine::NativeEngine;
use crate::notification::NotificationManager;
use crate::orchestration::MultiAgentOrchestrator;
use crate::permissions::PermissionManager;
use crate::preview_engine::PreviewEngine;
use crate::process::ProcessManager;
use crate::skills::SkillsManager;
use crate::streaming::StreamManager;
use crate::terminal::PtyManager;
use crate::watcher::FileWatcher;
use crate::bridge::{CsrfTokenStore, RateLimiter, ResearchTask};
use crate::bridge::interceptor::{MetricsInterceptor, RequestInterceptor};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Structured application state with named fields.
///
/// Usage: extract in handlers via `State(state): State<AppState>` then
/// access `state.db_manager`, `state.mcp_manager`, etc.
#[derive(Clone)]
pub struct AppState {
    pub engine_pool: Arc<Mutex<EnginePool>>,
    pub mcp_manager: Arc<McpServerManager>,
    pub stream_manager: Arc<Mutex<StreamManager>>,
    pub research_mode: Arc<Mutex<HashMap<String, bool>>>,
    pub config_manager: Arc<Mutex<Option<ConfigManager>>>,
    pub skills_manager: Arc<Mutex<SkillsManager>>,
    pub db_manager: Arc<DbManager>,
    pub task_executor: Arc<Mutex<Option<crate::task::TaskExecutor>>>,
    pub process_manager: Arc<Mutex<ProcessManager>>,
    pub terminal_manager: Arc<Mutex<PtyManager>>,
    pub file_watcher: Arc<Mutex<FileWatcher>>,
    pub clipboard_manager: Arc<Mutex<ClipboardManager>>,
    pub notification_manager: Arc<Mutex<NotificationManager>>,
    pub logger: Arc<Mutex<Logger>>,
    pub native_engine: Arc<Mutex<Option<NativeEngine>>>,
    pub active_research: Arc<Mutex<HashMap<String, ResearchTask>>>,
    pub orchestrator: Arc<Mutex<Option<MultiAgentOrchestrator>>>,
    pub memex_client: Arc<MemExClient>,
    pub api_key: String,
    pub rate_limiter: Arc<RateLimiter>,
    pub cost_tracker: Arc<CostTracker>,
    pub preview_engine: Arc<PreviewEngine>,
    pub analytics_store: Arc<AnalyticsStore>,
    pub context_manager: Arc<Mutex<ContextManager>>,
    pub git_integration: Arc<GitIntegration>,
    pub permission_manager: Arc<PermissionManager>,
    pub csrf_token_store: Arc<CsrfTokenStore>,
    pub request_interceptor: Arc<RequestInterceptor>,
    pub metrics_interceptor: Arc<MetricsInterceptor>,
}

impl AppState {
    /// Create a new `AppState` by extracting from the legacy tuple-based state.
    /// This allows incremental migration of handlers to the new struct.
    #[allow(dead_code)]
    pub fn from_tuple(state: &crate::bridge::AppStateTuple) -> Self {
        Self {
            engine_pool: state.0.clone(),
            mcp_manager: state.1.clone(),
            stream_manager: state.2.clone(),
            research_mode: state.3.clone(),
            config_manager: state.4.clone(),
            skills_manager: state.5.clone(),
            db_manager: state.6.clone(),
            task_executor: state.7.clone(),
            process_manager: state.8.clone(),
            terminal_manager: state.9.clone(),
            file_watcher: state.10.clone(),
            clipboard_manager: state.11.clone(),
            notification_manager: state.12.clone(),
            logger: state.13.clone(),
            native_engine: state.14.clone(),
            active_research: state.15.clone(),
            orchestrator: state.16.clone(),
            memex_client: state.17.clone(),
            api_key: state.18.clone(),
            rate_limiter: state.19.clone(),
            cost_tracker: state.20.clone(),
            preview_engine: state.21.clone(),
            analytics_store: state.22.clone(),
            context_manager: state.23.clone(),
            git_integration: Arc::new(GitIntegration::new(
                std::env::current_dir().unwrap_or_default(),
            )),
            permission_manager: Arc::new(PermissionManager::new(
                Arc::new(crate::permissions::AuditLogger::new(1000)),
            )),
            csrf_token_store: Arc::new(CsrfTokenStore::new()),
            request_interceptor: Arc::new(RequestInterceptor::new()),
            metrics_interceptor: Arc::new(MetricsInterceptor::new()),
        }
    }
}
