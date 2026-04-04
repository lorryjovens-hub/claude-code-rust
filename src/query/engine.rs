//! 查询引擎核心实现
//!
//! 对应 TypeScript 的 QueryEngine.ts，实现主查询循环和工具调用。

use crate::config::{Config, Settings, SystemPromptBuilder};
use crate::state::AppState;
use crate::tools::ToolManager;
use crate::services::api::claude::ClaudeApi;
use crate::error::Result;

use super::context::QueryContext;
use super::message::{Message, MessageRole, MessageContent};
use super::result::{QueryResult, QueryError};
use super::pipeline::QueryPipeline;
use super::compressor::ContextCompressor;

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn, error};

/// 查询引擎配置
#[derive(Debug, Clone)]
pub struct QueryEngineConfig {
    /// 启用流式响应
    pub streaming: bool,
    /// 最大重试次数
    pub max_retries: u32,
    /// 启用工具调用
    pub enable_tools: bool,
    /// 启用上下文压缩
    pub enable_compression: bool,
    /// 最大上下文长度（token数）
    pub max_context_tokens: usize,
    /// 模型名称
    pub model: String,
}

impl Default for QueryEngineConfig {
    fn default() -> Self {
        Self {
            streaming: true,
            max_retries: 3,
            enable_tools: true,
            enable_compression: true,
            max_context_tokens: 128000, // Claude 上下文窗口
            model: "claude-3-5-sonnet-20241022".to_string(),
        }
    }
}

/// 查询引擎主结构
pub struct QueryEngine {
    /// 配置
    config: QueryEngineConfig,
    /// 应用设置
    settings: Settings,
    /// 应用状态
    state: AppState,
    /// 工具管理器
    tool_manager: Arc<ToolManager>,
    /// Claude API 客户端
    api_client: Arc<ClaudeApi>,
    /// 查询管道
    pipeline: Arc<QueryPipeline>,
    /// 上下文压缩器
    compressor: Arc<ContextCompressor>,
    /// 当前会话消息
    messages: Arc<RwLock<Vec<Message>>>,
}

impl QueryEngine {
    /// 创建新的查询引擎
    pub async fn new(
        settings: Settings,
        state: AppState,
        tool_manager: Arc<ToolManager>,
    ) -> Result<Self> {
        let config = QueryEngineConfig::default();

        // 创建 API 客户端
        let api_client = Arc::new(ClaudeApi::new()
            .with_model(settings.model.clone()));

        // 创建查询管道
        let pipeline = Arc::new(QueryPipeline::new());

        // 创建上下文压缩器
        let compressor = Arc::new(ContextCompressor::new(
            config.max_context_tokens,
        ));

        // 初始化消息列表
        let messages = Arc::new(RwLock::new(Vec::new()));

        Ok(Self {
            config,
            settings: settings.clone(),
            state,
            tool_manager,
            api_client,
            pipeline,
            compressor,
            messages,
        })
    }

    /// 提交消息进行处理（主入口）
    pub async fn submit_message(&self, user_input: &str) -> Result<QueryResult> {
        info!("Processing user message: {}", user_input);

        // 1. 创建查询上下文
        let context = QueryContext::new(
            self.settings.clone(),
            self.state.clone(),
            self.tool_manager.clone(),
        );

        // 2. 构建系统提示词
        let system_prompt = self.build_system_prompt(&context).await?;

        // 3. 准备用户消息
        let user_message = Message {
            role: MessageRole::User,
            content: MessageContent::Text(user_input.to_string()),
            timestamp: chrono::Utc::now(),
        };

        // 4. 添加到消息历史
        {
            let mut messages = self.messages.write().await;
            messages.push(user_message.clone());
        }

        // 5. 执行查询循环
        let result = self.query_loop(system_prompt, user_message, context).await?;

        // 6. 更新消息历史
        if let Some(response) = &result.response {
            let mut messages = self.messages.write().await;
            messages.push(response.clone());
        }

        info!("Query completed successfully");
        Ok(result)
    }

