use crate::cost_tracker::{BudgetCheckResult, CostTracker};
use crate::db::DbManager;
use crate::native_engine::provider_manager::ProviderManager;
use crate::native_engine::tool_loop::{EngineEvent, ToolLoopExecutor};
use crate::mcp::McpToolRegistry;
use crate::permissions::{PermissionContext, PermissionManager, PermissionResult, audit::{AuditAction, AuditEntry}};
use crate::prefetch::PrefetchEngine;
use crate::skills::{SkillExecutionContext, SkillsManager};
use crate::cache::FileCache;
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, Mutex, oneshot};
use futures::FutureExt;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReasoningMode {
    Quick,
    Standard,
    Deep,
}

pub fn analyze_complexity(message: &str) -> ReasoningMode {
    let msg_lower = message.to_lowercase();
    let has_code = msg_lower.contains("function") || msg_lower.contains("class ") || msg_lower.contains("def ") || msg_lower.contains("import ") || msg_lower.contains("const ") || msg_lower.contains("let ") || msg_lower.contains("fn ");
    let has_file_op = msg_lower.contains("file") || msg_lower.contains("编辑") || msg_lower.contains("修改") || msg_lower.contains("删除") || msg_lower.contains("write") || msg_lower.contains("read") || msg_lower.contains("目录");
    let has_danger = msg_lower.contains("delete") || msg_lower.contains("rm ") || msg_lower.contains("drop ") || msg_lower.contains("删除") || msg_lower.contains("格式化") || msg_lower.contains("format");
    let is_long = message.len() > 200;

    if has_danger {
        ReasoningMode::Deep
    } else if has_code || has_file_op || is_long {
        ReasoningMode::Standard
    } else {
        ReasoningMode::Quick
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub conversation_id: String,
    pub messages: Vec<Value>,
    pub model: String,
    pub system_prompt: Option<String>,
    pub max_tokens: Option<u32>,
    pub workspace_path: Option<String>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub reasoning_mode: Option<ReasoningMode>,
}

#[derive(Debug)]
pub struct ActiveTurn {
    pub event_tx: mpsc::Sender<EngineEvent>,
    pub executor_handle: Option<tokio::task::JoinHandle<()>>,
    pub cancelled: bool,
}

#[derive(Debug, Clone)]
pub struct ConversationState {
    pub conversation_id: String,
    pub model: String,
    pub messages: Vec<Value>,
    pub system_prompt: Option<String>,
    pub last_activity: String,
    pub turn_count: usize,
    pub status: ConversationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConversationStatus {
    Idle,
    Active,
    WaitingForUser,
    Completed,
    Error,
}

#[derive(Debug, Clone)]
pub struct ToolCallRecord {
    pub tool_use_id: String,
    pub tool_name: String,
    pub tool_input: Value,
    pub output: String,
    pub is_error: bool,
    pub timestamp: String,
}

pub struct QueryEngine {
    provider_manager: Arc<Mutex<ProviderManager>>,
    db_manager: Arc<DbManager>,
    active_turns: Arc<Mutex<HashMap<String, ActiveTurn>>>,
    conversation_states: Arc<Mutex<HashMap<String, ConversationState>>>,
    workspaces_dir: PathBuf,
    mcp_registry: Option<Arc<McpToolRegistry>>,
    answer_waiters: Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>,
    tool_call_history: Arc<Mutex<HashMap<String, Vec<ToolCallRecord>>>>,
    permission_manager: Arc<PermissionManager>,
    skills_manager: Option<Arc<Mutex<SkillsManager>>>,
    file_cache: Arc<FileCache>,
    cost_tracker: Option<Arc<CostTracker>>,
}

impl QueryEngine {
    pub fn new(
        provider_manager: Arc<Mutex<ProviderManager>>,
        db_manager: Arc<DbManager>,
        workspaces_dir: PathBuf,
        permission_manager: Arc<PermissionManager>,
        file_cache: Arc<FileCache>,
    ) -> Self {
        Self {
            provider_manager,
            db_manager,
            active_turns: Arc::new(Mutex::new(HashMap::new())),
            conversation_states: Arc::new(Mutex::new(HashMap::new())),
            workspaces_dir,
            mcp_registry: None,
            answer_waiters: Arc::new(Mutex::new(HashMap::new())),
            tool_call_history: Arc::new(Mutex::new(HashMap::new())),
            permission_manager,
            skills_manager: None,
            file_cache,
            cost_tracker: None,
        }
    }

    pub fn with_mcp_registry(mut self, registry: Arc<McpToolRegistry>) -> Self {
        self.mcp_registry = Some(registry);
        self
    }

    pub fn with_skills_manager(mut self, manager: Arc<Mutex<SkillsManager>>) -> Self {
        self.skills_manager = Some(manager);
        self
    }

    pub fn with_cost_tracker(mut self, tracker: Arc<CostTracker>) -> Self {
        self.cost_tracker = Some(tracker);
        self
    }

    pub async fn check_tool_permission(
        &self,
        tool_name: &str,
        tool_input: Value,
        conversation_id: &str,
        workspace_path: Option<String>,
    ) -> PermissionResult {
        let context = PermissionContext {
            tool_name: tool_name.to_string(),
            tool_input,
            conversation_id: conversation_id.to_string(),
            user_id: None,
            workspace_path,
        };
        self.permission_manager.check_permission(&context)
    }

    pub async fn execute_skill(
        &self,
        skill_id: &str,
        conversation_id: &str,
        messages: Vec<Value>,
        workspace_path: Option<String>,
    ) -> Result<String> {
        let skills_manager = self.skills_manager.as_ref()
            .ok_or_else(|| anyhow!("Skills manager not configured"))?;

        let manager = skills_manager.lock().await;

        let context = SkillExecutionContext {
            conversation_id: conversation_id.to_string(),
            messages,
            available_tools: crate::tools::get_tool_definitions(),
            available_mcp_tools: Vec::new(),
            current_input: "".to_string(),
            workspace_path,
            variables: std::collections::HashMap::new(),
            mcp_server_manager: None,
        };

        manager.execute_skill(skill_id, "", Some(context)).await
    }

    pub async fn confirm_tool_permission(&self, conversation_id: &str, tool_name: &str) {
        self.permission_manager.confirm_permission(conversation_id, tool_name);
    }

    pub async fn set_permission_mode(&self, mode: crate::permissions::PermissionMode) {
        self.permission_manager.set_mode(mode);
    }

    pub async fn sync_providers(&self, providers: Vec<crate::native_engine::provider_manager::Provider>) {
        let mut pm = self.provider_manager.lock().await;
        for provider in providers {
            let id = provider.id.clone();
            pm.update_provider(&id, provider);
        }
    }

    pub async fn resolve_provider(&self, model_id: &str) -> Option<crate::native_engine::provider_manager::ResolvedProvider> {
        let pm = self.provider_manager.lock().await;
        pm.resolve_provider(model_id).await
    }

    pub async fn get_conversation_state(&self, conv_id: &str) -> Option<ConversationState> {
        let states = self.conversation_states.lock().await;
        states.get(conv_id).cloned()
    }

    pub async fn update_conversation_state(&self, conv_id: &str, state: ConversationState) {
        let mut states = self.conversation_states.lock().await;
        states.insert(conv_id.to_string(), state);
    }

    pub async fn load_conversation_state(&self, conv_id: &str) -> Result<Option<ConversationState>> {
        let db = self.db_manager.clone();
        let conv_id_clone = conv_id.to_string();

        let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Option<ConversationState>> {
            db.with_conn(|conn| {
                let messages = crate::db::message_repo::get_messages_by_conversation(conn, &conv_id_clone)?;
                let conv = crate::db::conversation_repo::get_conversation(conn, &conv_id_clone)?;

                if messages.is_empty() && conv.is_none() {
                    return Ok(None);
                }

                let model = conv.as_ref().and_then(|c| c.model.as_deref()).unwrap_or("");

                let messages_json: Vec<Value> = messages.into_iter().map(|msg| {
                    serde_json::json!({
                        "role": msg.role,
                        "content": msg.content,
                    })
                }).collect();

                Ok(Some(ConversationState {
                    conversation_id: conv_id_clone,
                    model: model.to_string(),
                    messages: messages_json,
                    system_prompt: None,
                    last_activity: Utc::now().to_rfc3339(),
                    turn_count: 0,
                    status: ConversationStatus::Idle,
                }))
            })?
        }).await??;

        if let Some(state) = &result {
            self.update_conversation_state(conv_id, state.clone()).await;
        }

        Ok(result)
    }

    pub async fn save_conversation_state(&self, conv_id: &str) -> Result<()> {
        let states = self.conversation_states.lock().await;
        if let Some(state) = states.get(conv_id) {
            let db = self.db_manager.clone();
            let state_clone = state.clone();

            tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
                let result = db.with_conn(|conn| {
                    let tx = conn.unchecked_transaction()?;

                    let existing = crate::db::conversation_repo::get_conversation(&tx, &state_clone.conversation_id)?;
                    if existing.is_none() {
                        crate::db::conversation_repo::insert_conversation(
                            &tx,
                            &state_clone.conversation_id,
                            None,
                            Some(&state_clone.model),
                            None,
                            None,
                            None,
                            false,
                            false,
                            false,
                            &state_clone.last_activity,
                            &state_clone.last_activity,
                            state_clone.turn_count as i64,
                        )?;
                    } else {
                        crate::db::conversation_repo::update_conversation(
                            &tx,
                            &state_clone.conversation_id,
                            None,
                            Some(&state_clone.model),
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            Some(&state_clone.last_activity),
                            Some(state_clone.turn_count as i64),
                        )?;
                    }

                    tx.commit()?;
                    Ok(())
                });
                result?
            }).await??;
        }
        Ok(())
    }

    pub async fn send_message(&self, request: ChatRequest) -> Result<mpsc::Receiver<EngineEvent>> {
        let conv_id = request.conversation_id.clone();
        let model = request.model.clone();
        let workspace_path = request.workspace_path.clone().unwrap_or_else(|| ".".to_string());

        tracing::info!(module = "EngineCore", "send_message: conv_id={}, model={}, workspace={}", conv_id, model, workspace_path);

        if let Some(ref tracker) = self.cost_tracker {
            let estimated_tokens = request.messages.iter()
                .filter_map(|m| m.get("content").and_then(|c| c.as_str()))
                .map(|c| (c.len() as u64) / 4)
                .sum::<u64>();
            let estimated_total = estimated_tokens + 2048;

            match tracker.check_budget(estimated_total) {
                BudgetCheckResult::Exceeded(msg) => {
                    tracing::warn!(module = "EngineCore", "Budget exceeded: {}", msg);
                    anyhow::bail!("Budget exceeded: {}", msg);
                }
                BudgetCheckResult::Warning(msg, usage, limit) => {
                    tracing::warn!(module = "EngineCore", "Budget warning: {}", msg);
                    let (warn_tx, warn_rx) = mpsc::channel::<EngineEvent>(10);
                    let _ = warn_tx.send(EngineEvent::BudgetWarning {
                        message: msg,
                        usage,
                        limit,
                    }).await;
                    let _ = warn_tx.send(EngineEvent::MessageStop {
                        full_text: String::new(),
                        stop_reason: Some("budget_warning".to_string()),
                    }).await;
                    return Ok(warn_rx);
                }
                BudgetCheckResult::WithinBudget => {}
            }
        }

        let reasoning_mode = request.reasoning_mode.clone().unwrap_or_else(|| {
            let first_user_msg = request.messages.iter()
                .find(|m| m.get("role").and_then(|r| r.as_str()) == Some("user"))
                .and_then(|m| m.get("content"))
                .and_then(|c| c.as_str())
                .unwrap_or("");
            analyze_complexity(first_user_msg)
        });

        tracing::info!(module = "EngineCore", "Reasoning mode: {:?}", reasoning_mode);

        let provider = {
            let pm = self.provider_manager.lock().await;
            let resolved = pm.resolve_provider(&model).await;
            tracing::info!(module = "EngineCore", "resolve_provider({}): {:?}", model, resolved.as_ref().map(|p| &p.provider.id));
            resolved.ok_or_else(|| anyhow!("No provider found for model: {}", model))?
        };

        let (event_tx, event_rx) = mpsc::channel::<EngineEvent>(500);
        let event_tx_for_turn = event_tx.clone();

        let state = ConversationState {
            conversation_id: conv_id.clone(),
            model: model.clone(),
            messages: request.messages.clone(),
            system_prompt: request.system_prompt.clone(),
            last_activity: Utc::now().to_rfc3339(),
            turn_count: 0,
            status: ConversationStatus::Active,
        };
        self.update_conversation_state(&conv_id, state).await;
        tracing::info!(module = "EngineCore", "Conversation state updated");

        const MAX_OPENAI_TOKENS: u32 = 1000000;
        const DEFAULT_MAX_TOKENS: u32 = 131072;

        let max_tokens = match &reasoning_mode {
            ReasoningMode::Quick => 50000,
            ReasoningMode::Standard => {
                let raw = request.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);
                std::cmp::min(raw, MAX_OPENAI_TOKENS)
            }
            ReasoningMode::Deep => {
                let raw = request.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);
                std::cmp::min(raw, MAX_OPENAI_TOKENS)
            }
        };

        let mut executor = ToolLoopExecutor::new(
            provider,
            request.messages,
            request.system_prompt,
            max_tokens,
            event_tx.clone(),
            workspace_path.clone(),
            reasoning_mode.clone(),
        )
        .with_conv_id(conv_id.clone())
        .with_answer_waiters(self.answer_waiters.clone())
        .with_permission_manager(self.permission_manager.clone())
        .with_prefetch_engine(Arc::new(PrefetchEngine::new(self.file_cache.clone(), workspace_path.clone())));

        if let Some(ref registry) = self.mcp_registry {
            executor = executor.with_mcp_registry(registry.clone());
        }

        let conv_id_clone = conv_id.clone();
        let active_turns_clone = self.active_turns.clone();
        let conversation_states_clone = self.conversation_states.clone();
        let _tool_call_history_clone = self.tool_call_history.clone();
        let db = self.db_manager.clone();
        let provider_manager_title = self.provider_manager.clone();
        let db_title = self.db_manager.clone();

        tracing::info!(module = "EngineCore", "Spawning executor task for conv_id={}", conv_id_clone);

        let executor_handle = tokio::spawn(async move {
            tracing::info!(module = "EngineCore", "Executor task STARTED for {}", conv_id_clone);

            // 使用 catch_unwind 捕获 panic
            let exec_result = std::panic::AssertUnwindSafe(executor.execute())
                .catch_unwind()
                .await;

            let exec_result = match exec_result {
                Ok(result) => result,
                Err(panic_info) => {
                    let panic_msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_info.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "Unknown panic".to_string()
                    };
                    tracing::error!(module = "EngineCore", "Executor PANIC for {}: {}", conv_id_clone, panic_msg);
                    let _ = event_tx.send(EngineEvent::Error(format!("Engine panic: {}", panic_msg))).await;
                    let _ = event_tx.send(EngineEvent::MessageStop {
                        full_text: String::new(),
                        stop_reason: Some("panic".to_string()),
                    }).await;
                    return;
                }
            };

            match exec_result {
                Ok((full_text, _stop_reason)) => {
                    tracing::info!(module = "EngineCore", "Executor completed for {}, full_text_len={}", conv_id_clone, full_text.len());
                    if !full_text.is_empty() {
                        let conv_id = conv_id_clone.clone();
                        let _ = tokio::task::spawn_blocking(move || {
                            db.with_conn(|conn| {
                                let msg_id = Uuid::new_v4().to_string();
                                let now = Utc::now().to_rfc3339();
                                let sort_order = crate::db::message_repo::get_messages_by_conversation(conn, &conv_id)
                                    .unwrap_or_default()
                                    .len() as i64;
                                crate::db::message_repo::insert_message(
                                    conn, &msg_id, &conv_id, "assistant", &full_text, None, &now, false, sort_order,
                                )?;
                                crate::db::conversation_repo::increment_message_count(conn, &conv_id)?;
                                Ok::<(), anyhow::Error>(())
                            })
                        }).await;
                    }
                }
                Err(e) => {
                    tracing::error!(module = "QueryEngine", "Error in turn for {}: {}", conv_id_clone, e);
                    // Note: execute() already sends EngineEvent::Error + MessageStop to event_tx,
                    // so the frontend will receive a proper error message instead of channel closed.
                    let mut states = conversation_states_clone.lock().await;
                    if let Some(state) = states.get_mut(&conv_id_clone) {
                        state.status = ConversationStatus::Error;
                        state.last_activity = Utc::now().to_rfc3339();
                    }
                }
            }

            let mut turns = active_turns_clone.lock().await;
            if let Some(turn) = turns.get_mut(&conv_id_clone) {
                turn.cancelled = true;
            }
            turns.remove(&conv_id_clone);

            let mut states = conversation_states_clone.lock().await;
            if let Some(state) = states.get_mut(&conv_id_clone) {
                state.status = ConversationStatus::Completed;
                state.last_activity = Utc::now().to_rfc3339();
                let current_turn_count = state.turn_count;
                state.turn_count += 1;

                if current_turn_count == 0 {
                    let title_conv_id = conv_id_clone.clone();
                    let title_model = state.model.clone();
                    let pm = provider_manager_title.clone();
                    let db_t = db_title.clone();
                    tokio::spawn(async move {
                        tracing::info!(module = "EngineCore", "Auto-generating title for conv_id={}", title_conv_id);
                        let title_conv_id_inner = title_conv_id.clone();
                        let result = tokio::task::spawn_blocking(move || {
                            db_t.with_conn(|conn| {
                                let conv = crate::db::conversation_repo::get_conversation(conn, &title_conv_id_inner)?;
                                if conv.as_ref().and_then(|c| c.title.as_ref()).map_or(false, |t| !t.is_empty()) {
                                    return Ok::<bool, anyhow::Error>(false);
                                }
                                let messages = crate::db::message_repo::get_messages_by_conversation(conn, &title_conv_id_inner)?;
                                let _first_user_msg = messages.iter().find(|m| m.role == "user").map(|m| m.content.clone());
                                let _first_assistant_msg = messages.iter().find(|m| m.role == "assistant").map(|m| m.content.clone());
                                Ok::<bool, anyhow::Error>(true)
                            })
                        }).await;

                        let should_generate = match result {
                            Ok(Ok(Ok(true))) => true,
                            _ => false,
                        };

                        if should_generate {
                            let resolved = {
                                let pm_guard = pm.lock().await;
                                pm_guard.resolve_provider(&title_model).await
                            };

                            if let Some(resolved) = resolved {
                                let db_msgs = db_title.clone();
                                let conv_id_for_msgs = title_conv_id.clone();
                                let msgs_result = tokio::task::spawn_blocking(move || {
                                    db_msgs.with_conn(|conn| {
                                        let messages = crate::db::message_repo::get_messages_by_conversation(conn, &conv_id_for_msgs)?;
                                        let first_user = messages.iter().find(|m| m.role == "user").map(|m| m.content.clone());
                                        let first_assistant = messages.iter().find(|m| m.role == "assistant").map(|m| m.content.clone());
                                        Ok::<_, anyhow::Error>((first_user, first_assistant))
                                    })
                                }).await;

                                if let Ok(Ok(Ok((Some(user_msg), assistant_msg)))) = msgs_result {
                                    let assistant_text: String = assistant_msg.unwrap_or_default();
                                    let truncated_assistant = if assistant_text.chars().count() > 200 {
                                        let mut end = 0;
                                        for (i, _) in assistant_text.char_indices() {
                                            if i >= 200 { break; }
                                            end = i;
                                        }
                                        assistant_text[..end].to_string()
                                    } else {
                                        assistant_text
                                    };

                                    let prompt = format!(
                                        "Based on the following conversation, generate a concise title (max 50 characters, in the same language as the user's message). Only output the title, nothing else.\n\nUser: {}\nAssistant: {}",
                                        user_msg, truncated_assistant
                                    );

                                    let user_msg_ref: &str = &user_msg;
                                    let fallback: String = if user_msg_ref.chars().count() > 30 {
                                        let mut end = 0;
                                        for (i, _) in user_msg_ref.char_indices() {
                                            if i >= 30 { break; }
                                            end = i;
                                        }
                                        user_msg_ref[..end].to_string()
                                    } else {
                                        user_msg.clone()
                                    };

                                    let title_result: Result<String, anyhow::Error> = match resolved.provider.api_format {
                                        crate::native_engine::provider_manager::ApiFormat::OpenAI => {
                                            let client = crate::native_engine::openai_client::OpenAIClient::new();
                                            let msgs = vec![
                                                crate::native_engine::openai_client::OpenAIMessage {
                                                    role: "user".to_string(),
                                                    content: crate::native_engine::openai_client::OpenAIContent::Text(prompt),
                                                    tool_calls: None,
                                                    tool_call_id: None,
                                                    reasoning_content: None,
                                                }
                                            ];
                                            match client.send_message(&resolved, msgs, None, vec![], 100).await {
                                                Ok(resp) => {
                                                    let t = resp.choices.first()
                                                        .map(|c| match &c.message.content {
                                                            crate::native_engine::openai_client::OpenAIContent::Text(t) => t.trim().to_string(),
                                                            crate::native_engine::openai_client::OpenAIContent::Multi(parts) => parts.iter()
                                                                .filter_map(|p| if let crate::native_engine::openai_client::OpenAIContentPart::Text { text } = p { Some(text.as_str()) } else { None })
                                                                .collect::<Vec<_>>()
                                                                .join(""),
                                                        })
                                                        .unwrap_or_default();
                                                    Ok(t)
                                                }
                                                Err(e) => Err(e),
                                            }
                                        }
                                        crate::native_engine::provider_manager::ApiFormat::Anthropic => {
                                            let client = crate::native_engine::anthropic_client::AnthropicClient::new();
                                            let msgs = vec![
                                                crate::native_engine::anthropic_client::AnthropicMessage {
                                                    role: "user".to_string(),
                                                    content: crate::native_engine::anthropic_client::AnthropicContent::Text(prompt),
                                                }
                                            ];
                                            match client.send_message(&resolved, msgs, None, vec![], 100).await {
                                                Ok(resp) => {
                                                    let t = resp.content.iter()
                                                        .filter_map(|block| if let crate::native_engine::anthropic_client::ContentBlock::Text { text } = block { Some(text.as_str()) } else { None })
                                                        .collect::<Vec<_>>()
                                                        .join("")
                                                        .trim()
                                                        .to_string();
                                                    Ok(t)
                                                }
                                                Err(e) => Err(e),
                                            }
                                        }
                                    };

                                    let final_title = match title_result {
                                        Ok(t) if !t.is_empty() => t,
                                        _ => fallback,
                                    };

                                    let db_save = db_title.clone();
                                    let conv_id_save = title_conv_id.clone();
                                    let title_save = final_title.clone();
                                    let _ = tokio::task::spawn_blocking(move || {
                                        db_save.with_conn(|conn| {
                                            crate::db::conversation_repo::update_title_if_empty(conn, &conv_id_save, &title_save)
                                        })
                                    }).await;
                                    tracing::info!(module = "EngineCore", "Title generated for {}: {}", title_conv_id, final_title);
                                }
                            }
                        }
                    });
                }
            }
        });

        {
            let mut turns = self.active_turns.lock().await;
            turns.insert(conv_id.clone(), ActiveTurn {
                event_tx: event_tx_for_turn,
                executor_handle: Some(executor_handle),
                cancelled: false,
            });
        }

        Ok(event_rx)
    }

    pub async fn cancel_turn(&self, conv_id: &str) {
        let mut turns = self.active_turns.lock().await;
        if let Some(turn) = turns.get_mut(conv_id) {
            turn.cancelled = true;
            if let Some(handle) = turn.executor_handle.take() {
                handle.abort();
            }
        }
        turns.remove(conv_id);

        let mut states = self.conversation_states.lock().await;
        if let Some(state) = states.get_mut(conv_id) {
            state.status = ConversationStatus::Idle;
            state.last_activity = Utc::now().to_rfc3339();
        }
    }

    pub async fn resume_with_answer(&self, conv_id: &str, answer: String) -> Result<()> {
        let mut waiters = self.answer_waiters.lock().await;
        if let Some(tx) = waiters.remove(conv_id) {
            tx.send(answer).map_err(|_| anyhow!("Failed to send answer: receiver already dropped"))?;
            Ok(())
        } else {
            anyhow::bail!("No pending AskUserQuestion for conversation {}", conv_id)
        }
    }

    pub async fn record_tool_call(
        &self,
        conv_id: &str,
        tool_name: &str,
        tool_input: &Value,
        output: &str,
        is_error: bool,
        message_id: Option<&str>,
    ) {
        let start = Instant::now();
        let elapsed = start.elapsed();

        let record = ToolCallRecord {
            tool_use_id: message_id.unwrap_or("").to_string(),
            tool_name: tool_name.to_string(),
            tool_input: tool_input.clone(),
            output: output.to_string(),
            is_error,
            timestamp: Utc::now().to_rfc3339(),
        };

        let mut history = self.tool_call_history.lock().await;
        history.entry(conv_id.to_string())
            .or_insert_with(Vec::new)
            .push(record);

        let _audit_entry = AuditEntry::new(
            if is_error { AuditAction::ToolCancelled } else { AuditAction::ToolExecuted },
            tool_name,
            conv_id,
        )
        .with_result(if is_error { "Error" } else { "Success" })
        .with_details(serde_json::json!({
            "input": tool_input,
            "output": output,
            "is_error": is_error,
            "duration_ms": elapsed.as_millis(),
            "message_id": message_id,
        }));

        tracing::info!(
            module = "Audit",
            "Tool: {} | Conv: {} | Error: {} | Duration: {}ms",
            tool_name, conv_id, is_error, elapsed.as_millis()
        );
    }

    pub async fn get_tool_call_history(&self, conv_id: &str) -> Vec<ToolCallRecord> {
        let history = self.tool_call_history.lock().await;
        history.get(conv_id).cloned().unwrap_or_default()
    }

    pub fn get_workspaces_dir(&self) -> &PathBuf {
        &self.workspaces_dir
    }

    pub async fn list_active_conversations(&self) -> Vec<String> {
        let states = self.conversation_states.lock().await;
        states.values()
            .filter(|s| s.status == ConversationStatus::Active)
            .map(|s| s.conversation_id.clone())
            .collect()
    }

    pub async fn cleanup_inactive_conversations(&self, max_idle_minutes: u64) -> usize {
        let now = Utc::now();
        let mut removed_count = 0;

        let mut states = self.conversation_states.lock().await;
        let inactive_ids: Vec<String> = states.iter()
            .filter(|(_, s)| {
                if let Ok(last_activity) = chrono::DateTime::parse_from_rfc3339(&s.last_activity) {
                    let duration = now.signed_duration_since(last_activity);
                    duration.num_minutes() > max_idle_minutes as i64
                } else {
                    true
                }
            })
            .map(|(id, _)| id.clone())
            .collect();

        for id in inactive_ids {
            states.remove(&id);
            removed_count += 1;
        }

        removed_count
    }
}

pub type NativeEngine = QueryEngine;
