//! 权限系统类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 权限上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionContext {
    /// 用户ID
    pub user_id: String,
    /// 角色列表
    pub roles: Vec<String>,
    /// 会话ID
    pub session_id: Option<String>,
    /// 工作目录
    pub working_directory: Option<String>,
    /// 额外上下文
    pub extra: HashMap<String, String>,
}

impl PermissionContext {
    /// 创建新的权限上下文
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            roles: Vec::new(),
            session_id: None,
            working_directory: None,
            extra: HashMap::new(),
        }
    }
    
    /// 添加角色
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }
    
    /// 设置会话ID
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }
    
    /// 设置工作目录
    pub fn with_working_directory(mut self, dir: impl Into<String>) -> Self {
        self.working_directory = Some(dir.into());
        self
    }
}

/// 文件操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileOperation {
    /// 读取
    Read,
    /// 写入
    Write,
    /// 执行
    Execute,
    /// 删除
    Delete,
    /// 列出目录
    List,
}

impl std::fmt::Display for FileOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileOperation::Read => write!(f, "read"),
            FileOperation::Write => write!(f, "write"),
            FileOperation::Execute => write!(f, "execute"),
            FileOperation::Delete => write!(f, "delete"),
            FileOperation::List => write!(f, "list"),
        }
    }
}

/// 权限规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    /// 规则ID
    pub id: String,
    /// 规则名称
    pub name: String,
    /// 规则描述
    pub description: String,
    /// 资源模式（支持通配符）
    pub resource_pattern: String,
    /// 允许的操作
    pub allowed_operations: Vec<String>,
    /// 拒绝的操作
    pub denied_operations: Vec<String>,
    /// 优先级（数字越大优先级越高）
    pub priority: u32,
    /// 是否启用
    pub enabled: bool,
}

/// 角色定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// 角色ID
    pub id: String,
    /// 角色名称
    pub name: String,
    /// 角色描述
    pub description: String,
    /// 权限列表
    pub permissions: Vec<String>,
    /// 继承的角色
    pub inherits: Vec<String>,
    /// 是否为系统角色
    pub is_system: bool,
}

/// 用户权限配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissionConfig {
    /// 用户ID
    pub user_id: String,
    /// 直接分配的角色
    pub roles: Vec<String>,
    /// 直接分配的权限
    pub permissions: Vec<String>,
    /// 自定义规则
    pub custom_rules: Vec<PermissionRule>,
}

/// 权限策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPolicy {
    /// 策略ID
    pub id: String,
    /// 策略名称
    pub name: String,
    /// 策略描述
    pub description: String,
    /// 规则列表
    pub rules: Vec<PermissionRule>,
    /// 默认决策
    pub default_decision: String,
}

/// 审批请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// 请求ID
    pub id: String,
    /// 请求者
    pub requester: String,
    /// 资源类型
    pub resource_type: String,
    /// 资源标识
    pub resource_id: String,
    /// 操作
    pub operation: String,
    /// 原因
    pub reason: String,
    /// 审批级别
    pub approval_level: u32,
    /// 当前审批状态
    pub approvals: Vec<Approval>,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 过期时间
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 审批记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approval {
    /// 审批人
    pub approver: String,
    /// 审批时间
    pub approved_at: chrono::DateTime<chrono::Utc>,
    /// 审批结果
    pub approved: bool,
    /// 审批意见
    pub comment: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_permission_context() {
        let ctx = PermissionContext::new("user1")
            .with_role("admin")
            .with_session("session1")
            .with_working_directory("/home/user");
        
        assert_eq!(ctx.user_id, "user1");
        assert_eq!(ctx.roles, vec!["admin"]);
        assert_eq!(ctx.session_id, Some("session1".to_string()));
    }
    
    #[test]
    fn test_file_operation() {
        assert_eq!(FileOperation::Read.to_string(), "read");
        assert_eq!(FileOperation::Write.to_string(), "write");
    }
}