    /// 构建系统提示词
    async fn build_system_prompt(&self, context: &QueryContext) -> Result<String> {
        debug!("Building system prompt");

        let mut builder = SystemPromptBuilder::new(self.settings.clone());

        // 设置语言偏好
        builder.set_language(&self.settings.output.language);

        // 设置输出样式
        builder.set_output_style(&self.settings.output.style);

        // 设置简短模式
        builder.set_brief_mode(self.settings.output.brief_mode);

        // 添加环境信息
        self.add_environment_info(&mut builder);

        // 添加会话特定指南
        self.add_session_guidance(&mut builder).await;

        // 添加记忆
        self.add_memories(&mut builder).await;

        // 构建最终提示词
        let prompt = builder.build();

        debug!("System prompt length: {} chars", prompt.len());
        Ok(prompt)
    }

    /// 添加环境信息
    fn add_environment_info(&self, builder: &mut SystemPromptBuilder) {
        use std::env;
        use whoami;

        // 工作目录
        let cwd = std::env::current_dir()
            .unwrap_or_else(|_| ".".into());
        builder.set_env_info("Primary working directory", &cwd.to_string_lossy());

        // Git 仓库状态
        if cwd.join(".git").exists() {
            builder.set_env_info("Is a git repository", "true");
        }

        // 平台信息
        builder.set_env_info("Platform", &std::env::consts::OS);
        builder.set_env_info("Shell", &whoami::shell().to_string_lossy());
        builder.set_env_info("OS Version", &whoami::distro());

        // Claude Code 信息
        builder.set_env_info("Powered by", "Claude Opus 4.6");
        builder.set_env_info("Model ID", "claude-opus-4-6");
        builder.set_env_info("Assistant knowledge cutoff", "May 2025");
        builder.set_env_info("Claude Code availability",
            "CLI, desktop app (Mac/Windows), web app (claude.ai/code), IDE extensions");
    }

    /// 添加会话特定指南
    async fn add_session_guidance(&self, builder: &mut SystemPromptBuilder) {
        // TODO: 从状态或配置中获取会话特定指南
        builder.add_session_guidance(
            "If you need the user to run a shell command themselves, suggest they type `! <command>` in the prompt."
        );
        builder.add_session_guidance(
            "For simple, directed codebase searches use Glob or Grep directly."
        );
    }

    /// 添加记忆
    async fn add_memories(&self, builder: &mut SystemPromptBuilder) {
        // TODO: 从记忆系统中加载持久记忆
        // 暂时添加示例记忆
        builder.add_memory("User is working on a Rust project");
    }

    /// 主查询循环
    async fn query_loop(
        &self,
        system_prompt: String,
        user_message: Message,
        context: QueryContext,
    ) -> Result<QueryResult> {
        debug!("Starting query loop");

        let mut messages = vec![
            Message {
                role: MessageRole::System,
                content: MessageContent::Text(system_prompt),
                timestamp: chrono::Utc::now(),
            },
            user_message,
        ];

        let mut iteration = 0;
        let max_iterations = 10; // 防止无限循环

        loop {
            iteration += 1;
            if iteration > max_iterations {
                warn!("Query loop exceeded maximum iterations");
                break;
            }

            debug!("Query loop iteration {}", iteration);

            // 1. 应用上下文压缩
            if self.config.enable_compression {
                let compressed = self.compressor.compress(&messages).await?;
                if compressed.len() < messages.len() {
                    debug!("Compressed messages from {} to {}", messages.len(), compressed.len());
                    messages = compressed;
                }
            }

            // 2. 调用 API
            let api_response = self.call_api(&messages).await?;

            // 3. 处理响应
            match api_response {
                ApiResponse::Text(text) => {
                    // 纯文本响应，直接返回
                    let response_message = Message {
                        role: MessageRole::Assistant,
                        content: MessageContent::Text(text.clone()),
                        timestamp: chrono::Utc::now(),
                    };

                    return Ok(QueryResult {
                        response: Some(response_message),
                        tool_calls: Vec::new(),
                        tokens_used: 0,
                        duration_ms: 0,
                        status: super::result::QueryStatus::Completed,
                    });
                }
                ApiResponse::ToolCalls(tool_calls) => {
                    // 有工具调用需要执行
                    debug!("Executing {} tool calls", tool_calls.len());

                    let tool_results = self.execute_tools(tool_calls, &context).await?;

                    // 添加工具结果到消息历史
                    for result in &tool_results {
                        messages.push(Message {
                            role: MessageRole::Tool,
                            content: MessageContent::ToolResult(result.clone()),
                            timestamp: chrono::Utc::now(),
                        });
                    }

                    // 继续循环，让模型处理工具结果
                    continue;
                }
            }
        }

        // 如果循环结束但没有返回，返回错误
        Err(QueryError::LoopExceeded(max_iterations))
    }

