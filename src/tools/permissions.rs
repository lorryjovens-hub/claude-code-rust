//! 工具权限系统
//! 
//! 这个模块实现了工具权限检查机制

use crate::error::Result;
use super::types::{
    PermissionResult, PermissionBehavior, ToolPermissionContext, 
    ToolPermissionRule, ToolPermissionRulesBySource,
};

/// 权限检查器
pub struct PermissionChecker;

impl PermissionChecker {
    /// 检查工具使用权限
    pub fn check(
        tool_name: &str,
        input: &serde_json::Value,
        context: &ToolPermissionContext,
    ) -> PermissionResult {
        // 1. 检查 alwaysAllowRules
        if Self::check_allow_rules(tool_name, input, &context.always_allow_rules) {
            return PermissionResult::allow();
        }
        
        // 2. 检查 alwaysDenyRules
        if Self::check_deny_rules(tool_name, input, &context.always_deny_rules) {
            return PermissionResult::deny(format!("Tool {} is denied by rule", tool_name));
        }
        
        // 3. 检查 alwaysAskRules
        if Self::check_ask_rules(tool_name, input, &context.always_ask_rules) {
            return PermissionResult::ask();
        }
        
        // 4. 根据模式决定
        match context.mode {
            super::PermissionMode::Bypass => PermissionResult::allow(),
            super::PermissionMode::Plan => PermissionResult::allow(),
            super::PermissionMode::Default => PermissionResult::allow(),
        }
    }
    
    /// 检查允许规则
    fn check_allow_rules(
        tool_name: &str,
        _input: &serde_json::Value,
        rules: &ToolPermissionRulesBySource,
    ) -> bool {
        Self::check_rules(tool_name, rules, true)
    }
    
    /// 检查拒绝规则
    fn check_deny_rules(
        tool_name: &str,
        _input: &serde_json::Value,
        rules: &ToolPermissionRulesBySource,
    ) -> bool {
        Self::check_rules(tool_name, rules, true)
    }
    
    /// 检查询问规则
    fn check_ask_rules(
        tool_name: &str,
        _input: &serde_json::Value,
        rules: &ToolPermissionRulesBySource,
    ) -> bool {
        Self::check_rules(tool_name, rules, true)
    }
    
