//! 消息结构定义
//!
//! 定义查询引擎中使用的消息、工具调用和工具结果结构。

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 消息角色
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
    #[serde(rename = "tool")]
    Tool,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
            MessageRole::System => write!(f, "system"),
            MessageRole::Tool => write!(f, "tool"),
        }
    }
}

/// 消息内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    #[serde(rename = "text")]
    Text(String),
    #[serde(rename = "tool_calls")]
    ToolCalls(Vec<ToolCall>),
    #[serde(rename = "tool_result")]
    ToolResult(ToolResult),
}

impl MessageContent {
    /// 检查是否为文本内容
    pub fn is_text(&self) -> bool {
        matches!(self, MessageContent::Text(_))
    }

    /// 检查是否为工具调用
    pub fn is_tool_calls(&self) -> bool {
        matches!(self, MessageContent::ToolCalls(_))
    }

    /// 检查是否为工具结果
    pub fn is_tool_result(&self) -> bool {
        matches!(self, MessageContent::ToolResult(_))
    }

    /// 获取文本内容（如果是文本）
    pub fn text(&self) -> Option<&str> {
        match self {
            MessageContent::Text(text) => Some(text),
            _ => None,
        }
    }

    /// 获取工具调用列表（如果是工具调用）
    pub fn tool_calls(&self) -> Option<&[ToolCall]> {
        match self {
            MessageContent::ToolCalls(calls) => Some(calls),
            _ => None,
        }
    }

    /// 获取工具结果（如果是工具结果）
    pub fn tool_result(&self) -> Option<&ToolResult> {
        match self {
            MessageContent::ToolResult(result) => Some(result),
            _ => None,
        }
    }
}

/// 消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// 角色
    pub role: MessageRole,
    /// 内容
    pub content: MessageContent,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

impl Message {
    /// 创建用户消息
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: MessageContent::Text(text.into()),
            timestamp: Utc::now(),
        }
    }

    /// 创建助理消息
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: MessageContent::Text(text.into()),
            timestamp: Utc::now(),
        }
    }

    /// 创建系统消息
    pub fn system(text: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: MessageContent::Text(text.into()),
            timestamp: Utc::now(),
        }
    }

    /// 创建工具消息
    pub fn tool(result: ToolResult) -> Self {
        Self {
            role: MessageRole::Tool,
            content: MessageContent::ToolResult(result),
            timestamp: Utc::now(),
        }
    }

    /// 创建包含工具调用的助理消息
    pub fn assistant_with_tools(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: MessageContent::ToolCalls(tool_calls),
            timestamp: Utc::now(),
        }
    }

    /// 获取文本内容
    pub fn text_content(&self) -> Option<&str> {
        self.content.text()
    }
}

/// 工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// 工具调用 ID
    pub id: String,
    /// 工具名称
    pub name: String,
    /// 输入参数
    pub input: serde_json::Value,
}

impl ToolCall {
    /// 创建新的工具调用
    pub fn new(name: impl Into<String>, input: serde_json::Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            input,
        }
    }
}

/// 工具结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// 对应的工具调用 ID
    pub tool_use_id: String,
    /// 结果内容
    pub content: String,
    /// 是否为错误结果
    #[serde(default)]
    pub is_error: bool,
}

impl ToolResult {
    /// 创建成功结果
    pub fn success(tool_use_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: content.into(),
            is_error: false,
        }
    }

    /// 创建错误结果
    pub fn error(tool_use_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: content.into(),
            is_error: true,
        }
    }
}

/// 消息批次
#[derive(Debug, Clone, Default)]
pub struct MessageBatch {
    messages: Vec<Message>,
}

impl MessageBatch {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn add(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn extend(&mut self, messages: impl IntoIterator<Item = Message>) {
        self.messages.extend(messages);
    }
}