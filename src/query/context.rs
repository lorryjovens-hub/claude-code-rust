//! 查询上下文
//!
//! 包含查询执行所需的所有上下文信息。

use crate::config::Settings;
use crate::state::AppState;
use crate::tools::ToolManager;
use std::sync::Arc;

/// 查询上下文
#[derive(Debug, Clone)]
pub struct QueryContext {
    /// 配置设置
    pub settings: Settings,
    /// 应用状态
    pub state: AppState,
    /// 工具管理器
    pub tool_manager: Arc<ToolManager>,
    /// 当前工作目录
    pub working_dir: std::path::PathBuf,
    /// 查询开始时间
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// 查询选项
    pub options: QueryOptions,
}

/// 查询选项
#[derive(Debug, Clone, Default)]
pub struct QueryOptions {
    /// 启用工具调用
    pub enable_tools: bool,
    /// 启用流式响应
    pub streaming: bool,
    /// 最大重试次数
    pub max_retries: u32,
    /// 超时时间（毫秒）
    pub timeout_ms: Option<u64>,
    /// 模型覆盖
    pub model_override: Option<String>,
    /// 温度设置
    pub temperature: Option<f32>,
    /// 最大 token 数
    pub max_tokens: Option<u32>,
}

impl QueryContext {
    /// 创建新的查询上下文
    pub fn new(
        settings: Settings,
        state: AppState,
        tool_manager: Arc<ToolManager>,
    ) -> Self {
        let working_dir = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."));

        Self {
            settings,
            state,
            tool_manager,
            working_dir,
            start_time: chrono::Utc::now(),
            options: QueryOptions::default(),
        }
    }

    /// 设置查询选项
    pub fn with_options(mut self, options: QueryOptions) -> Self {
        self.options = options;
        self
    }

    /// 获取经过的时间（毫秒）
    pub fn elapsed_ms(&self) -> i64 {
        let now = chrono::Utc::now();
        (now - self.start_time).num_milliseconds()
    }

    /// 检查是否超时
    pub fn is_timed_out(&self) -> bool {
        if let Some(timeout) = self.options.timeout_ms {
            self.elapsed_ms() > timeout as i64
        } else {
            false
        }
    }

    /// 获取有效模型名称
    pub fn effective_model(&self) -> String {
        self.options.model_override
            .clone()
            .unwrap_or_else(|| self.settings.model.clone())
    }

    /// 获取工具是否启用
    pub fn tools_enabled(&self) -> bool {
        self.options.enable_tools
    }

    /// 获取流式是否启用
    pub fn streaming_enabled(&self) -> bool {
        self.options.streaming
    }
}

impl QueryOptions {
    /// 创建默认查询选项
    pub fn default() -> Self {
        Self {
            enable_tools: true,
            streaming: true,
            max_retries: 3,
            timeout_ms: Some(30000), // 30秒
            model_override: None,
            temperature: None,
            max_tokens: None,
        }
    }

    /// 启用工具调用
    pub fn enable_tools(mut self, enable: bool) -> Self {
        self.enable_tools = enable;
        self
    }

    /// 启用流式响应
    pub fn streaming(mut self, enable: bool) -> Self {
        self.streaming = enable;
        self
    }

    /// 设置最大重试次数
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// 设置超时时间
    pub fn timeout_ms(mut self, timeout: Option<u64>) -> Self {
        self.timeout_ms = timeout;
        self
    }

    /// 设置模型覆盖
    pub fn model_override(mut self, model: Option<String>) -> Self {
        self.model_override = model;
        self
    }

    /// 设置温度
    pub fn temperature(mut self, temperature: Option<f32>) -> Self {
        self.temperature = temperature;
        self
    }

    /// 设置最大 token 数
    pub fn max_tokens(mut self, max_tokens: Option<u32>) -> Self {
        self.max_tokens = max_tokens;
        self
    }
}