    /// 检查规则匹配
    fn check_rules(tool_name: &str, rules: &ToolPermissionRulesBySource, _content_match: bool) -> bool {
        for (_source, tool_rules) in rules {
            for rule in tool_rules {
                // 检查工具名称匹配
                if Self::matches_tool_name(&rule.name, tool_name) {
                    // 如果规则有内容要求，需要进一步检查
                    if let Some(ref content) = rule.content {
                        if content.is_empty() {
                            return true; // 空内容表示完全匹配
                        }
                        // TODO: 匹配规则内容
                    } else {
                        return true; // 无内容要求表示完全允许/拒绝
                    }
                }
                
                // 支持通配符匹配
                if rule.name.contains('*') {
                    let pattern = rule.name.replace('*', ".*");
                    if let Ok(regex) = regex::Regex::new(&pattern) {
                        if regex.is_match(tool_name) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
    
    /// 检查工具名称是否匹配
    fn matches_tool_name(rule_name: &str, tool_name: &str) -> bool {
        // 完全匹配
        if rule_name == tool_name {
            return true;
        }
        
        // 支持 MCP 格式: mcp__server__tool
        if rule_name.starts_with("mcp__") {
            let rule_parts: Vec<&str> = rule_name.split("__").collect();
            let tool_parts: Vec<&str> = tool_name.split("__").collect();
            
            // 如果规则是 mcp__server，匹配该服务器的所有工具
            if rule_parts.len() == 2 && tool_parts.len() >= 2 {
                return rule_parts[1] == tool_parts[1];
            }
            
            // 如果规则是 mcp__server__tool，完全匹配
            if rule_parts.len() >= 3 && tool_parts.len() >= 3 {
                return rule_parts[1] == tool_parts[1] && rule_parts[2] == tool_parts[2];
            }
        }
        
        false
    }
    
    /// 添加允许规则
    pub fn add_allow_rule(
        context: &mut ToolPermissionContext,
        source: impl Into<String>,
        rule: ToolPermissionRule,
    ) {
        let source = source.into();
        context.always_allow_rules
            .entry(source)
            .or_insert_with(Vec::new)
            .push(rule);
    }
    
    /// 添加拒绝规则
    pub fn add_deny_rule(
        context: &mut ToolPermissionContext,
        source: impl Into<String>,
        rule: ToolPermissionRule,
    ) {
        let source = source.into();
        context.always_deny_rules
            .entry(source)
            .or_insert_with(Vec::new)
            .push(rule);
    }
    
    /// 添加询问规则
    pub fn add_ask_rule(
        context: &mut ToolPermissionContext,
        source: impl Into<String>,
        rule: ToolPermissionRule,
    ) {
        let source = source.into();
        context.always_ask_rules
            .entry(source)
            .or_insert_with(Vec::new)
            .push(rule);
    }
    
    /// 创建简单的允许规则
    pub fn allow_tool(tool_name: impl Into<String>) -> ToolPermissionRule {
        ToolPermissionRule {
            name: tool_name.into(),
            content: None,
        }
    }
    
    /// 创建带模式的允许规则
    pub fn allow_tool_pattern(tool_name: impl Into<String>, pattern: impl Into<String>) -> ToolPermissionRule {
        ToolPermissionRule {
            name: tool_name.into(),
            content: Some(pattern.into()),
        }
    }
    
    /// 创建拒绝规则
    pub fn deny_tool(tool_name: impl Into<String>) -> ToolPermissionRule {
        ToolPermissionRule {
            name: tool_name.into(),
            content: None,
        }
    }
}

/// 权限模式检查器
pub struct ModeChecker;

impl ModeChecker {
    /// 检查是否可以绕过权限
    pub fn can_bypass(context: &ToolPermissionContext) -> bool {
        context.is_bypass_permissions_mode_available
    }
    
    /// 检查是否应该自动允许
    pub fn should_auto_allow(context: &ToolPermissionContext) -> bool {
        matches!(context.mode, super::PermissionMode::Bypass | super::PermissionMode::Plan)
    }
    
    /// 检查是否应该询问用户
    pub fn should_ask(context: &ToolPermissionContext) -> bool {
        matches!(context.mode, super::PermissionMode::Default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::types::{PermissionMode, PermissionBehavior};
    
    #[test]
    fn test_permission_checker_allow() {
        let context = ToolPermissionContext::empty();
        
        let result = PermissionChecker::check("Read", &serde_json::json!({}), &context);
        assert_eq!(result.behavior, PermissionBehavior::Allow);
    }
    
    #[test]
    fn test_permission_checker_deny() {
        let mut context = ToolPermissionContext::empty();
        context.mode = PermissionMode::Default;
        
        let result = PermissionChecker::check("Bash", &serde_json::json!({"command": "rm -rf /"}), &context);
        assert_eq!(result.behavior, PermissionBehavior::Deny);
    }
    
    #[test]
    fn test_permission_checker_mcp_tool() {
        let context = ToolPermissionContext::empty();
        
        let result = PermissionChecker::check("mcp__server__tool", &serde_json::json!({}), &context);
        assert_eq!(result.behavior, PermissionBehavior::Deny);
    }
    
    #[test]
    fn test_mode_checker() {
        let mut context = ToolPermissionContext::empty();
        context.is_bypass_permissions_mode_available = true;
        
        assert!(ModeChecker::can_bypass(&context));
        
        context.mode = PermissionMode::Bypass;
        assert!(ModeChecker::should_auto_allow(&context));
        
        context.mode = PermissionMode::Default;
        assert!(ModeChecker::should_ask(&context));
    }
}
