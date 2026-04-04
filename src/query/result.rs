//! 查询结果定义
//!
//! 定义查询结果、错误和状态。

use super::message::{Message, ToolCall};
use thiserror::Error;

/// 查询状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryStatus {
    /// 查询完成
    Completed,
    /// 查询进行中（流式）
    InProgress,
    /// 查询被用户取消
    Cancelled,
    /// 查询失败
    Failed,
    /// 查询超时
    TimedOut,
    /// 需要用户输入
    RequiresUserInput,
}

impl std::fmt::Display for QueryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryStatus::Completed => write!(f, "completed"),
            QueryStatus::InProgress => write!(f, "in_progress"),
            QueryStatus::Cancelled => write!(f, "cancelled"),
            QueryStatus::Failed => write!(f, "failed"),
            QueryStatus::TimedOut => write!(f, "timed_out"),
            QueryStatus::RequiresUserInput => write!(f, "requires_user_input"),
        }
    }
}

/// 查询结果
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// 助理响应消息
    pub response: Option<Message>,
    /// 执行的工具调用
    pub tool_calls: Vec<ToolCall>,
    /// 使用的 token 数
    pub tokens_used: u32,
    /// 查询持续时间（毫秒）
    pub duration_ms: u64,
    /// 查询状态
    pub status: QueryStatus,
}

impl QueryResult {
    /// 创建成功结果
    pub fn success(response: Message, tokens_used: u32, duration_ms: u64) -> Self {
        Self {
            response: Some(response),
            tool_calls: Vec::new(),
            tokens_used,
            duration_ms,
            status: QueryStatus::Completed,
        }
    }

    /// 创建工具调用结果
    pub fn tool_calls(tool_calls: Vec<ToolCall>, tokens_used: u32, duration_ms: u64) -> Self {
        Self {
            response: None,
            tool_calls,
            tokens_used,
            duration_ms,
            status: QueryStatus::InProgress,
        }
    }

    /// 创建错误结果
    pub fn error(error: QueryError, duration_ms: u64) -> Self {
        Self {
            response: Some(Message::assistant(format!("Error: {}", error))),
            tool_calls: Vec::new(),
            tokens_used: 0,
            duration_ms,
            status: QueryStatus::Failed,
        }
    }

    /// 检查是否成功
    pub fn is_success(&self) -> bool {
        matches!(self.status, QueryStatus::Completed)
    }

    /// 检查是否失败
    pub fn is_failure(&self) -> bool {
        matches!(self.status, QueryStatus::Failed | QueryStatus::TimedOut | QueryStatus::Cancelled)
    }

    /// 检查是否进行中
    pub fn is_in_progress(&self) -> bool {
        matches!(self.status, QueryStatus::InProgress)
    }

    /// 获取响应文本
    pub fn response_text(&self) -> Option<&str> {
        self.response.as_ref().and_then(|m| m.text_content())
    }
}

/// 查询错误
#[derive(Debug, Error)]
pub enum QueryError {
    /// API 错误
    #[error("API error: {0}")]
    Api(String),

    /// 网络错误
    #[error("Network error: {0}")]
    Network(String),

    /// 配置错误
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// 工具执行错误
    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    /// 权限错误
    #[error("Permission error: {0}")]
    Permission(String),

    /// 超时错误
    #[error("Query timeout after {0}ms")]
    Timeout(u64),

    /// 上下文太长
    #[error("Context too long: {0} tokens exceeds limit")]
    ContextTooLong(usize),

    /// 循环次数超过限制
    #[error("Query loop exceeded maximum iterations ({0})")]
    LoopExceeded(u32),

    /// 用户取消
    #[error("Query cancelled by user")]
    Cancelled,

    /// 输入无效
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// 其他错误
    #[error("{0}")]
    Other(String),
}

impl From<reqwest::Error> for QueryError {
    fn from(err: reqwest::Error) -> Self {
        QueryError::Network(err.to_string())
    }
}

impl From<serde_json::Error> for QueryError {
    fn from(err: serde_json::Error) -> Self {
        QueryError::Other(format!("JSON error: {}", err))
    }
}

impl From<std::io::Error> for QueryError {
    fn from(err: std::io::Error) -> Self {
        QueryError::Other(format!("IO error: {}", err))
    }
}

impl From<anyhow::Error> for QueryError {
    fn from(err: anyhow::Error) -> Self {
        QueryError::Other(err.to_string())
    }
}