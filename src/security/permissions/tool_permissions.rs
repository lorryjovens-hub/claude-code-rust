//! 工具权限管理器

use super::{PermissionContext, PermissionDecision, PermissionRule, ApprovalLevel};
use crate::error::Result;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// 工具权限管理器
/// 
/// 管理工具访问权限，支持基于角色和用户的权限分配
#[derive(Debug)]
pub struct ToolPermissionManager {
    /// 工具权限规则
    rules: RwLock<HashMap<String, PermissionRule>>,
    /// 工具分类
    tool_categories: RwLock<HashMap<String, String>>,
    /// 危险工具列表
    dangerous_tools: RwLock<Vec<String>>,
    /// 需要审批的工具
    approval_required_tools: RwLock<Vec<String>>,
}

impl ToolPermissionManager {
    /// 创建新的工具权限管理器
    pub fn new() -> Self {
        Self {
            rules: RwLock::new(HashMap::new()),
            tool_categories: RwLock::new(HashMap::new()),
            dangerous_tools: RwLock::new(Vec::new()),
            approval_required_tools: RwLock::new(Vec::new()),
        }
    }
    
    /// 加载默认策略
    pub async fn load_default_policies(&mut self) -> Result<()> {
        let mut rules = self.rules.write().await;
        
        rules.insert("read_allow".to_string(), PermissionRule {
            id: "read_allow".to_string(),
            name: "Read Tool Access".to_string(),
            description: "Allow read tool access".to_string(),
            resource_pattern: "Read".to_string(),
            allowed_operations: vec!["execute".to_string()],
            denied_operations: vec![],
            priority: 100,
            enabled: true,
        });
        
        rules.insert("write_allow".to_string(), PermissionRule {
            id: "write_allow".to_string(),
            name: "Write Tool Access".to_string(),
            description: "Allow write tool access".to_string(),
            resource_pattern: "Write".to_string(),
            allowed_operations: vec!["execute".to_string()],
            denied_operations: vec![],
            priority: 100,
            enabled: true,
        });
        
        rules.insert("bash_dangerous".to_string(), PermissionRule {
            id: "bash_dangerous".to_string(),
            name: "Bash Tool Restriction".to_string(),
            description: "Bash tool requires approval".to_string(),
            resource_pattern: "Bash".to_string(),
            allowed_operations: vec![],
            denied_operations: vec!["execute".to_string()],
            priority: 200,
            enabled: true,
        });
        
        let mut categories = self.tool_categories.write().await;
        categories.insert("Read".to_string(), "file".to_string());
        categories.insert("Write".to_string(), "file".to_string());
        categories.insert("Edit".to_string(), "file".to_string());
        categories.insert("Bash".to_string(), "command".to_string());
        categories.insert("Glob".to_string(), "search".to_string());
        categories.insert("Grep".to_string(), "search".to_string());
        
        let mut dangerous = self.dangerous_tools.write().await;
        *dangerous = vec![
            "Bash".to_string(),
            "ExecuteCommand".to_string(),
            "DeleteFile".to_string(),
        ];
        
        let mut approval_required = self.approval_required_tools.write().await;
        *approval_required = vec![
            "Bash".to_string(),
            "ExecuteCommand".to_string(),
        ];
        
        Ok(())
    }
    