    /// 调用 Claude API
    async fn call_api(&self, messages: &[Message]) -> Result<ApiResponse> {
        debug!("Calling Claude API");

        // TODO: 实现真正的 API 调用
        // 目前返回模拟响应

        // 模拟 API 延迟
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // 模拟文本响应
        Ok(ApiResponse::Text(
            "This is a simulated response from Claude. Tool execution not yet implemented.".to_string()
        ))
    }

    /// 执行工具调用
    async fn execute_tools(
        &self,
        tool_calls: Vec<ToolCall>,
        context: &QueryContext,
    ) -> Result<Vec<ToolResult>> {
        let mut results = Vec::new();

        for tool_call in tool_calls {
            debug!("Executing tool: {}", tool_call.name);

            // TODO: 实现真正的工具执行
            // 这里应该调用 tool_manager 执行工具

            let result = ToolResult {
                tool_use_id: tool_call.id.clone(),
                content: format!("Tool '{}' executed (simulated)", tool_call.name),
                is_error: false,
            };

            results.push(result);
        }

        Ok(results)
    }
}

/// API 响应类型
enum ApiResponse {
    Text(String),
    ToolCalls(Vec<ToolCall>),
}

/// 查询引擎构建器
pub struct QueryEngineBuilder {
    config: QueryEngineConfig,
    settings: Option<Settings>,
    state: Option<AppState>,
    tool_manager: Option<Arc<ToolManager>>,
}

impl QueryEngineBuilder {
    pub fn new() -> Self {
        Self {
            config: QueryEngineConfig::default(),
            settings: None,
            state: None,
            tool_manager: None,
        }
    }

    pub fn with_config(mut self, config: QueryEngineConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_settings(mut self, settings: Settings) -> Self {
        self.settings = Some(settings);
        self
    }

    pub fn with_state(mut self, state: AppState) -> Self {
        self.state = Some(state);
        self
    }

    pub fn with_tool_manager(mut self, tool_manager: Arc<ToolManager>) -> Self {
        self.tool_manager = Some(tool_manager);
        self
    }

    pub async fn build(self) -> Result<QueryEngine> {
        let settings = self.settings.ok_or_else(||
            QueryError::Configuration("Settings missing".to_string()))?;
        let state = self.state.ok_or_else(||
            QueryError::Configuration("AppState missing".to_string()))?;
        let tool_manager = self.tool_manager.ok_or_else(||
            QueryError::Configuration("ToolManager missing".to_string()))?;

        QueryEngine::new(settings, state, tool_manager).await
    }
}

impl Default for QueryEngine {
    async fn default() -> Self {
        let settings = Settings::default();
        let state = AppState::default();
        let tool_manager = Arc::new(crate::tools::init().await.expect("Failed to init tools"));

        QueryEngine::new(settings, state, tool_manager).await
            .expect("Failed to create QueryEngine")
    }
}