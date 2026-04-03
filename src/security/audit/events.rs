//! 审计事件定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use super::AuditLevel;

/// 审计事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// 事件ID
    pub id: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 审计级别
    pub level: AuditLevel,
    /// 事件类型
    pub event_type: AuditEventType,
    /// 用户ID
    pub user_id: String,
    /// 详情
    pub details: serde_json::Value,
}

impl AuditEvent {
    /// 创建新的审计事件
    pub fn new(level: AuditLevel, event_type: AuditEventType, user_id: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            level,
            event_type,
            user_id: user_id.to_string(),
            details: serde_json::json!({}),
        }
    }
    
    /// 创建工具调用事件
    pub fn tool_call(
        tool_name: &str,
        user_id: &str,
        input: &serde_json::Value,
        result: &serde_json::Value,
        duration_ms: u64,
    ) -> Self {
        let mut event = Self::new(
            AuditLevel::Info,
            AuditEventType::ToolCall,
            user_id,
        );
        
        event.details = serde_json::json!({
            "tool_name": tool_name,
            "input": input,
            "result": result,
            "duration_ms": duration_ms,
        });
        
        event
    }
    
    /// 创建权限决策事件
    pub fn permission_decision(
        resource_type: &str,
        resource_id: &str,
        operation: &str,
        user_id: &str,
        decision: &str,
        reason: Option<&str>,
    ) -> Self {
        let mut event = Self::new(
            AuditLevel::Info,
            AuditEventType::PermissionDecision,
            user_id,
        );
        
        let level = match decision {
            "allow" => AuditLevel::Info,
            "deny" => AuditLevel::Warning,
            "require_approval" => AuditLevel::Warning,
            _ => AuditLevel::Info,
        };
        
        event.level = level;
        event.details = serde_json::json!({
            "resource_type": resource_type,
            "resource_id": resource_id,
            "operation": operation,
            "decision": decision,
            "reason": reason,
        });
        
        event
    }
    
    /// 创建文件操作事件
    pub fn file_operation(
        operation: &str,
        path: &std::path::Path,
        user_id: &str,
        success: bool,
    ) -> Self {
        let level = if success { AuditLevel::Info } else { AuditLevel::Warning };
        
        let mut event = Self::new(
            level,
            AuditEventType::FileOperation,
            user_id,
        );
        
        event.details = serde_json::json!({
            "operation": operation,
            "path": path.to_string_lossy(),
            "success": success,
        });
        
        event
    }
    
    /// 创建网络请求事件
    pub fn network_request(
        url: &str,
        method: &str,
        user_id: &str,
        status_code: Option<u16>,
        response_size: Option<u64>,
    ) -> Self {
        let level = if status_code.map(|c| c >= 400).unwrap_or(false) {
            AuditLevel::Warning
        } else {
            AuditLevel::Info
        };
        
        let mut event = Self::new(
            level,
            AuditEventType::NetworkRequest,
            user_id,
        );
        
        event.details = serde_json::json!({
            "url": url,
            "method": method,
            "status_code": status_code,
            "response_size": response_size,
        });
        
        event
    }
    
    /// 创建危险命令事件
    pub fn dangerous_command(
        command: &str,
        user_id: &str,
        danger_level: &str,
    ) -> Self {
        let level = match danger_level {
            "critical" => AuditLevel::Critical,
            "high" => AuditLevel::Error,
            "medium" => AuditLevel::Warning,
            _ => AuditLevel::Info,
        };
        
        let mut event = Self::new(
            level,
            AuditEventType::DangerousCommand,
            user_id,
        );
        
        event.details = serde_json::json!({
            "command": command,
            "danger_level": danger_level,
        });
        
        event
    }
    
    /// 创建沙箱执行事件
    pub fn sandbox_execution(
        command: &str,
        sandboxed: bool,
        success: bool,
        reason: Option<&str>,
    ) -> Self {
        let level = if success { AuditLevel::Info } else { AuditLevel::Warning };
        
        let mut event = Self::new(
            level,
            AuditEventType::SandboxExecution,
            "system",
        );
        
        event.details = serde_json::json!({
            "command": command,
            "sandboxed": sandboxed,
            "success": success,
            "reason": reason,
        });
        
        event
    }
    
    /// 创建认证事件
    pub fn authentication(
        user_id: &str,
        success: bool,
        method: &str,
        source_ip: Option<&str>,
    ) -> Self {
        let level = if success { AuditLevel::Info } else { AuditLevel::Warning };
        
        let mut event = Self::new(
            level,
            AuditEventType::Authentication,
            user_id,
        );
        
        event.details = serde_json::json!({
            "success": success,
            "method": method,
            "source_ip": source_ip,
        });
        
        event
    }
}

/// 审计事件类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditEventType {
    /// 工具调用
    ToolCall,
    /// 权限决策
    PermissionDecision,
    /// 文件操作
    FileOperation,
    /// 网络请求
    NetworkRequest,
    /// 危险命令
    DangerousCommand,
    /// 沙箱执行
    SandboxExecution,
    /// 认证
    Authentication,
    /// 系统事件
    SystemEvent,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditEventType::ToolCall => write!(f, "TOOL_CALL"),
            AuditEventType::PermissionDecision => write!(f, "PERMISSION_DECISION"),
            AuditEventType::FileOperation => write!(f, "FILE_OPERATION"),
            AuditEventType::NetworkRequest => write!(f, "NETWORK_REQUEST"),
            AuditEventType::DangerousCommand => write!(f, "DANGEROUS_COMMAND"),
            AuditEventType::SandboxExecution => write!(f, "SANDBOX_EXECUTION"),
            AuditEventType::Authentication => write!(f, "AUTHENTICATION"),
            AuditEventType::SystemEvent => write!(f, "SYSTEM_EVENT"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tool_call_event() {
        let event = AuditEvent::tool_call(
            "Read",
            "user1",
            &serde_json::json!({"path": "/test"}),
            &serde_json::json!({"content": "test"}),
            100,
        );
        
        assert_eq!(event.event_type, AuditEventType::ToolCall);
        assert_eq!(event.user_id, "user1");
    }
    
    #[test]
    fn test_permission_decision_event() {
        let event = AuditEvent::permission_decision(
            "file",
            "/etc/passwd",
            "read",
            "user1",
            "deny",
            Some("Path is blacklisted"),
        );
        
        assert_eq!(event.event_type, AuditEventType::PermissionDecision);
        assert!(event.level >= AuditLevel::Warning);
    }
}
