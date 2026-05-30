use crate::native_engine::anthropic_client::{AnthropicClient, AnthropicContent, AnthropicMessage, ContentBlock};
use crate::native_engine::engine_core::ReasoningMode;
use crate::native_engine::openai_client::{OpenAIClient, OpenAIContent, OpenAIMessage};
use crate::native_engine::provider_manager::{ApiFormat, ResolvedProvider};
use crate::native_engine::token_counter::TokenCounter;
use crate::native_engine::context_compressor::{ContextCompressor, ContextWindow, CompressionLevel};
use crate::permissions::{PermissionManager, PermissionResult};
use crate::prefetch::PrefetchEngine;
use crate::streaming::sse_parser::{consume_sse_payloads, merge_tool_args, recover_malformed_tool_input};
use crate::tools::get_tool_definitions;
use crate::mcp::McpToolRegistry;
use crate::memory::{ContextManager, MemExClient, TieredCompressor};
use anyhow::{anyhow, Result};
use futures::StreamExt;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::{mpsc, Mutex, oneshot};
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone)]
pub enum EngineEvent {
    Text(String),
    Thinking(String),
    ToolUseStart {
        tool_use_id: String,
        tool_name: String,
        tool_input: Value,
        text_before: String,
    },
    ToolArgDelta {
        tool_use_id: String,
        delta: String,
    },
    ToolUseDone {
        tool_use_id: String,
        tool_name: String,
        tool_input: Value,
        output: String,
        is_error: bool,
    },
    MessageStart {
        model: String,
    },
    MessageDelta {
        stop_reason: Option<String>,
    },
    MessageStop {
        full_text: String,
        stop_reason: Option<String>,
    },
    Error(String),
    Usage(Value),
    AskUser {
        request_id: String,
        question: String,
        options: Vec<String>,
    },
    ToolPermission {
        request_id: String,
        tool_use_id: String,
        tool_name: String,
        input: serde_json::Value,
    },
    PipelineToolResult {
        tool_use_id: String,
        tool_name: String,
        tool_input: Value,
        output: String,
        is_error: bool,
        completed_count: usize,
        total_count: usize,
    },
    BudgetWarning {
        message: String,
        usage: u64,
        limit: u64,
    },
}

pub struct ToolLoopExecutor {
    provider: ResolvedProvider,
    messages: Vec<Value>,
    system_prompt: Option<String>,
    max_tokens: u32,
    max_tool_iterations: usize,
    event_tx: mpsc::Sender<EngineEvent>,
    anthropic_client: AnthropicClient,
    openai_client: OpenAIClient,
    workspace_cwd: String,
    mcp_registry: Option<Arc<McpToolRegistry>>,
    streaming_tool_args: HashMap<String, StreamingToolCall>,
    conv_id: Option<String>,
    answer_waiters: Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>,
    permission_manager: Option<Arc<PermissionManager>>,
    memex_client: Option<MemExClient>,
    context_manager: Option<ContextManager>,
    reasoning_mode: ReasoningMode,
    prefetch_engine: Option<Arc<PrefetchEngine>>,
}

#[derive(Debug, Clone)]
struct StreamingToolCall {
    name: String,
    accumulated_args: String,
}

struct PendingToolCall {
    id: String,
    name: String,
    input: Value,
    requires_user_interaction: bool,
}

fn compact_tool_output(tool_name: &str, output: &str) -> String {
    match tool_name {
        "Edit" | "Write" | "MultiEdit" | "smart_edit" => {
            if output.len() > 500 {
                let safe_end = output.char_indices()
                    .take(500)
                    .last()
                    .map(|(idx, ch)| idx + ch.len_utf8())
                    .unwrap_or(0);
                format!("{}... (truncated, {} bytes total)", &output[..safe_end], output.len())
            } else {
                output.to_string()
            }
        }
        "Read" => {
            if output.len() > 2000 {
                let safe_end = output.char_indices()
                    .take(2000)
                    .last()
                    .map(|(idx, ch)| idx + ch.len_utf8())
                    .unwrap_or(0);
                format!("{}... (truncated, {} bytes total)", &output[..safe_end], output.len())
            } else {
                output.to_string()
            }
        }
        _ => output.to_string(),
    }
}

/// Persist large text output to a temp file, returning the file path.
/// This avoids bloating conversation context with huge continuation outputs.
fn persist_large_output(content: &str, conv_id: &str, suffix: &str) -> String {
    let tmp_dir = std::env::temp_dir().join("claude-desktop").join("continuations");
    let _ = std::fs::create_dir_all(&tmp_dir);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let filename = format!("continuation_{}_{}_{}.txt", conv_id, timestamp, suffix);
    let path = tmp_dir.join(&filename);
    let _ = std::fs::write(&path, content);
    path.display().to_string()
}

/// Build a compact summary message for persistence-based continuation.
/// Instead of pushing full text into conversation context, push only a summary + file path.
fn build_compact_continuation_message(full_text: &str, file_path: &str, max_preview: usize) -> String {
    let preview_end = full_text.char_indices()
        .take(max_preview)
        .last()
        .map(|(idx, ch)| idx + ch.len_utf8())
        .unwrap_or(0);
    let preview = if full_text.len() > max_preview {
        format!("{}...", &full_text[..preview_end])
    } else {
        full_text.to_string()
    };
    format!(
        "[Output was truncated by token limit. Full content saved to: {}]\n\
         Preview (first {} chars):\n\
         {}\n\n\
         Continue exactly where you left off. Do not repeat anything you've already written. \
         Write the continuation directly into the file above.",
        file_path, max_preview, preview
    )
}

async fn execute_tool_standalone(
    tool_name: String,
    tool_input: Value,
    tool_use_id: String,
    event_tx: mpsc::Sender<EngineEvent>,
    mcp_registry: Option<Arc<McpToolRegistry>>,
    workspace_cwd: String,
    permission_manager: Option<Arc<PermissionManager>>,
    conv_id: Option<String>,
    completed_count: Arc<AtomicUsize>,
    total_count: usize,
) -> (String, String, Value, String, bool) {
    crate::metrics::TOOL_CALLS_TOTAL.inc();
    let _tool_timer = crate::metrics::TOOL_CALL_DURATION.start_timer();

    let permission_result = if let Some(ref pm) = permission_manager {
        let cid = conv_id.clone().unwrap_or_default();
        let workspace_path = Some(workspace_cwd.clone());
        let context = crate::permissions::PermissionContext {
            tool_name: tool_name.clone(),
            tool_input: tool_input.clone(),
            conversation_id: cid,
            user_id: None,
            workspace_path,
        };
        pm.check_permission(&context)
    } else {
        PermissionResult::Granted
    };

    match permission_result {
        PermissionResult::Denied(reason) => {
            let output = format!("Permission denied: {}", reason);
            let count = completed_count.fetch_add(1, Ordering::SeqCst) + 1;
            let _ = event_tx.send(EngineEvent::ToolUseDone {
                tool_use_id: tool_use_id.clone(),
                tool_name: tool_name.clone(),
                tool_input: tool_input.clone(),
                output: output.clone(),
                is_error: true,
            }).await;
            let _ = event_tx.send(EngineEvent::PipelineToolResult {
                tool_use_id: tool_use_id.clone(),
                tool_name: tool_name.clone(),
                tool_input: tool_input.clone(),
                output: output.clone(),
                is_error: true,
                completed_count: count,
                total_count,
            }).await;
            return (tool_use_id, tool_name, tool_input, output, true);
        }
        PermissionResult::RequiresConfirmation(_) => {
            let output = "Tool requires user confirmation - must be executed serially".to_string();
            let count = completed_count.fetch_add(1, Ordering::SeqCst) + 1;
            let _ = event_tx.send(EngineEvent::ToolUseDone {
                tool_use_id: tool_use_id.clone(),
                tool_name: tool_name.clone(),
                tool_input: tool_input.clone(),
                output: output.clone(),
                is_error: true,
            }).await;
            let _ = event_tx.send(EngineEvent::PipelineToolResult {
                tool_use_id: tool_use_id.clone(),
                tool_name: tool_name.clone(),
                tool_input: tool_input.clone(),
                output: output.clone(),
                is_error: true,
                completed_count: count,
                total_count,
            }).await;
            return (tool_use_id, tool_name, tool_input, output, true);
        }
        PermissionResult::Granted => {}
    }

    let (output_str, is_error) = if let Some(ref registry) = mcp_registry {
        if registry.is_mcp_tool(&tool_name).await {
            let result = registry.execute_tool(&tool_name, tool_input.clone()).await;
            match result {
                Ok(val) => (serde_json::to_string_pretty(&val).unwrap_or_default(), false),
                Err(e) => (format!("Error: {}", e), true),
            }
        } else {
            let result = crate::tools::execute_tool_async(&tool_name, tool_input.clone(), &workspace_cwd).await;
            match &result {
                Ok(val) => (serde_json::to_string_pretty(val).unwrap_or_default(), false),
                Err(e) => (format!("Error: {}", e), true),
            }
        }
    } else {
        let result = crate::tools::execute_tool_async(&tool_name, tool_input.clone(), &workspace_cwd).await;
        match &result {
            Ok(val) => (serde_json::to_string_pretty(val).unwrap_or_default(), false),
            Err(e) => (format!("Error: {}", e), true),
        }
    };

    let count = completed_count.fetch_add(1, Ordering::SeqCst) + 1;
    let _ = event_tx.send(EngineEvent::ToolUseDone {
        tool_use_id: tool_use_id.clone(),
        tool_name: tool_name.clone(),
        tool_input: tool_input.clone(),
        output: output_str.clone(),
        is_error,
    }).await;
    let _ = event_tx.send(EngineEvent::PipelineToolResult {
        tool_use_id: tool_use_id.clone(),
        tool_name: tool_name.clone(),
        tool_input: tool_input.clone(),
        output: output_str.clone(),
        is_error,
        completed_count: count,
        total_count,
    }).await;

    (tool_use_id, tool_name, tool_input, output_str, is_error)
}

