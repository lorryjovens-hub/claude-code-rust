//! 审计日志系统模块
//! 
//! 记录所有安全相关的操作和事件

pub mod logger;
pub mod events;

pub use logger::AuditLogger;
pub use events::*;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 审计日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// 是否启用审计日志
    pub enabled: bool,
    /// 日志目录
    pub log_dir: PathBuf,
    /// 最大日志文件大小（MB）
    pub max_file_size_mb: u64,
    /// 最大保留文件数
    pub max_files: u32,
    /// 是否记录工具调用
    pub log_tool_calls: bool,
    /// 是否记录权限决策
    pub log_permission_decisions: bool,
    /// 是否记录文件操作
    pub log_file_operations: bool,
    /// 是否记录网络请求
    pub log_network_requests: bool,
    /// 日志级别过滤
    pub min_level: AuditLevel,
}

impl Default for AuditConfig {
    fn default() -> Self {
        let log_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("claude-code")
            .join("audit-logs");
        
        Self {
            enabled: true,
            log_dir,
            max_file_size_mb: 10,
            max_files: 100,
            log_tool_calls: true,
            log_permission_decisions: true,
            log_file_operations: true,
            log_network_requests: true,
            min_level: AuditLevel::Info,
        }
    }
}

/// 审计级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AuditLevel {
    /// 调试
    Debug,
    /// 信息
    Info,
    /// 警告
    Warning,
    /// 错误
    Error,
    /// 严重
    Critical,
}

impl std::fmt::Display for AuditLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditLevel::Debug => write!(f, "DEBUG"),
            AuditLevel::Info => write!(f, "INFO"),
            AuditLevel::Warning => write!(f, "WARNING"),
            AuditLevel::Error => write!(f, "ERROR"),
            AuditLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}
