pub mod cron;

use crate::db::DbManager;
use crate::native_engine::anthropic_client::AnthropicClient;
use crate::native_engine::openai_client::OpenAIClient;
use crate::native_engine::provider_manager::{ApiFormat, ProviderManager, ResolvedProvider};
use anyhow::Result;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequest {
    pub task_id: String,
    pub prompt: String,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub context: Option<Vec<serde_json::Value>>,
    pub tools: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEvent {
    pub task_id: String,
    pub event_type: String,
    pub data: serde_json::Value,
}

pub struct TaskExecutor {
    provider_manager: Arc<Mutex<ProviderManager>>,
    db_manager: Arc<DbManager>,
    anthropic_client: AnthropicClient,
    openai_client: OpenAIClient,
    tasks: Arc<Mutex<HashMap<String, TaskState>>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TaskState {
    task_id: String,
    status: TaskStatus,
    result: Option<TaskResult>,
    started_at: std::time::Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl TaskExecutor {
    pub fn new_with_provider_manager(
        provider_manager: Arc<Mutex<ProviderManager>>,
        db_manager: Arc<DbManager>,
    ) -> Self {
        Self {
            provider_manager,
            db_manager,
            anthropic_client: AnthropicClient::new(),
            openai_client: OpenAIClient::new(),
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn resolve_provider(&self, model_id: &str) -> Result<ResolvedProvider> {
        let pm = self.provider_manager.lock().await;
        pm.resolve_provider(model_id).await
            .ok_or_else(|| anyhow::anyhow!("No provider found for model '{}'", model_id))
    }

    pub async fn execute_task(
        &self,
        request: TaskRequest,
    ) -> Result<TaskResult> {
        let task_id = request.task_id.clone();
        let start_time = std::time::Instant::now();

        {
            let mut tasks = self.tasks.lock().await;
            tasks.insert(task_id.clone(), TaskState {
                task_id: task_id.clone(),
                status: TaskStatus::Pending,
                result: None,
                started_at: start_time,
            });
        }

        let model = request.model.clone().unwrap_or_else(|| "claude-sonnet-4-6".to_string());
        let max_tokens = request.max_tokens.unwrap_or(4096);

        let resolved = match self.resolve_provider(&model).await {
            Ok(r) => r,
            Err(e) => {
                let duration_ms = start_time.elapsed().as_millis() as u64;
                let result = TaskResult {
                    task_id: task_id.clone(),
                    success: false,
                    output: None,
                    error: Some(e.to_string()),
                    duration_ms,
                };
                let mut tasks = self.tasks.lock().await;
                if let Some(state) = tasks.get_mut(&task_id) {
                    state.status = TaskStatus::Failed;
                    state.result = Some(result.clone());
                }
                return Ok(result);
            }
        };

        {
            let mut tasks = self.tasks.lock().await;
            if let Some(state) = tasks.get_mut(&task_id) {
                state.status = TaskStatus::Running;
            }
        }

        let system_prompt = r#"You are a sub-agent executing a specific task. Your role is to complete the assigned task efficiently and report back the results.

Guidelines:
- Stay focused on the specific task assigned
- Report progress periodically
- Return results in a structured format
- If you encounter errors, report them clearly
"#;

        let mut attempts = 0;
        let max_attempts = 3;
        let mut last_error = None;

        while attempts < max_attempts {
            attempts += 1;

            let result = match resolved.provider.api_format {
                ApiFormat::Anthropic => {
                    self.execute_anthropic(&resolved, &request, system_prompt, max_tokens).await
                }
                ApiFormat::OpenAI => {
                    self.execute_openai(&resolved, &request, system_prompt, max_tokens).await
                }
            };

            match result {
                Ok(output) => {
                    let duration_ms = start_time.elapsed().as_millis() as u64;
                    let result = TaskResult {
                        task_id: task_id.clone(),
                        success: true,
                        output: Some(output),
                        error: None,
                        duration_ms,
                    };

                    {
                        let mut tasks = self.tasks.lock().await;
                        if let Some(state) = tasks.get_mut(&task_id) {
                            state.status = TaskStatus::Completed;
                            state.result = Some(result.clone());
                        }
                    }

                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                }
            }

            if attempts < max_attempts {
                tokio::time::sleep(Duration::from_secs(2 * attempts as u64)).await;
            }
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let error_msg = last_error.unwrap_or_else(|| "Unknown error".to_string());
        let result = TaskResult {
            task_id: task_id.clone(),
            success: false,
            output: None,
            error: Some(error_msg.clone()),
            duration_ms,
        };

        {
            let mut tasks = self.tasks.lock().await;
            if let Some(state) = tasks.get_mut(&task_id) {
                state.status = TaskStatus::Failed;
                state.result = Some(result.clone());
            }
        }

        Ok(result)
    }

    async fn execute_anthropic(
        &self,
        resolved: &ResolvedProvider,
        request: &TaskRequest,
        system_prompt: &str,
        max_tokens: u32,
    ) -> Result<String> {
        use crate::native_engine::anthropic_client::{AnthropicContent, AnthropicMessage, ContentBlock};

        let mut messages = Vec::new();

        if let Some(context) = &request.context {
            for ctx in context {
                messages.push(AnthropicMessage {
                    role: "user".to_string(),
                    content: AnthropicContent::Blocks(vec![ContentBlock::Text {
                        text: serde_json::to_string(ctx).unwrap_or_default(),
                    }]),
                });
            }
        }

        messages.push(AnthropicMessage {
            role: "user".to_string(),
            content: AnthropicContent::Blocks(vec![ContentBlock::Text {
                text: format!("Task: {}\n\nPlease execute this task and report your results.", request.prompt),
            }]),
        });

        let response = self.anthropic_client
            .send_message(resolved, messages, Some(system_prompt), vec![], max_tokens)
            .await?;

        let output = response.content.iter()
            .filter_map(|block| {
                if let ContentBlock::Text { text } = block {
                    Some(text.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(output)
    }

    async fn execute_openai(
        &self,
        resolved: &ResolvedProvider,
        request: &TaskRequest,
        system_prompt: &str,
        max_tokens: u32,
    ) -> Result<String> {
        use crate::native_engine::openai_client::{OpenAIContent, OpenAIMessage};

        let mut messages = Vec::new();

        if let Some(context) = &request.context {
            for ctx in context {
                messages.push(OpenAIMessage {
                    role: "user".to_string(),
                    content: OpenAIContent::Text(serde_json::to_string(ctx).unwrap_or_default()),
                    tool_calls: None,
                    tool_call_id: None,
                    reasoning_content: None,
                });
            }
        }

        messages.push(OpenAIMessage {
            role: "user".to_string(),
            content: OpenAIContent::Text(format!("Task: {}\n\nPlease execute this task and report your results.", request.prompt)),
            tool_calls: None,
            tool_call_id: None,
            reasoning_content: None,
        });

        let response = self.openai_client
            .send_message(resolved, messages, Some(system_prompt), vec![], max_tokens)
            .await?;

        let output = response.choices.iter()
            .filter_map(|choice| {
                match &choice.message.content {
                    OpenAIContent::Text(text) => Some(text.clone()),
                    OpenAIContent::Multi(parts) => {
                        let text: Vec<String> = parts.iter()
                            .filter_map(|p| {
                                if let crate::native_engine::openai_client::OpenAIContentPart::Text { text } = p {
                                    Some(text.clone())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        if text.is_empty() { None } else { Some(text.join("\n")) }
                    }
                }
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(output)
    }

    pub async fn execute_task_streaming(
        &self,
        request: TaskRequest,
    ) -> Result<TaskResult> {
        let task_id = request.task_id.clone();
        let start_time = std::time::Instant::now();

        {
            let mut tasks = self.tasks.lock().await;
            tasks.insert(task_id.clone(), TaskState {
                task_id: task_id.clone(),
                status: TaskStatus::Running,
                result: None,
                started_at: start_time,
            });
        }

        let model = request.model.clone().unwrap_or_else(|| "claude-sonnet-4-6".to_string());
        let max_tokens = request.max_tokens.unwrap_or(4096);

        let resolved = match self.resolve_provider(&model).await {
            Ok(r) => r,
            Err(e) => {
                let duration_ms = start_time.elapsed().as_millis() as u64;
                let result = TaskResult {
                    task_id: task_id.clone(),
                    success: false,
                    output: None,
                    error: Some(e.to_string()),
                    duration_ms,
                };
                let mut tasks = self.tasks.lock().await;
                if let Some(state) = tasks.get_mut(&task_id) {
                    state.status = TaskStatus::Failed;
                    state.result = Some(result.clone());
                }
                return Ok(result);
            }
        };

        let system_prompt = "You are a sub-agent executing a specific task. Your role is to complete the assigned task efficiently and report back the results.";

        let stream_result = match resolved.provider.api_format {
            ApiFormat::Anthropic => {
                self.execute_anthropic_streaming(&resolved, &request, system_prompt, max_tokens).await
            }
            ApiFormat::OpenAI => {
                self.execute_openai_streaming(&resolved, &request, system_prompt, max_tokens).await
            }
        };

        let full_output = match stream_result {
            Ok(output) => output,
            Err(e) => {
                let duration_ms = start_time.elapsed().as_millis() as u64;
                let result = TaskResult {
                    task_id: task_id.clone(),
                    success: false,
                    output: None,
                    error: Some(e.to_string()),
                    duration_ms,
                };
                let mut tasks = self.tasks.lock().await;
                if let Some(state) = tasks.get_mut(&task_id) {
                    state.status = TaskStatus::Failed;
                    state.result = Some(result.clone());
                }
                return Ok(result);
            }
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let result = TaskResult {
            task_id: task_id.clone(),
            success: true,
            output: Some(full_output),
            error: None,
            duration_ms,
        };

        {
            let mut tasks = self.tasks.lock().await;
            if let Some(state) = tasks.get_mut(&task_id) {
                state.status = TaskStatus::Completed;
                state.result = Some(result.clone());
            }
        }

        Ok(result)
    }

    async fn execute_anthropic_streaming(
        &self,
        resolved: &ResolvedProvider,
        request: &TaskRequest,
        system_prompt: &str,
        max_tokens: u32,
    ) -> Result<String> {
        use crate::native_engine::anthropic_client::{AnthropicContent, AnthropicMessage, ContentBlock};

        let mut messages = Vec::new();

        if let Some(context) = &request.context {
            for ctx in context {
                messages.push(AnthropicMessage {
                    role: "user".to_string(),
                    content: AnthropicContent::Blocks(vec![ContentBlock::Text {
                        text: serde_json::to_string(ctx).unwrap_or_default(),
                    }]),
                });
            }
        }

        messages.push(AnthropicMessage {
            role: "user".to_string(),
            content: AnthropicContent::Blocks(vec![ContentBlock::Text {
                text: format!("Task: {}", request.prompt),
            }]),
        });

        let mut stream = self.anthropic_client
            .send_message_stream(resolved, messages, Some(system_prompt), vec![], max_tokens)
            .await?;

        let mut full_output = String::new();
        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    buffer.push_str(&chunk);
                    while let Some(pos) = buffer.find('\n') {
                        let line = buffer[..pos].to_string();
                        buffer = buffer[pos + 1..].to_string();

                        if line.starts_with("data: ") {
                            let data = &line[6..];
                            if data != "[DONE]" {
                                if let Ok(event_data) = serde_json::from_str::<serde_json::Value>(data) {
                                    if let Some(delta) = event_data.get("delta")
                                        .and_then(|d| d.get("text"))
                                        .and_then(|t| t.as_str())
                                    {
                                        full_output.push_str(delta);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(module = "TaskExecutor", "Stream error: {}", e);
                }
            }
        }

        Ok(full_output)
    }

    async fn execute_openai_streaming(
        &self,
        resolved: &ResolvedProvider,
        request: &TaskRequest,
        system_prompt: &str,
        max_tokens: u32,
    ) -> Result<String> {
        use crate::native_engine::openai_client::{OpenAIContent, OpenAIMessage};

        let mut messages = Vec::new();

        if let Some(context) = &request.context {
            for ctx in context {
                messages.push(OpenAIMessage {
                    role: "user".to_string(),
                    content: OpenAIContent::Text(serde_json::to_string(ctx).unwrap_or_default()),
                    tool_calls: None,
                    tool_call_id: None,
                    reasoning_content: None,
                });
            }
        }

        messages.push(OpenAIMessage {
            role: "user".to_string(),
            content: OpenAIContent::Text(format!("Task: {}", request.prompt)),
            tool_calls: None,
            tool_call_id: None,
            reasoning_content: None,
        });

        let mut stream = self.openai_client
            .send_message_stream(resolved, messages, Some(system_prompt), vec![], max_tokens)
            .await?;

        let mut full_output = String::new();
        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    buffer.push_str(&chunk);
                    while let Some(pos) = buffer.find('\n') {
                        let line = buffer[..pos].to_string();
                        buffer = buffer[pos + 1..].to_string();

                        if line.starts_with("data: ") {
                            let data = &line[6..];
                            if data != "[DONE]" {
                                if let Ok(chunk_data) = serde_json::from_str::<serde_json::Value>(data) {
                                    if let Some(content) = chunk_data.get("choices")
                                        .and_then(|c| c.get(0))
                                        .and_then(|c| c.get("delta"))
                                        .and_then(|d| d.get("content"))
                                        .and_then(|c| c.as_str())
                                    {
                                        full_output.push_str(content);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(module = "TaskExecutor", "Stream error: {}", e);
                }
            }
        }

        Ok(full_output)
    }

    pub async fn get_task_status(&self, task_id: &str) -> Option<TaskStatus> {
        let tasks = self.tasks.lock().await;
        tasks.get(task_id).map(|s| s.status.clone())
    }

    pub async fn get_task_result(&self, task_id: &str) -> Option<TaskResult> {
        let tasks = self.tasks.lock().await;
        tasks.get(task_id).and_then(|s| s.result.clone())
    }

    pub async fn cancel_task(&self, task_id: &str) -> bool {
        let mut tasks = self.tasks.lock().await;
        if let Some(state) = tasks.get_mut(task_id) {
            if state.status == TaskStatus::Running || state.status == TaskStatus::Pending {
                state.status = TaskStatus::Cancelled;
                return true;
            }
        }
        false
    }

    pub async fn list_tasks(&self) -> Vec<(String, TaskStatus)> {
        let tasks = self.tasks.lock().await;
        tasks.iter()
            .map(|(id, state)| (id.clone(), state.status.clone()))
            .collect()
    }

    pub async fn cleanup_completed(&self) {
        let mut tasks = self.tasks.lock().await;
        tasks.retain(|_, state| {
            state.status == TaskStatus::Running ||
            state.status == TaskStatus::Pending
        });
    }
}