impl ToolLoopExecutor {
    pub fn new(
        provider: ResolvedProvider,
        messages: Vec<Value>,
        system_prompt: Option<String>,
        max_tokens: u32,
        event_tx: mpsc::Sender<EngineEvent>,
        workspace_cwd: String,
        reasoning_mode: ReasoningMode,
    ) -> Self {
        Self {
            provider,
            messages,
            system_prompt,
            max_tokens,
            max_tool_iterations: crate::config::OrchestrationConfig::max_tool_iterations(),
            event_tx,
            anthropic_client: AnthropicClient::new(),
            openai_client: OpenAIClient::new(),
            workspace_cwd,
            mcp_registry: None,
            streaming_tool_args: HashMap::new(),
            conv_id: None,
            answer_waiters: Arc::new(Mutex::new(HashMap::new())),
            permission_manager: None,
            memex_client: None,
            context_manager: None,
            reasoning_mode,
            prefetch_engine: None,
        }
    }

    pub fn with_conv_id(mut self, conv_id: String) -> Self {
        self.conv_id = Some(conv_id);
        self
    }

    pub fn with_answer_waiters(mut self, waiters: Arc<Mutex<HashMap<String, oneshot::Sender<String>>>>) -> Self {
        self.answer_waiters = waiters;
        self
    }

    pub fn get_answer_waiters(&self) -> Arc<Mutex<HashMap<String, oneshot::Sender<String>>>> {
        self.answer_waiters.clone()
    }

    pub fn with_mcp_registry(mut self, registry: Arc<McpToolRegistry>) -> Self {
        self.mcp_registry = Some(registry);
        self
    }

    pub fn with_permission_manager(mut self, manager: Arc<PermissionManager>) -> Self {
        self.permission_manager = Some(manager);
        self
    }

    pub fn with_memex(mut self, client: MemExClient) -> Self {
        let client_arc = Arc::new(client.clone());
        self.context_manager = Some(ContextManager::new(client_arc));
        self.memex_client = Some(client);
        self
    }

    pub fn with_prefetch_engine(mut self, engine: Arc<PrefetchEngine>) -> Self {
        self.prefetch_engine = Some(engine);
        self
    }

    async fn check_permission(&self, tool_name: &str, tool_input: &Value) -> PermissionResult {
        if let Some(ref pm) = self.permission_manager {
            let conv_id = self.conv_id.clone().unwrap_or_default();
            let workspace_path = Some(self.workspace_cwd.clone());

            let context = crate::permissions::PermissionContext {
                tool_name: tool_name.to_string(),
                tool_input: tool_input.clone(),
                conversation_id: conv_id,
                user_id: None,
                workspace_path,
            };
            pm.check_permission(&context)
        } else {
            PermissionResult::Granted
        }
    }

    fn check_permission_requires_confirmation(&self, tool_name: &str, tool_input: &Value) -> bool {
        if let Some(ref pm) = self.permission_manager {
            let conv_id = self.conv_id.clone().unwrap_or_default();
            let workspace_path = Some(self.workspace_cwd.clone());
            let context = crate::permissions::PermissionContext {
                tool_name: tool_name.to_string(),
                tool_input: tool_input.clone(),
                conversation_id: conv_id,
                user_id: None,
                workspace_path,
            };
            matches!(pm.check_permission(&context), PermissionResult::RequiresConfirmation(_))
        } else {
            false
        }
    }

