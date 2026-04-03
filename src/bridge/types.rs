//! Bridge 系统类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 默认会话超时时间（24小时）
pub const DEFAULT_SESSION_TIMEOUT_MS: u64 = 24 * 60 * 60 * 1000;

/// 会话模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpawnMode {
    /// 单会话模式：完全隔离的执行环境，适用于临时任务
    SingleSession,
    /// Git 工作树模式：通过 Git 工作树实现环境隔离，适用于并行开发场景
    Worktree,
    /// 共享目录模式：无隔离的高效执行环境，适用于快速迭代开发
    SameDir,
}

impl Default for SpawnMode {
    fn default() -> Self {
        SpawnMode::SingleSession
    }
}

impl std::fmt::Display for SpawnMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpawnMode::SingleSession => write!(f, "single-session"),
            SpawnMode::Worktree => write!(f, "worktree"),
            SpawnMode::SameDir => write!(f, "same-dir"),
        }
    }
}

/// Bridge 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    /// 工作目录
    pub dir: String,
    /// 机器名称
    pub machine_name: String,
    /// 分支名称
    pub branch: String,
    /// Git 仓库 URL
    pub git_repo_url: Option<String>,
    /// 最大会话数
    pub max_sessions: usize,
    /// 会话模式
    pub spawn_mode: SpawnMode,
    /// 是否启用详细日志
    pub verbose: bool,
    /// 是否启用沙箱
    pub sandbox: bool,
    /// Bridge ID（客户端生成的 UUID）
    pub bridge_id: String,
    /// 工作器类型
    pub worker_type: String,
    /// 环境 ID（客户端生成的 UUID）
    pub environment_id: String,
    /// 重用的环境 ID（用于恢复会话）
    pub reuse_environment_id: Option<String>,
    /// API 基础 URL
    pub api_base_url: String,
    /// 会话入口 URL
    pub session_ingress_url: String,
    /// 调试文件路径
    pub debug_file: Option<String>,
    /// 会话超时时间（毫秒）
    pub session_timeout_ms: Option<u64>,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            dir: ".".to_string(),
            machine_name: "unknown".to_string(),
            branch: "main".to_string(),
            git_repo_url: None,
            max_sessions: 1,
            spawn_mode: SpawnMode::default(),
            verbose: false,
            sandbox: false,
            bridge_id: uuid::Uuid::new_v4().to_string(),
            worker_type: "claude_code".to_string(),
            environment_id: uuid::Uuid::new_v4().to_string(),
            reuse_environment_id: None,
            api_base_url: "https://api.anthropic.com".to_string(),
            session_ingress_url: "https://session.anthropic.com".to_string(),
            debug_file: None,
            session_timeout_ms: Some(DEFAULT_SESSION_TIMEOUT_MS),
        }
    }
}

/// 工作数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkData {
    /// 工作类型
    #[serde(rename = "type")]
    pub work_type: WorkType,
    /// 工作ID
    pub id: String,
}

/// 工作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkType {
    #[serde(rename = "session")]
    Session,
    #[serde(rename = "healthcheck")]
    HealthCheck,
}

/// 工作响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkResponse {
    /// 工作ID
    pub id: String,
    /// 响应类型
    #[serde(rename = "type")]
    pub response_type: String,
    /// 环境ID
    pub environment_id: String,
    /// 状态
    pub state: String,
    /// 工作数据
    pub data: WorkData,
    /// 密钥（base64url 编码的 JSON）
    pub secret: String,
    /// 创建时间
    pub created_at: String,
}

/// 工作密钥
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSecret {
    /// 版本
    pub version: u32,
    /// 会话入口令牌
    pub session_ingress_token: String,
    /// API 基础 URL
    pub api_base_url: String,
    /// 源列表
    pub sources: Vec<WorkSource>,
    /// 认证信息
    pub auth: Vec<AuthInfo>,
    /// Claude Code 参数
    pub claude_code_args: Option<HashMap<String, String>>,
    /// MCP 配置
    pub mcp_config: Option<serde_json::Value>,
    /// 环境变量
    pub environment_variables: Option<HashMap<String, String>>,
    /// 是否使用代码会话
    pub use_code_sessions: Option<bool>,
}

/// 工作源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSource {
    /// 源类型
    #[serde(rename = "type")]
    pub source_type: String,
    /// Git 信息
    pub git_info: Option<GitInfo>,
}

/// Git 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    /// Git 类型
    #[serde(rename = "type")]
    pub git_type: String,
    /// 仓库名称
    pub repo: String,
    /// 引用
    pub r#ref: Option<String>,
    /// 令牌
    pub token: Option<String>,
}

/// 认证信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthInfo {
    /// 认证类型
    #[serde(rename = "type")]
    pub auth_type: String,
    /// 令牌
    pub token: String,
}

/// 会话完成状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionDoneStatus {
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "interrupted")]
    Interrupted,
}

/// 会话活动类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionActivityType {
    #[serde(rename = "tool_start")]
    ToolStart,
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "result")]
    Result,
    #[serde(rename = "error")]
    Error,
}

/// 会话活动
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionActivity {
    /// 活动类型
    #[serde(rename = "type")]
    pub activity_type: SessionActivityType,
    /// 摘要
    pub summary: String,
    /// 时间戳
    pub timestamp: u64,
}

/// 权限响应事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResponseEvent {
    /// 事件类型
    #[serde(rename = "type")]
    pub event_type: String,
    /// 响应
    pub response: PermissionResponse,
}

/// 权限响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResponse {
    /// 子类型
    pub subtype: String,
    /// 请求ID
    pub request_id: String,
    /// 响应数据
    pub response: HashMap<String, serde_json::Value>,
}

/// Bridge 工作器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeWorkerType {
    #[serde(rename = "claude_code")]
    ClaudeCode,
    #[serde(rename = "claude_code_assistant")]
    ClaudeCodeAssistant,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spawn_mode() {
        assert_eq!(SpawnMode::default(), SpawnMode::SingleSession);
        assert_eq!(SpawnMode::SingleSession.to_string(), "single-session");
        assert_eq!(SpawnMode::Worktree.to_string(), "worktree");
        assert_eq!(SpawnMode::SameDir.to_string(), "same-dir");
    }
    
    #[test]
    fn test_bridge_config_default() {
        let config = BridgeConfig::default();
        assert_eq!(config.max_sessions, 1);
        assert_eq!(config.spawn_mode, SpawnMode::SingleSession);
        assert!(!config.sandbox);
    }
}