    /// 检查工具权限
    pub async fn check_permission(
        &self,
        tool_name: &str,
        user_id: &str,
        context: &PermissionContext,
    ) -> Result<PermissionDecision> {
        let rules = self.rules.read().await;
        let dangerous = self.dangerous_tools.read().await;
        let approval_required = self.approval_required_tools.read().await;
        
        let mut matched_rules: Vec<&PermissionRule> = rules
            .values()
            .filter(|rule| {
                rule.enabled && Self::matches_pattern(&rule.resource_pattern, tool_name)
            })
            .collect();
        
        matched_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        for rule in matched_rules {
            if rule.denied_operations.contains(&"execute".to_string()) {
                if dangerous.contains(&tool_name.to_string()) {
                    return Ok(PermissionDecision::RequireApproval {
                        level: ApprovalLevel::Single,
                        reason: format!("Tool '{}' is classified as dangerous", tool_name),
                    });
                }
                return Ok(PermissionDecision::Deny(format!(
                    "Tool '{}' is denied by rule '{}'",
                    tool_name, rule.name
                )));
            }
            
            if rule.allowed_operations.contains(&"execute".to_string()) {
                return Ok(PermissionDecision::Allow);
            }
        }
        
        if approval_required.contains(&tool_name.to_string()) {
            return Ok(PermissionDecision::RequireApproval {
                level: ApprovalLevel::Single,
                reason: format!("Tool '{}' requires approval", tool_name),
            });
        }
        
        if context.roles.contains(&"admin".to_string()) {
            return Ok(PermissionDecision::Allow);
        }
        
        Ok(PermissionDecision::Allow)
    }
    
    /// 添加权限规则
    pub async fn add_rule(&self, rule: PermissionRule) -> Result<()> {
        let mut rules = self.rules.write().await;
        rules.insert(rule.id.clone(), rule);
        Ok(())
    }
    
    /// 移除权限规则
    pub async fn remove_rule(&self, rule_id: &str) -> Result<()> {
        let mut rules = self.rules.write().await;
        rules.remove(rule_id);
        Ok(())
    }
    
    /// 标记工具为危险
    pub async fn mark_dangerous(&self, tool_name: String) -> Result<()> {
        let mut dangerous = self.dangerous_tools.write().await;
        if !dangerous.contains(&tool_name) {
            dangerous.push(tool_name);
        }
        Ok(())
    }
    
    /// 设置工具需要审批
    pub async fn set_approval_required(&self, tool_name: String) -> Result<()> {
        let mut approval_required = self.approval_required_tools.write().await;
        if !approval_required.contains(&tool_name) {
            approval_required.push(tool_name);
        }
        Ok(())
    }
    
    /// 匹配模式
    fn matches_pattern(pattern: &str, tool_name: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        
        if pattern == tool_name {
            return true;
        }
        
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return tool_name.starts_with(prefix) && tool_name.ends_with(suffix);
            }
        }
        
        if pattern.starts_with("mcp__") {
            let pattern_parts: Vec<&str> = pattern.split("__").collect();
            let tool_parts: Vec<&str> = tool_name.split("__").collect();
            
            if pattern_parts.len() == 2 && tool_parts.len() >= 2 {
                return pattern_parts[1] == tool_parts[1];
            }
            
            if pattern_parts.len() >= 3 && tool_parts.len() >= 3 {
                return pattern_parts[1] == tool_parts[1] && pattern_parts[2] == tool_parts[2];
            }
        }
        
        false
    }
}

impl Default for ToolPermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tool_permission_manager() {
        let mut manager = ToolPermissionManager::new();
        manager.load_default_policies().await.unwrap();
        
        let context = PermissionContext::new("user1");
        let decision = manager.check_permission("Read", "user1", &context).await.unwrap();
        assert_eq!(decision, PermissionDecision::Allow);
    }
    
    #[tokio::test]
    async fn test_dangerous_tool() {
        let mut manager = ToolPermissionManager::new();
        manager.load_default_policies().await.unwrap();
        
        let context = PermissionContext::new("user1");
        let decision = manager.check_permission("Bash", "user1", &context).await.unwrap();
        
        match decision {
            PermissionDecision::RequireApproval { .. } => (),
            _ => panic!("Expected RequireApproval for dangerous tool"),
        }
    }
    
    #[test]
    fn test_pattern_matching() {
        assert!(ToolPermissionManager::matches_pattern("Read", "Read"));
        assert!(ToolPermissionManager::matches_pattern("*", "AnyTool"));
        assert!(ToolPermissionManager::matches_pattern("Read*", "ReadFile"));
        assert!(ToolPermissionManager::matches_pattern("*File", "ReadFile"));
    }
}