    pub async fn execute(&mut self) -> Result<(String, Option<String>)> {
        let conv_id = self.conv_id.clone().unwrap_or_default();
        tracing::info!(module = "ToolLoop", "[E1] execute ENTRY: conv_id={}, messages_count={}, provider={}", conv_id, self.messages.len(), self.provider.provider.id);

        // Token 计数与自动压缩 - First pass
        tracing::info!(module = "ToolLoop", "[E2] Checking token threshold...");
        let token_threshold = crate::config::OrchestrationConfig::token_threshold();
        if TokenCounter::exceeds_threshold(&self.messages, token_threshold) {
            tracing::info!(module = "ToolLoop", "[E2.1] Tokens exceed threshold, compressing with TieredCompressor...");
            let compressor = TieredCompressor::new();
            let compressed = compressor.compress(&self.messages);
            let original_len = self.messages.len();
            self.messages = compressed;
            tracing::info!(module = "ToolLoop", "Context compressed: {} -> {} messages", original_len, self.messages.len());
        }
        tracing::info!(module = "ToolLoop", "[E2] Token check COMPLETED");

        // 提取最后一条用户消息用于记忆检索
        tracing::info!(module = "ToolLoop", "[E3] Extracting last user message...");
        let last_user_msg = self.messages.iter()
            .rev()
            .find(|m| m.get("role").and_then(|r| r.as_str()) == Some("user"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();
        tracing::info!(module = "ToolLoop", "[E3] Last user msg len={}", last_user_msg.len());

        if let Some(ref prefetch) = self.prefetch_engine {
            let msg = last_user_msg.clone();
            let prefetch_clone = prefetch.clone();
            tokio::spawn(async move {
                prefetch_clone.prefetch_for_message(&msg).await;
            });
        }

        if let Some(ctx_mgr) = &self.context_manager {
            tracing::info!(module = "ToolLoop", "[E4] Injecting memory context...");
            if let Some(memory_context) = ctx_mgr.before_api_call(&conv_id, &last_user_msg).await {
                self.messages.insert(0, json!({
                    "role": "user",
                    "content": memory_context
                }));
            }
        }
        tracing::info!(module = "ToolLoop", "[E4] Memory injection COMPLETED");

        // 模型特定的检查和二次压缩
        tracing::info!(module = "ToolLoop", "[E4.5] Model-specific token check...");
        let model_id = &self.provider.model.id;
        let safety_margin = 0.85; // Keep 15% buffer for response
        let safe_limit = TokenCounter::get_safe_limit(model_id, safety_margin);
        let estimated_tokens = TokenCounter::estimate(&self.messages);

        tracing::info!(module = "ToolLoop", "Model: {}, Estimated tokens: {}, Safe limit: {}", model_id, estimated_tokens, safe_limit);

        if estimated_tokens > safe_limit {
            tracing::warn!(module = "ToolLoop", "Context still exceeds model limit! Applying progressive compression...");
            let compressor = TieredCompressor::new();
            let original_estimate = TokenCounter::estimate(&self.messages);

            self.messages = compressor.compress_progressive(
                &self.messages,
                safe_limit,
                |msgs| TokenCounter::estimate(msgs),
            );

            let final_estimate = TokenCounter::estimate(&self.messages);
            tracing::info!(module = "ToolLoop", "Progressive compression complete: {} -> {} tokens", original_estimate, final_estimate);
        }
        tracing::info!(module = "ToolLoop", "[E4.5] Model check COMPLETED");

        tracing::info!(module = "ToolLoop", "[E5] Sending MessageStart event...");
        let _ = self.event_tx.send(EngineEvent::MessageStart {
            model: self.provider.model.id.clone(),
        }).await;
        tracing::info!(module = "ToolLoop", "[E5] MessageStart sent");

        tracing::info!(module = "ToolLoop", "[E6] API format: {:?}", self.provider.provider.api_format);

        let loop_result = match self.provider.provider.api_format {
            ApiFormat::Anthropic => {
                tracing::info!(module = "ToolLoop", "[E6.1] Starting Anthropic loop...");
                self.execute_anthropic_loop().await
            }
            ApiFormat::OpenAI => {
                tracing::info!(module = "ToolLoop", "[E6.2] Starting OpenAI loop...");
                self.execute_openai_loop().await
            }
        };

        match &loop_result {
            Ok((full_text, stop_reason)) => {
                tracing::info!(module = "ToolLoop", "[E6] API loop COMPLETED, full_text_len={}", full_text.len());
                let _ = self.event_tx.send(EngineEvent::MessageStop {
                    full_text: full_text.clone(),
                    stop_reason: stop_reason.clone(),
                }).await;
                tracing::info!(module = "ToolLoop", "[E7] MessageStop sent (success)");
            }
            Err(e) => {
                let err_msg = format!("Engine execution failed: {}", e);
                tracing::error!(module = "ToolLoop", "[E6] API loop FAILED: {}", err_msg);
                let _ = self.event_tx.send(EngineEvent::Error(err_msg)).await;
                let _ = self.event_tx.send(EngineEvent::MessageStop {
                    full_text: String::new(),
                    stop_reason: Some("error".to_string()),
                }).await;
                tracing::info!(module = "ToolLoop", "[E7] Error + MessageStop sent (failure)");
            }
        }

        // 回复后：存储记忆
        if let Some(ctx_mgr) = &self.context_manager {
            if !last_user_msg.is_empty() {
                if let Ok((ref full_text, _)) = loop_result {
                    ctx_mgr.after_response(&conv_id, &last_user_msg, full_text).await;
                }
            }
        }

        loop_result
    }

    async fn execute_tool_call(
        &mut self,
        tool_name: &str,
        tool_input: &Value,
        _tool_use_id: &str,
    ) -> (Value, String, bool) {
        if tool_name == "AskUserQuestion" {
            return self.execute_ask_user_question(tool_input).await;
        }

        // Route memory_search and memory_ingest directly to MemEx backend
        if tool_name == "memory_search" {
            return self.execute_memory_search(tool_input).await;
        }
        if tool_name == "memory_ingest" {
            return self.execute_memory_ingest(tool_input).await;
        }

        let permission_result = self.check_permission(tool_name, tool_input).await;
        match permission_result {
            PermissionResult::Denied(reason) => {
                return (tool_input.clone(), format!("Permission denied: {}", reason), true);
            }
            PermissionResult::RequiresConfirmation(message) => {
                return self.execute_ask_user_confirmation(tool_name, tool_input, _tool_use_id, &message).await;
            }
            PermissionResult::Granted => {}
        }

        let output_str;
        let is_error;

        if let Some(ref registry) = self.mcp_registry {
            if registry.is_mcp_tool(tool_name).await {
                let result = registry.execute_tool(tool_name, tool_input.clone()).await;
                match result {
                    Ok(val) => {
                        output_str = serde_json::to_string_pretty(&val).unwrap_or_default();
                        is_error = false;
                    }
                    Err(e) => {
                        output_str = format!("Error: {}", e);
                        is_error = true;
                    }
                };
            } else {
                let cwd = self.get_workspace_cwd().to_string();
                let result = crate::tools::execute_tool_async(tool_name, tool_input.clone(), &cwd).await;
                output_str = match &result {
                    Ok(val) => serde_json::to_string_pretty(val).unwrap_or_default(),
                    Err(e) => format!("Error: {}", e),
                };
                is_error = result.is_err();
            }
        } else {
            let cwd = self.get_workspace_cwd().to_string();
            let result = crate::tools::execute_tool_async(tool_name, tool_input.clone(), &cwd).await;
            output_str = match &result {
                Ok(val) => serde_json::to_string_pretty(val).unwrap_or_default(),
                Err(e) => format!("Error: {}", e),
            };
            is_error = result.is_err();
        }

        (tool_input.clone(), output_str, is_error)
    }

    async fn execute_ask_user_question(&mut self, tool_input: &Value) -> (Value, String, bool) {
        let request_id = uuid::Uuid::new_v4().to_string();
        let question = tool_input["question"].as_str().unwrap_or("").to_string();
        let options: Vec<String> = tool_input["options"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|opt| opt["label"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let _ = self.event_tx.send(EngineEvent::AskUser {
            request_id: request_id.clone(),
            question: question.clone(),
            options: options.clone(),
        }).await;

        let (tx, rx) = oneshot::channel::<String>();
        {
            let mut waiters = self.answer_waiters.lock().await;
            waiters.insert(request_id.clone(), tx);
        }

        match timeout(Duration::from_secs(300), rx).await {
            Ok(Ok(answer)) => {
                let result = serde_json::json!({
                    "type": "ask_user_question",
                    "question": question,
                    "answer": answer,
                    "content": answer,
                    "requires_user_input": false
                });
                (tool_input.clone(), serde_json::to_string_pretty(&result).unwrap_or_default(), false)
            }
            Ok(Err(_)) => {
                let mut waiters = self.answer_waiters.lock().await;
                waiters.remove(&request_id);
                let result = serde_json::json!({
                    "type": "ask_user_question",
                    "question": question,
                    "content": "User did not respond",
                    "is_error": true
                });
                (tool_input.clone(), serde_json::to_string_pretty(&result).unwrap_or_default(), true)
            }
            Err(_) => {
                let mut waiters = self.answer_waiters.lock().await;
                waiters.remove(&request_id);
                let result = serde_json::json!({
                    "type": "ask_user_question",
                    "question": question,
                    "content": "User response timed out (300s)",
                    "is_error": true
                });
                (tool_input.clone(), serde_json::to_string_pretty(&result).unwrap_or_default(), true)
            }
        }
    }

    async fn execute_memory_search(&self, tool_input: &Value) -> (Value, String, bool) {
        let query = tool_input["query"].as_str().unwrap_or("");
        let top_k = tool_input["top_k"].as_u64().map(|v| v as usize);

        if let Some(ref client) = self.memex_client {
            match client.search(query, top_k).await {
                Ok(results) => {
                    let output = serde_json::json!({
                        "status": "search_completed",
                        "query": query,
                        "results": results,
                        "count": results.len()
                    });
                    (tool_input.clone(), serde_json::to_string_pretty(&output).unwrap_or_default(), false)
                }
                Err(e) => {
                    let output = serde_json::json!({
                        "status": "search_failed",
                        "query": query,
                        "error": format!("{}", e)
                    });
                    (tool_input.clone(), serde_json::to_string_pretty(&output).unwrap_or_default(), true)
                }
            }
        } else {
            let output = serde_json::json!({
                "status": "search_unavailable",
                "query": query,
                "message": "MemEx backend is not configured. Memory search is unavailable."
            });
            (tool_input.clone(), serde_json::to_string_pretty(&output).unwrap_or_default(), false)
        }
    }

    async fn execute_memory_ingest(&self, tool_input: &Value) -> (Value, String, bool) {
        let content = tool_input["content"].as_str().unwrap_or("");
        let importance = tool_input["importance"].as_f64();

        if let Some(ref client) = self.memex_client {
            match client.ingest(content, importance, None).await {
                Ok(()) => {
                    let output = serde_json::json!({
                        "status": "ingested",
                        "content_length": content.len(),
                        "importance": importance
                    });
                    (tool_input.clone(), serde_json::to_string_pretty(&output).unwrap_or_default(), false)
                }
                Err(e) => {
                    let output = serde_json::json!({
                        "status": "ingest_failed",
                        "error": format!("{}", e)
                    });
                    (tool_input.clone(), serde_json::to_string_pretty(&output).unwrap_or_default(), true)
                }
            }
        } else {
            let output = serde_json::json!({
                "status": "ingest_unavailable",
                "message": "MemEx backend is not configured. Memory ingest is unavailable."
            });
            (tool_input.clone(), serde_json::to_string_pretty(&output).unwrap_or_default(), false)
        }
    }

    async fn execute_ask_user_confirmation(&mut self, tool_name: &str, tool_input: &Value, tool_use_id: &str, _message: &str) -> (Value, String, bool) {
        let request_id = uuid::Uuid::new_v4().to_string();
        let _ = self.event_tx.send(EngineEvent::ToolPermission {
            request_id: request_id.clone(),
            tool_use_id: tool_use_id.to_string(),
            tool_name: tool_name.to_string(),
            input: tool_input.clone(),
        }).await;

        let (tx, rx) = oneshot::channel::<String>();
        {
            let mut waiters = self.answer_waiters.lock().await;
            waiters.insert(request_id.clone(), tx);
        }

        match timeout(Duration::from_secs(300), rx).await {
            Ok(Ok(answer)) => {
                let answer_lower = answer.trim().to_lowercase();
                let is_allowed = matches!(answer_lower.as_str(),
                    "yes" | "allow" | "ok" | "continue" | "proceed" | "approved" | "always_allow" |
                    "是" | "允许" | "确认" | "继续" | "同意" | "好的" | "可以"
                );

                let is_denied = matches!(answer_lower.as_str(),
                    "no" | "cancel" | "deny" | "reject" | "stop" | "abort" |
                    "否" | "取消" | "拒绝" | "停止" | "不要"
                );

                if is_allowed {
                    let conv_id = self.conv_id.clone().unwrap_or_default();
                    if answer_lower == "always_allow" {
                        if let Some(ref pm) = self.permission_manager {
                            pm.set_tool_permission(tool_name, crate::permissions::ToolPermission {
                                tool_name: tool_name.to_string(),
                                allowed: true,
                                requires_confirmation: false,
                                scope: crate::permissions::PermissionScope::Global,
                                level: crate::permissions::PermissionLevel::Execute,
                            });
                        }
                    }
                    if let Some(ref pm) = self.permission_manager {
                        pm.confirm_permission(&conv_id, tool_name);
                    }

                    let result = self.execute_tool_call_unchecked(tool_name, tool_input, "").await;
                    result
                } else if is_denied {
                    (tool_input.clone(), "User denied the operation".to_string(), true)
                } else {
                    (tool_input.clone(), "User cancelled the operation".to_string(), true)
                }
            }
            Ok(Err(_)) => {
                let mut waiters = self.answer_waiters.lock().await;
                waiters.remove(&request_id);
                (tool_input.clone(), "User did not respond, operation cancelled".to_string(), true)
            }
            Err(_) => {
                let mut waiters = self.answer_waiters.lock().await;
                waiters.remove(&request_id);
                (tool_input.clone(), "User response timed out (300s), operation cancelled".to_string(), true)
            }
        }
    }

    async fn execute_tool_call_unchecked(
        &mut self,
        tool_name: &str,
        tool_input: &Value,
        _tool_use_id: &str,
    ) -> (Value, String, bool) {
        let output_str;
        let is_error;

        if let Some(ref registry) = self.mcp_registry {
            if registry.is_mcp_tool(tool_name).await {
                let result = registry.execute_tool(tool_name, tool_input.clone()).await;
                output_str = match &result {
                    Ok(val) => serde_json::to_string_pretty(val).unwrap_or_default(),
                    Err(e) => format!("Error: {}", e),
                };
                is_error = result.is_err();
            } else {
                let cwd = self.get_workspace_cwd().to_string();
                let result = crate::tools::execute_tool_async(tool_name, tool_input.clone(), &cwd).await;
                output_str = match &result {
                    Ok(val) => serde_json::to_string_pretty(val).unwrap_or_default(),
                    Err(e) => format!("Error: {}", e),
                };
                is_error = result.is_err();
            }
        } else {
            let cwd = self.get_workspace_cwd().to_string();
            let result = crate::tools::execute_tool_async(tool_name, tool_input.clone(), &cwd).await;
            output_str = match &result {
                Ok(val) => serde_json::to_string_pretty(val).unwrap_or_default(),
                Err(e) => format!("Error: {}", e),
            };
            is_error = result.is_err();
        }

        (tool_input.clone(), output_str, is_error)
    }

    async fn execute_anthropic_loop(&mut self) -> Result<(String, Option<String>)> {
        tracing::info!(module = "AnthropicLoop", "ENTRY: building messages, msg_count={}", self.messages.len());
        let mut conversation_messages: Vec<AnthropicMessage> = self.build_anthropic_messages();
        tracing::info!(module = "AnthropicLoop", "Messages built, count={}", conversation_messages.len());

        let tools = get_tool_definitions();
        tracing::info!(module = "AnthropicLoop", "Tool definitions loaded, count={}, mode={:?}", tools.len(), self.reasoning_mode);

        if matches!(self.reasoning_mode, ReasoningMode::Deep) {
            if let Some(ref sp) = self.system_prompt {
                self.system_prompt = Some(format!("{}\n\nYou are in deep reasoning mode. For any destructive operations, you MUST ask for user confirmation before proceeding.", sp));
            } else {
                self.system_prompt = Some("You are in deep reasoning mode. For any destructive operations, you MUST ask for user confirmation before proceeding.".to_string());
            }
        }

        let mut full_text = String::new();
        let mut stop_reason = None;
        let base_url = self.provider.provider.base_url.clone();
        let model_id = self.provider.model.id.clone();
        let api_key_len = self.provider.provider.api_key.len();

        let enable_thinking = matches!(self.reasoning_mode, ReasoningMode::Deep);

        for iteration in 0..self.max_tool_iterations {
            tracing::info!(module = "AnthropicLoop", "Iteration {}/{} starting, base_url={}, model={}", iteration+1, self.max_tool_iterations, base_url, model_id);
            tracing::info!(module = "AnthropicLoop", "API key length={}", api_key_len);

            // Context compression check before each API call
            if iteration > 0 {
                let model_context_limit = TokenCounter::get_safe_limit(&model_id, 0.90);
                let conv_values: Vec<Value> = conversation_messages.iter().filter_map(|m| {
                    match &m.content {
                        AnthropicContent::Text(t) => Some(json!({"role": m.role, "content": t})),
                        AnthropicContent::Blocks(blocks) => {
                            let content_blocks: Vec<Value> = blocks.iter().filter_map(|b| {
                                match b {
                                    ContentBlock::Text { text } => Some(json!({"type": "text", "text": text})),
                                    ContentBlock::ToolResult { tool_use_id, content, is_error } => {
                                        Some(json!({"type": "tool_result", "tool_use_id": tool_use_id, "content": content, "is_error": is_error}))
                                    }
                                    ContentBlock::ToolUse { id, name, input } => {
                                        Some(json!({"type": "tool_use", "id": id, "name": name, "input": input}))
                                    }
                                    _ => None,
                                }
                            }).collect();
                            Some(json!({"role": m.role, "content": content_blocks}))
                        }
                    }
                }).collect();
                let estimated = TokenCounter::estimate(&conv_values);
                if estimated > (model_context_limit * 70 / 100) {
                    tracing::info!(module = "AnthropicLoop", "Context compression needed: {} tokens (limit: {})", estimated, model_context_limit);
                    let compressor = ContextCompressor::new(None);
                    let context_window = ContextWindow {
                        model_id: model_id.clone(),
                        total_tokens: estimated as u64,
                        reserved_for_completion: (self.max_tokens as u64) * 2,
                        context_window: model_context_limit as u64,
                    };
                    let level = compressor.determine_compression_level(&context_window);
                    if level != CompressionLevel::None {
                        let conv_values: Vec<Value> = conversation_messages.iter().filter_map(|m| {
                            match &m.content {
                                AnthropicContent::Text(t) => Some(json!({"role": m.role, "content": t})),
                                AnthropicContent::Blocks(blocks) => {
                                    let content_blocks: Vec<Value> = blocks.iter().filter_map(|b| {
                                        match b {
                                            ContentBlock::Text { text } => Some(json!({"type": "text", "text": text})),
                                            ContentBlock::ToolResult { tool_use_id, content, is_error } => {
                                                Some(json!({"type": "tool_result", "tool_use_id": tool_use_id, "content": content, "is_error": is_error}))
                                            }
                                            ContentBlock::ToolUse { id, name, input } => {
                                                Some(json!({"type": "tool_use", "id": id, "name": name, "input": input}))
                                            }
                                            _ => None,
                                        }
                                    }).collect();
                                    Some(json!({"role": m.role, "content": content_blocks}))
                                }
                            }
                        }).collect();
                        let (compressed, metadata) = compressor.compress(
                            conv_values,
                            self.system_prompt.clone(),
                            level,
                            Some(&self.provider),
                        ).await;
                        if metadata.was_compressed {
                            conversation_messages = compressed.iter().filter_map(|v| {
                                let role = v.get("role")?.as_str()?.to_string();
                                let content = v.get("content")?;
                                let anthropic_content = if content.is_string() {
                                    AnthropicContent::Text(content.as_str()?.to_string())
                                } else if content.is_array() {
                                    let blocks: Vec<ContentBlock> = content.as_array()?.iter().filter_map(|b| {
                                        let bt = b.get("type")?.as_str()?;
                                        match bt {
                                            "text" => Some(ContentBlock::Text { text: b.get("text")?.as_str()?.to_string() }),
                                            "tool_result" => Some(ContentBlock::ToolResult {
                                                tool_use_id: b.get("tool_use_id")?.as_str()?.to_string(),
                                                content: b.get("content")?.as_str()?.to_string(),
                                                is_error: b.get("is_error").and_then(|v| v.as_bool()),
                                            }),
                                            "tool_use" => Some(ContentBlock::ToolUse {
                                                id: b.get("id")?.as_str()?.to_string(),
                                                name: b.get("name")?.as_str()?.to_string(),
                                                input: b.get("input")?.clone(),
                                            }),
                                            _ => None,
                                        }
                                    }).collect();
                                    AnthropicContent::Blocks(blocks)
                                } else {
                                    return None;
                                };
                                Some(AnthropicMessage { role, content: anthropic_content })
                            }).collect();
                            tracing::info!(module = "AnthropicLoop", "Context compressed: {} -> {} messages, saved {} tokens",
                                metadata.original_count, metadata.compressed_count, metadata.token_reduction);
                        }
                    }
                }
            }

            self.streaming_tool_args.clear();
            let stream = self.anthropic_client
                .send_message_stream_with_thinking(
                    &self.provider,
                    conversation_messages.clone(),
                    self.system_prompt.as_deref(),
                    tools.clone(),
                    self.max_tokens,
                    enable_thinking,
                )
                .await;

            let mut stream = match stream {
                Ok(s) => {
                    tracing::info!(module = "AnthropicLoop", "Stream created successfully");
                    s
                }
                Err(e) => {
                    let err_msg = format!("API request failed: {}", e);
                    tracing::error!(module = "AnthropicLoop", "Stream creation FAILED: {}", err_msg);
                    let _ = self.event_tx.send(EngineEvent::Error(err_msg.clone())).await;
                    return Err(anyhow!(err_msg));
                }
            };

            let mut sse_buffer = String::new();
            let mut has_tool_use = false;
            let mut assistant_blocks: Vec<ContentBlock> = Vec::new();
            let mut current_text = String::new();
            let mut current_thinking = String::new();
            let mut current_tool_use_id: Option<String> = None;
            let mut current_tool_name: Option<String> = None;
            let mut pending_tool_calls: Vec<PendingToolCall> = Vec::new();

            while let Some(chunk_result) = stream.next().await {
                if self.event_tx.is_closed() {
                    drop(stream);
                    break;
                }
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => {
                        let err_msg = format!("Stream error: {}", e);
                        tracing::error!(module = "AnthropicLoop", "Stream chunk error: {}", err_msg);
                        let _ = self.event_tx.send(EngineEvent::Error(err_msg)).await;
                        let _ = self.event_tx.send(EngineEvent::MessageStop {
                            full_text: full_text.clone(),
                            stop_reason: Some("stream_error".to_string()),
                        }).await;
                        drop(stream);
                        return Ok((full_text, Some("stream_error".to_string())));
                    }
                };

                sse_buffer.push_str(&chunk);
                let consumed = consume_sse_payloads(&sse_buffer);
                sse_buffer = consumed.remainder;

                for payload in &consumed.payloads {
                    let event: Value = match serde_json::from_str(payload) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    let event_type = event.get("type").and_then(|t| t.as_str()).unwrap_or("");

                    match event_type {
                        "message_start" => {
                            if let Some(message) = event.get("message") {
                                if let Some(model) = message.get("model").and_then(|m| m.as_str()) {
                                    let _ = self.event_tx.send(EngineEvent::MessageStart {
                                        model: model.to_string(),
                                    }).await;
                                }
                                if let Some(usage) = message.get("usage") {
                                    if let Some(input_tokens) = usage.get("input_tokens").and_then(|v| v.as_f64()) {
                                        crate::metrics::TOKENS_CONSUMED.inc_by(input_tokens);
                                    }
                                    if let Some(output_tokens) = usage.get("output_tokens").and_then(|v| v.as_f64()) {
                                        crate::metrics::TOKENS_CONSUMED.inc_by(output_tokens);
                                    }
                                    let _ = self.event_tx.send(EngineEvent::Usage(usage.clone())).await;
                                }
                            }
                        }
                        "content_block_start" => {
                            let block = event.get("content_block");
                            let block_type = block.and_then(|b| b.get("type")).and_then(|t| t.as_str()).unwrap_or("");

                            match block_type {
                                "text" => {
                                    current_text.clear();
                                }
                                "thinking" => {
                                    current_thinking.clear();
                                }
                                "tool_use" => {
                                    has_tool_use = true;
                                    let id = block.and_then(|b| b.get("id")).and_then(|i| i.as_str()).unwrap_or("").to_string();
                                    let name = block.and_then(|b| b.get("name")).and_then(|n| n.as_str()).unwrap_or("").to_string();
                                    current_tool_use_id = Some(id.clone());
                                    current_tool_name = Some(name.clone());

                                    let _ = self.event_tx.send(EngineEvent::ToolUseStart {
                                        tool_use_id: id,
                                        tool_name: name,
                                        tool_input: json!({}),
                                        text_before: full_text.clone(),
                                    }).await;
                                }
                                _ => {}
                            }
                        }
                        "content_block_delta" => {
                            let delta = event.get("delta");
                            let delta_type = delta.and_then(|d| d.get("type")).and_then(|t| t.as_str()).unwrap_or("");

                            match delta_type {
                                "text_delta" => {
                                    let text = delta.and_then(|d| d.get("text")).and_then(|t| t.as_str()).unwrap_or("");
                                    if !text.is_empty() {
                                        current_text.push_str(text);
                                        full_text.push_str(text);
                                        let _ = self.event_tx.send(EngineEvent::Text(text.to_string())).await;
                                    }
                                }
                                "thinking_delta" => {
                                    let thinking = delta.and_then(|d| d.get("thinking")).and_then(|t| t.as_str()).unwrap_or("");
                                    if !thinking.is_empty() {
                                        current_thinking.push_str(thinking);
                                        let _ = self.event_tx.send(EngineEvent::Thinking(thinking.to_string())).await;
                                    }
                                }
                                "input_json_delta" => {
                                    let partial = delta.and_then(|d| d.get("partial_json")).and_then(|p| p.as_str()).unwrap_or("");
                                    if !partial.is_empty() {
                                        if let (Some(ref id), Some(ref name)) = (&current_tool_use_id, &current_tool_name) {
                                            self.handle_streaming_tool_arg_delta(id, name, partial);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        "content_block_stop" => {
                            let _index = event.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;

                            if !current_text.is_empty() {
                                assistant_blocks.push(ContentBlock::Text { text: current_text.clone() });
                                current_text.clear();
                            } else if !current_thinking.is_empty() {
                                assistant_blocks.push(ContentBlock::Thinking {
                                    thinking: current_thinking.clone(),
                                    signature: None,
                                });
                                current_thinking.clear();
                            } else if current_tool_use_id.is_some() {
                                let id = current_tool_use_id.clone().unwrap_or_default();
                                let name = current_tool_name.clone().unwrap_or_default();
                                let input = self.finalize_streaming_tool_args(&id);

                                let requires_user_interaction = name == "AskUserQuestion"
                                    || self.check_permission_requires_confirmation(&name, &input);

                                assistant_blocks.push(ContentBlock::ToolUse {
                                    id: id.clone(),
                                    name: name.clone(),
                                    input: input.clone(),
                                });

                                pending_tool_calls.push(PendingToolCall {
                                    id,
                                    name,
                                    input,
                                    requires_user_interaction,
                                });

                                current_tool_use_id = None;
                                current_tool_name = None;
                            }
                        }
                        "message_delta" => {
                            let delta = event.get("delta");
                            let sr = delta.and_then(|d| d.get("stop_reason")).and_then(|s| s.as_str()).map(String::from);
                            if sr.is_some() {
                                stop_reason = sr.clone();
                                let _ = self.event_tx.send(EngineEvent::MessageDelta {
                                    stop_reason: sr,
                                }).await;
                            }
                            if let Some(usage) = event.get("usage") {
                                if let Some(output_tokens) = usage.get("output_tokens").and_then(|v| v.as_f64()) {
                                    crate::metrics::TOKENS_CONSUMED.inc_by(output_tokens);
                                }
                                let _ = self.event_tx.send(EngineEvent::Usage(usage.clone())).await;
                            }
                        }
                        "message_stop" => {}
                        "ping" => {}
                        _ => {}
                    }
                }
            }

            if !sse_buffer.is_empty() {
                let consumed = consume_sse_payloads(&sse_buffer);
                for payload in &consumed.payloads {
                    let event: Value = match serde_json::from_str(payload) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    let event_type = event.get("type").and_then(|t| t.as_str()).unwrap_or("");
                    if event_type == "message_delta" {
                        let delta = event.get("delta");
                        let sr = delta.and_then(|d| d.get("stop_reason")).and_then(|s| s.as_str()).map(String::from);
                        if sr.is_some() {
                            stop_reason = sr.clone();
                            let _ = self.event_tx.send(EngineEvent::MessageDelta {
                                stop_reason: sr,
                            }).await;
                        }
                    }
                }
            }

            if has_tool_use && !pending_tool_calls.is_empty() {
                let mut tool_results: Vec<AnthropicMessage> = Vec::new();

                let (serial_calls, parallel_calls): (Vec<_>, Vec<_>) = pending_tool_calls
                    .into_iter()
                    .partition::<Vec<_>, _>(|tc| tc.requires_user_interaction);

                for tc in &serial_calls {
                    let (.., output_str, is_error) = self.execute_tool_call(&tc.name, &tc.input, &tc.id).await;
                    let _ = self.event_tx.send(EngineEvent::ToolUseDone {
                        tool_use_id: tc.id.clone(),
                        tool_name: tc.name.clone(),
                        tool_input: tc.input.clone(),
                        output: output_str.clone(),
                        is_error,
                    }).await;
                    let compacted = compact_tool_output(&tc.name, &output_str);
                    tool_results.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: AnthropicContent::Blocks(vec![ContentBlock::ToolResult {
                            tool_use_id: tc.id.clone(),
                            content: compacted,
                            is_error: Some(is_error),
                        }]),
                    });
                }

                if !parallel_calls.is_empty() {
                    let total = parallel_calls.len();
                    let completed_count = Arc::new(AtomicUsize::new(serial_calls.len()));
                    let event_tx_clone = self.event_tx.clone();
                    let mcp_registry_clone = self.mcp_registry.clone();
                    let workspace_cwd_clone = self.workspace_cwd.clone();
                    let permission_manager_clone = self.permission_manager.clone();
                    let conv_id_clone = self.conv_id.clone();

                    let mut futures = Vec::new();
                    for tc in parallel_calls {
                        let fut = execute_tool_standalone(
                            tc.name,
                            tc.input.clone(),
                            tc.id.clone(),
                            event_tx_clone.clone(),
                            mcp_registry_clone.clone(),
                            workspace_cwd_clone.clone(),
                            permission_manager_clone.clone(),
                            conv_id_clone.clone(),
                            completed_count.clone(),
                            total + serial_calls.len(),
                        );
                        futures.push(fut);
                    }

                    let results = futures::future::join_all(futures).await;

                    for (tool_use_id, tool_name, _tool_input, output_str, is_error) in results {
                        let compacted = compact_tool_output(&tool_name, &output_str);
                        tool_results.push(AnthropicMessage {
                            role: "user".to_string(),
                            content: AnthropicContent::Blocks(vec![ContentBlock::ToolResult {
                                tool_use_id: tool_use_id.clone(),
                                content: compacted,
                                is_error: Some(is_error),
                            }]),
                        });
                    }
                }

                conversation_messages.push(AnthropicMessage {
                    role: "assistant".to_string(),
                    content: AnthropicContent::Blocks(assistant_blocks),
                });

                for tool_result_msg in tool_results {
                    conversation_messages.push(tool_result_msg);
                }
            } else {
                break;
            }

            if iteration == self.max_tool_iterations - 1 {
                let _ = self.event_tx.send(EngineEvent::Error("Max tool iterations reached".to_string())).await;
                let _ = self.event_tx.send(EngineEvent::MessageStop {
                    full_text: full_text.clone(),
                    stop_reason: Some("max_iterations".to_string()),
                }).await;
                break;
            }

            // Auto-resume when stopped due to max_tokens
            if let Some(ref sr) = stop_reason {
                if sr == "max_tokens" {
                    tracing::info!(module = "AnthropicLoop", "Response truncated by max_tokens, auto-resuming...");
                    let _ = self.event_tx.send(EngineEvent::Text("\n... [continuing] ...".to_string())).await;

                    // Check if assistant_blocks was already moved by has_tool_use branch
                    // Use full_text for persistence since it's always available
                    let conv_id = self.conv_id.as_deref().unwrap_or("unknown");
                    let file_path = if full_text.len() > 2000 {
                        persist_large_output(&full_text, conv_id, "anthro")
                    } else {
                        String::new()
                    };

                    let compact_content = if !file_path.is_empty() {
                        build_compact_continuation_message(&full_text, &file_path, 1500)
                    } else {
                        full_text.clone()
                    };

                    // Push compact assistant response (always available, unlike assistant_blocks which may be moved)
                    conversation_messages.push(AnthropicMessage {
                        role: "assistant".to_string(),
                        content: AnthropicContent::Text(compact_content),
                    });

                    // Add a continuation prompt so the model picks up where it left off
                    conversation_messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: AnthropicContent::Text("Continue exactly where you left off. Do not repeat anything.".to_string()),
                    });

                    // Reset state for the next iteration
                    assistant_blocks = Vec::new();
                    current_text = String::new();
                    current_thinking = String::new();
                    current_tool_use_id = None;
                    current_tool_name = None;
                    stop_reason = None;
                    has_tool_use = false;

                    continue;
                }
            }
        }

        Ok((full_text, stop_reason))
    }

    async fn execute_openai_loop(&mut self) -> Result<(String, Option<String>)> {
        tracing::info!(module = "OpenAILoop", "ENTRY: building messages, msg_count={}", self.messages.len());
        let mut conversation_messages: Vec<OpenAIMessage> = self.build_openai_messages();
        tracing::info!(module = "OpenAILoop", "Messages built, count={}", conversation_messages.len());

        let tools = get_tool_definitions();
        tracing::info!(module = "OpenAILoop", "Tool definitions loaded, count={}, mode={:?}", tools.len(), self.reasoning_mode);

        if matches!(self.reasoning_mode, ReasoningMode::Deep) {
            if let Some(ref sp) = self.system_prompt {
                self.system_prompt = Some(format!("{}\n\nYou are in deep reasoning mode. For any destructive operations, you MUST ask for user confirmation before proceeding.", sp));
            } else {
                self.system_prompt = Some("You are in deep reasoning mode. For any destructive operations, you MUST ask for user confirmation before proceeding.".to_string());
            }
        }

        let mut full_text = String::new();
        let mut stop_reason = None;
        let base_url = self.provider.provider.base_url.clone();
        let model_id = self.provider.model.id.clone();
        let api_key_len = self.provider.provider.api_key.len();

        for iteration in 0..self.max_tool_iterations {
            tracing::info!(module = "OpenAILoop", "Iteration {}/{} starting, base_url={}, model={}", iteration+1, self.max_tool_iterations, base_url, model_id);
            tracing::info!(module = "OpenAILoop", "API key length={}", api_key_len);

            // Context compression check before each API call
            if iteration > 0 {
                let model_context_limit = TokenCounter::get_safe_limit(&model_id, 0.90);
                let conv_values: Vec<Value> = conversation_messages.iter().filter_map(|m| {
                    let content_value = match &m.content {
                        OpenAIContent::Text(t) => json!(t),
                        OpenAIContent::Multi(parts) => {
                            let arr: Vec<Value> = parts.iter().filter_map(|p| {
                                match p {
                                    crate::native_engine::openai_client::OpenAIContentPart::Text { text } => Some(json!({"type": "text", "text": text})),
                                    crate::native_engine::openai_client::OpenAIContentPart::Image { image_url } => Some(json!({"type": "image_url", "image_url": {"url": &image_url.url}})),
                                }
                            }).collect();
                            json!(arr)
                        }
                    };
                    let mut obj = json!({"role": m.role, "content": content_value});
                    if let Some(ref tc) = m.tool_calls {
                        if let Ok(tc_val) = serde_json::to_value(tc) {
                            obj["tool_calls"] = tc_val;
                        }
                    }
                    if let Some(ref tid) = m.tool_call_id {
                        obj["tool_call_id"] = json!(tid);
                    }
                    Some(obj)
                }).collect();
                let estimated = TokenCounter::estimate(&conv_values);
                if estimated > (model_context_limit * 70 / 100) {
                    tracing::info!(module = "OpenAILoop", "Context compression needed: {} tokens (limit: {})", estimated, model_context_limit);
                    let compressor = ContextCompressor::new(None);
                    let context_window = ContextWindow {
                        model_id: model_id.clone(),
                        total_tokens: estimated as u64,
                        reserved_for_completion: (self.max_tokens as u64) * 2,
                        context_window: model_context_limit as u64,
                    };
                    let level = compressor.determine_compression_level(&context_window);
                    if level != CompressionLevel::None {
                        let conv_values: Vec<Value> = conversation_messages.iter().filter_map(|m| {
                            let content_value = match &m.content {
                                OpenAIContent::Text(t) => json!(t),
                                OpenAIContent::Multi(parts) => {
                                    let arr: Vec<Value> = parts.iter().filter_map(|p| {
                                        match p {
                                            crate::native_engine::openai_client::OpenAIContentPart::Text { text } => Some(json!({"type": "text", "text": text})),
                                            crate::native_engine::openai_client::OpenAIContentPart::Image { image_url } => Some(json!({"type": "image_url", "image_url": {"url": &image_url.url}})),
                                        }
                                    }).collect();
                                    json!(arr)
                                }
                            };
                            let mut obj = json!({"role": m.role, "content": content_value});
                            if let Some(ref tc) = m.tool_calls {
                                if let Ok(tc_val) = serde_json::to_value(tc) {
                                    obj["tool_calls"] = tc_val;
                                }
                            }
                            if let Some(ref tid) = m.tool_call_id {
                                obj["tool_call_id"] = json!(tid);
                            }
                            Some(obj)
                        }).collect();
                        let (compressed, metadata) = compressor.compress(
                            conv_values,
                            self.system_prompt.clone(),
                            level,
                            Some(&self.provider),
                        ).await;
                        if metadata.was_compressed {
                            conversation_messages = compressed.iter().filter_map(|v| {
                                let role = v.get("role")?.as_str()?.to_string();
                                let content = v.get("content")?;
                                let openai_content = if content.is_string() {
                                    OpenAIContent::Text(content.as_str()?.to_string())
                                } else if content.is_array() {
                                    let parts: Vec<crate::native_engine::openai_client::OpenAIContentPart> = content.as_array()?.iter().filter_map(|p| {
                                        let pt = p.get("type")?.as_str()?;
                                        match pt {
                                            "text" => Some(crate::native_engine::openai_client::OpenAIContentPart::Text { text: p.get("text")?.as_str()?.to_string() }),
                                            "image_url" => Some(crate::native_engine::openai_client::OpenAIContentPart::Image {
                                                image_url: crate::native_engine::openai_client::ImageUrl { url: p.get("image_url")?.get("url")?.as_str()?.to_string() },
                                            }),
                                            _ => None,
                                        }
                                    }).collect();
                                    OpenAIContent::Multi(parts)
                                } else {
                                    return None;
                                };
                                let tool_calls = v.get("tool_calls").and_then(|tc| serde_json::from_value(tc.clone()).ok());
                                let tool_call_id = v.get("tool_call_id").and_then(|id| id.as_str()).map(String::from);
                                Some(OpenAIMessage { role, content: openai_content, tool_calls, tool_call_id, reasoning_content: None })
                            }).collect();
                            tracing::info!(module = "OpenAILoop", "Context compressed: {} -> {} messages, saved {} tokens",
                                metadata.original_count, metadata.compressed_count, metadata.token_reduction);
                        }
                    }
                }
            }

            self.streaming_tool_args.clear();
            let stream = self.openai_client
                .send_message_stream(
                    &self.provider,
                    conversation_messages.clone(),
                    self.system_prompt.as_deref(),
                    tools.clone(),
                    self.max_tokens,
                )
                .await;

            let mut stream = match stream {
                Ok(s) => {
                    tracing::info!(module = "OpenAILoop", "Stream created successfully");
                    s
                }
                Err(e) => {
                    let err_msg = format!("API request failed: {}", e);
                    tracing::error!(module = "OpenAILoop", "Stream creation FAILED: {}", err_msg);
                    let _ = self.event_tx.send(EngineEvent::Error(err_msg.clone())).await;
                    return Err(anyhow!(err_msg));
                }
            };

            let mut sse_buffer = String::new();
            let mut has_tool_calls = false;
            let mut assistant_content: Option<OpenAIContent> = None;
            let mut assistant_reasoning: Option<String> = None;
            let mut assistant_tool_calls: Vec<crate::native_engine::openai_client::OpenAIToolCall> = Vec::new();
            let mut openai_tool_args: HashMap<usize, (String, String, String)> = HashMap::new();

            while let Some(chunk_result) = stream.next().await {
                if self.event_tx.is_closed() {
                    drop(stream);
                    break;
                }
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => {
                        let err_msg = format!("Stream error: {}", e);
                        tracing::error!(module = "OpenAILoop", "Stream chunk error: {}", err_msg);
                        let _ = self.event_tx.send(EngineEvent::Error(err_msg)).await;
                        let _ = self.event_tx.send(EngineEvent::MessageStop {
                            full_text: full_text.clone(),
                            stop_reason: Some("stream_error".to_string()),
                        }).await;
                        drop(stream);
                        return Ok((full_text, Some("stream_error".to_string())));
                    }
                };

                sse_buffer.push_str(&chunk);
                let consumed = consume_sse_payloads(&sse_buffer);
                sse_buffer = consumed.remainder;

                for payload in &consumed.payloads {
                    if payload == "[DONE]" {
                        continue;
                    }

                    let event: Value = match serde_json::from_str(payload) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    let choices = match event.get("choices").and_then(|c| c.as_array()) {
                        Some(c) => c,
                        None => continue,
                    };

                    for choice in choices {
                        let delta = match choice.get("delta") {
                            Some(d) => d,
                            None => continue,
                        };

                        if let Some(role) = delta.get("role").and_then(|r| r.as_str()) {
                            if role == "assistant" {
                                let _ = self.event_tx.send(EngineEvent::MessageStart {
                                    model: self.provider.model.id.clone(),
                                }).await;
                            }
                        }

                        if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                            if !content.is_empty() {
                                full_text.push_str(content);
                                let _ = self.event_tx.send(EngineEvent::Text(content.to_string())).await;
                                match &assistant_content {
                                    None => {
                                        assistant_content = Some(OpenAIContent::Text(content.to_string()));
                                    }
                                    Some(OpenAIContent::Text(existing)) => {
                                        assistant_content = Some(OpenAIContent::Text(format!("{}{}", existing, content)));
                                    }
                                    Some(OpenAIContent::Multi(_)) => {}
                                }
                            }
                        }

                        if let Some(reasoning) = delta.get("reasoning_content").and_then(|r| r.as_str()) {
                            if !reasoning.is_empty() {
                                match &mut assistant_reasoning {
                                    None => assistant_reasoning = Some(reasoning.to_string()),
                                    Some(r) => r.push_str(reasoning),
                                }
                                let _ = self.event_tx.send(EngineEvent::Thinking(reasoning.to_string())).await;
                            }
                        }

                        if let Some(tool_calls_arr) = delta.get("tool_calls").and_then(|tc| tc.as_array()) {
                            has_tool_calls = true;
                            for tc_delta in tool_calls_arr {
                                let index = tc_delta.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;

                                let entry = openai_tool_args.entry(index).or_insert_with(|| (String::new(), String::new(), String::new()));

                                if let Some(id) = tc_delta.get("id").and_then(|i| i.as_str()) {
                                    entry.0 = id.to_string();
                                }
                                if let Some(call_type) = tc_delta.get("type").and_then(|t| t.as_str()) {
                                    entry.1 = call_type.to_string();
                                }
                                if let Some(func) = tc_delta.get("function") {
                                    if let Some(name) = func.get("name").and_then(|n| n.as_str()) {
                                        entry.1 = name.to_string();
                                    }
                                    if let Some(args) = func.get("arguments").and_then(|a| a.as_str()) {
                                        entry.2.push_str(args);
                                    }
                                }
                            }
                        }

                        if let Some(finish) = choice.get("finish_reason").and_then(|f| f.as_str()) {
                            if finish != "tool_calls" {
                                stop_reason = Some(finish.to_string());
                                let _ = self.event_tx.send(EngineEvent::MessageDelta {
                                    stop_reason: Some(finish.to_string()),
                                }).await;
                            }
                        }
                    }

                    if let Some(usage) = event.get("usage") {
                        if let Some(prompt_tokens) = usage.get("prompt_tokens").and_then(|v| v.as_f64()) {
                            crate::metrics::TOKENS_CONSUMED.inc_by(prompt_tokens);
                        }
                        if let Some(completion_tokens) = usage.get("completion_tokens").and_then(|v| v.as_f64()) {
                            crate::metrics::TOKENS_CONSUMED.inc_by(completion_tokens);
                        }
                        let _ = self.event_tx.send(EngineEvent::Usage(usage.clone())).await;
                    }
                }
            }

            if has_tool_calls {
                let mut indices: Vec<usize> = openai_tool_args.keys().copied().collect();
                indices.sort();

                let mut pending_tool_calls: Vec<PendingToolCall> = Vec::new();

                for idx in &indices {
                    let (id, name, args_str) = &openai_tool_args[idx];
                    let input: Value = serde_json::from_str(args_str).unwrap_or_else(|_| {
                        recover_malformed_tool_input(name, args_str).unwrap_or(json!({}))
                    });

                    let _ = self.event_tx.send(EngineEvent::ToolUseStart {
                        tool_use_id: id.clone(),
                        tool_name: name.clone(),
                        tool_input: input.clone(),
                        text_before: full_text.clone(),
                    }).await;

                    let requires_user_interaction = name == "AskUserQuestion"
                        || self.check_permission_requires_confirmation(name, &input);

                    assistant_tool_calls.push(crate::native_engine::openai_client::OpenAIToolCall {
                        id: id.clone(),
                        call_type: "function".to_string(),
                        function: crate::native_engine::openai_client::FunctionCall {
                            name: name.clone(),
                            arguments: args_str.clone(),
                        },
                    });

                    pending_tool_calls.push(PendingToolCall {
                        id: id.clone(),
                        name: name.clone(),
                        input,
                        requires_user_interaction,
                    });
                }

                let mut tool_results: Vec<OpenAIMessage> = Vec::new();

                let (serial_calls, parallel_calls): (Vec<_>, Vec<_>) = pending_tool_calls
                    .into_iter()
                    .partition::<Vec<_>, _>(|tc| tc.requires_user_interaction);

                for tc in &serial_calls {
                    let (.., output_str, is_error) = self.execute_tool_call(&tc.name, &tc.input, &tc.id).await;
                    let _ = self.event_tx.send(EngineEvent::ToolUseDone {
                        tool_use_id: tc.id.clone(),
                        tool_name: tc.name.clone(),
                        tool_input: tc.input.clone(),
                        output: output_str.clone(),
                        is_error,
                    }).await;
                    let compacted = compact_tool_output(&tc.name, &output_str);
                    tool_results.push(OpenAIMessage {
                        role: "tool".to_string(),
                        content: OpenAIContent::Text(compacted),
                        tool_calls: None,
                        tool_call_id: Some(tc.id.clone()),
                        reasoning_content: None,
                    });
                }

                if !parallel_calls.is_empty() {
                    let total = parallel_calls.len();
                    let completed_count = Arc::new(AtomicUsize::new(serial_calls.len()));
                    let event_tx_clone = self.event_tx.clone();
                    let mcp_registry_clone = self.mcp_registry.clone();
                    let workspace_cwd_clone = self.workspace_cwd.clone();
                    let permission_manager_clone = self.permission_manager.clone();
                    let conv_id_clone = self.conv_id.clone();

                    let mut futures = Vec::new();
                    for tc in parallel_calls {
                        let fut = execute_tool_standalone(
                            tc.name,
                            tc.input.clone(),
                            tc.id.clone(),
                            event_tx_clone.clone(),
                            mcp_registry_clone.clone(),
                            workspace_cwd_clone.clone(),
                            permission_manager_clone.clone(),
                            conv_id_clone.clone(),
                            completed_count.clone(),
                            total + serial_calls.len(),
                        );
                        futures.push(fut);
                    }

                    let results = futures::future::join_all(futures).await;

                    for (tool_use_id, tool_name, _tool_input, output_str, _is_error) in results {
                        let compacted = compact_tool_output(&tool_name, &output_str);
                        tool_results.push(OpenAIMessage {
                            role: "tool".to_string(),
                            content: OpenAIContent::Text(compacted),
                            tool_calls: None,
                            tool_call_id: Some(tool_use_id),
                            reasoning_content: None,
                        });
                    }
                }

                conversation_messages.push(OpenAIMessage {
                    role: "assistant".to_string(),
                    content: assistant_content.unwrap_or(OpenAIContent::Text(String::new())),
                    tool_calls: Some(assistant_tool_calls.clone()),
                    tool_call_id: None,
                    reasoning_content: assistant_reasoning.clone(),
                });

                for tool_result_msg in tool_results {
                    conversation_messages.push(tool_result_msg);
                }
            } else {
                break;
            }

            if iteration == self.max_tool_iterations - 1 {
                let _ = self.event_tx.send(EngineEvent::Error("Max tool iterations reached".to_string())).await;
                let _ = self.event_tx.send(EngineEvent::MessageStop {
                    full_text: full_text.clone(),
                    stop_reason: Some("max_iterations".to_string()),
                }).await;
                break;
            }

            // Auto-resume when stopped due to max_tokens
            if let Some(ref sr) = stop_reason {
                if sr == "max_tokens" || sr == "length" {
                    tracing::info!(module = "OpenAILoop", "Response truncated by max_tokens, auto-resuming...");

                    // Persist large output to file instead of bloating context
                    let conv_id = self.conv_id.as_deref().unwrap_or("unknown");
                    let file_path = if full_text.len() > 2000 {
                        persist_large_output(&full_text, conv_id, "openai")
                    } else {
                        String::new()
                    };

                    let compact_content = if !file_path.is_empty() {
                        build_compact_continuation_message(&full_text, &file_path, 1500)
                    } else {
                        full_text.clone()
                    };

                    conversation_messages.push(OpenAIMessage {
                        role: "assistant".to_string(),
                        content: OpenAIContent::Text(compact_content),
                        tool_calls: Some(assistant_tool_calls.clone()),
                        tool_call_id: None,
                        reasoning_content: assistant_reasoning.clone(),
                    });

                    // 续传提示：追加一个 user 消息让模型继续
                    conversation_messages.push(OpenAIMessage {
                        role: "user".to_string(),
                        content: OpenAIContent::Text("Continue exactly where you left off. Do not repeat anything. Write the continuation directly into the file mentioned above.".to_string()),
                        tool_calls: None,
                        tool_call_id: None,
                        reasoning_content: None,
                    });

                    let _ = self.event_tx.send(EngineEvent::Text("\n... [continuing] ...".to_string())).await;
                    // 重置状态，下一轮循环重新积累
                    stop_reason = None;
                    assistant_tool_calls.clear();
                    assistant_reasoning = None;
                    assistant_content = None;
                    openai_tool_args.clear();
                    has_tool_calls = false;
                    continue;
                }
            }
        }

        Ok((full_text, stop_reason))
    }

    fn build_anthropic_messages(&self) -> Vec<AnthropicMessage> {
        self.messages.iter().filter_map(|msg| {
            let role = msg.get("role")?.as_str()?;
            let content = msg.get("content")?;

            let anthropic_content = if content.is_string() {
                AnthropicContent::Text(content.as_str()?.to_string())
            } else if content.is_array() {
                let blocks: Vec<ContentBlock> = content.as_array()?.iter().filter_map(|block| {
                    let block_type = block.get("type")?.as_str()?;
                    match block_type {
                        "text" => {
                            let text = block.get("text")?.as_str()?.to_string();
                            Some(ContentBlock::Text { text })
                        }
                        "image" => {
                            let source = block.get("source")?;
                            Some(ContentBlock::Image {
                                source: crate::native_engine::anthropic_client::ImageSource {
                                    source_type: source.get("type")?.as_str()?.to_string(),
                                    media_type: source.get("media_type")?.as_str()?.to_string(),
                                    data: source.get("data")?.as_str()?.to_string(),
                                },
                            })
                        }
                        "tool_result" => {
                            Some(ContentBlock::ToolResult {
                                tool_use_id: block.get("tool_use_id")?.as_str()?.to_string(),
                                content: block.get("content")?.as_str()?.to_string(),
                                is_error: block.get("is_error").and_then(|v| v.as_bool()),
                            })
                        }
                        _ => None,
                    }
                }).collect();
                AnthropicContent::Blocks(blocks)
            } else {
                return None;
            };

            Some(AnthropicMessage {
                role: role.to_string(),
                content: anthropic_content,
            })
        }).collect()
    }

    fn build_openai_messages(&self) -> Vec<OpenAIMessage> {
        self.messages.iter().filter_map(|msg| {
            let role = msg.get("role")?.as_str()?;
            let content = msg.get("content")?;

            let openai_content = if content.is_string() {
                OpenAIContent::Text(content.as_str()?.to_string())
            } else if content.is_array() {
                let parts: Vec<crate::native_engine::openai_client::OpenAIContentPart> = content.as_array()?.iter().filter_map(|part| {
                    let part_type = part.get("type")?.as_str()?;
                    match part_type {
                        "text" => {
                            Some(crate::native_engine::openai_client::OpenAIContentPart::Text {
                                text: part.get("text")?.as_str()?.to_string(),
                            })
                        }
                        "image_url" => {
                            let url_obj = part.get("image_url")?;
                            Some(crate::native_engine::openai_client::OpenAIContentPart::Image {
                                image_url: crate::native_engine::openai_client::ImageUrl {
                                    url: url_obj.get("url")?.as_str()?.to_string(),
                                },
                            })
                        }
                        _ => None,
                    }
                }).collect();
                OpenAIContent::Multi(parts)
            } else {
                return None;
            };

            let tool_calls = msg.get("tool_calls").and_then(|tc| {
                serde_json::from_value(tc.clone()).ok()
            });

            let tool_call_id = msg.get("tool_call_id").and_then(|id| id.as_str()).map(String::from);

            let reasoning_content = msg.get("reasoning_content").and_then(|r| r.as_str()).map(String::from);

            Some(OpenAIMessage {
                role: role.to_string(),
                content: openai_content,
                tool_calls,
                tool_call_id,
                reasoning_content,
            })
        }).collect()
    }

    fn get_workspace_cwd(&self) -> &str {
        &self.workspace_cwd
    }

    fn handle_streaming_tool_arg_delta(&mut self, tool_use_id: &str, tool_name: &str, delta: &str) {
        let prev_args = self.streaming_tool_args
            .get(tool_use_id)
            .map(|s| s.accumulated_args.clone())
            .unwrap_or_default();

        let merged = merge_tool_args(&prev_args, delta);

        let delta_to_emit = if merged.starts_with(&prev_args) && !prev_args.is_empty() {
            merged[prev_args.len()..].to_string()
        } else {
            delta.to_string()
        };

        self.streaming_tool_args.insert(
            tool_use_id.to_string(),
            StreamingToolCall {
                name: tool_name.to_string(),
                accumulated_args: merged,
            },
        );

        if !delta_to_emit.is_empty() {
            let _ = self.event_tx.try_send(EngineEvent::ToolArgDelta {
                tool_use_id: tool_use_id.to_string(),
                delta: delta_to_emit,
            });
        }
    }

    fn finalize_streaming_tool_args(&mut self, tool_use_id: &str) -> Value {
        if let Some(stc) = self.streaming_tool_args.remove(tool_use_id) {
            let parsed: Option<Value> = serde_json::from_str(&stc.accumulated_args).ok();
            parsed.or_else(|| recover_malformed_tool_input(&stc.name, &stc.accumulated_args))
                .unwrap_or(json!({}))
        } else {
            json!({})
        }
    }
}
