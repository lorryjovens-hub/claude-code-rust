//! 环境变量安全检查

use crate::error::Result;
use std::collections::HashMap;

/// 环境检查结果
#[derive(Debug, Clone)]
pub struct EnvironmentCheckResult {
    /// 是否安全
    pub safe: bool,
    /// 原因
    pub reason: String,
    /// 发现的问题
    pub issues: Vec<String>,
}

/// 环境检查器
/// 
/// 检查环境变量的安全性
#[derive(Debug, Clone)]
pub struct EnvironmentChecker {
    /// 允许的环境变量
    allowed_vars: Vec<String>,
    /// 禁止的环境变量
    blocked_vars: Vec<String>,
    /// 敏感信息模式
    sensitive_patterns: Vec<String>,
}

impl EnvironmentChecker {
    /// 创建新的环境检查器
    pub fn new(allowed_vars: Vec<String>, blocked_vars: Vec<String>) -> Self {
        Self {
            allowed_vars,
            blocked_vars,
            sensitive_patterns: vec![
                "password".to_string(),
                "secret".to_string(),
                "api_key".to_string(),
                "apikey".to_string(),
                "token".to_string(),
                "auth".to_string(),
                "credential".to_string(),
                "private_key".to_string(),
                "privatekey".to_string(),
            ],
        }
    }
    
    /// 检查环境
    pub fn check_environment(&self) -> EnvironmentCheckResult {
        let mut issues = Vec::new();
        
        for (key, value) in std::env::vars() {
            if self.is_blocked_var(&key) {
                issues.push(format!("Blocked environment variable found: {}", key));
            }
            
            if self.contains_sensitive_data(&key, &value) {
                issues.push(format!("Sensitive data detected in variable: {}", key));
            }
        }
        
        let safe = issues.is_empty();
        let reason = if safe {
            "Environment is safe".to_string()
        } else {
            format!("Found {} security issues", issues.len())
        };
        
        EnvironmentCheckResult {
            safe,
            reason,
            issues,
        }
    }
    
    /// 检查特定变量
    pub fn check_variable(&self, key: &str, value: &str) -> EnvironmentCheckResult {
        let mut issues = Vec::new();
        
        if self.is_blocked_var(key) {
            issues.push(format!("Variable '{}' is in blocked list", key));
        }
        
        if self.contains_sensitive_data(key, value) {
            issues.push(format!("Variable '{}' contains sensitive data", key));
        }
        
        let safe = issues.is_empty();
        let reason = if safe {
            "Variable is safe".to_string()
        } else {
            issues.join("; ")
        };
        
        EnvironmentCheckResult {
            safe,
            reason,
            issues,
        }
    }
    
    /// 检查是否为禁止的变量
    fn is_blocked_var(&self, key: &str) -> bool {
        let key_lower = key.to_lowercase();
        
        for blocked in &self.blocked_vars {
            if key_lower == blocked.to_lowercase() {
                return true;
            }
        }
        
        for pattern in &self.sensitive_patterns {
            if key_lower.contains(pattern) {
                return true;
            }
        }
        
        false
    }
    
    /// 检查是否包含敏感数据
    fn contains_sensitive_data(&self, key: &str, value: &str) -> bool {
        let key_lower = key.to_lowercase();
        
        for pattern in &self.sensitive_patterns {
            if key_lower.contains(pattern) && !value.is_empty() {
                return true;
            }
        }
        
        if value.len() > 32 && Self::looks_like_secret(value) {
            return true;
        }
        
        false
    }
    
    /// 判断是否看起来像密钥
    fn looks_like_secret(value: &str) -> bool {
        let has_upper = value.chars().any(|c| c.is_uppercase());
        let has_lower = value.chars().any(|c| c.is_lowercase());
        let has_digit = value.chars().any(|c| c.is_numeric());
        let has_special = value.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:',.<>?".contains(c));
        
        let variety_count = [has_upper, has_lower, has_digit, has_special]
            .iter()
            .filter(|&&x| x)
            .count();
        
        variety_count >= 3
    }
    
    /// 获取安全的环境变量
    pub fn get_safe_environment(&self) -> HashMap<String, String> {
        let mut safe_env = HashMap::new();
        
        for (key, value) in std::env::vars() {
            if self.is_allowed_var(&key) && !self.is_blocked_var(&key) {
                safe_env.insert(key, value);
            }
        }
        
        safe_env
    }
    
    /// 检查是否为允许的变量
    fn is_allowed_var(&self, key: &str) -> bool {
        if self.allowed_vars.is_empty() {
            return true;
        }
        
        for allowed in &self.allowed_vars {
            if key == allowed {
                return true;
            }
        }
        
        false
    }
    
    /// 添加敏感模式
    pub fn add_sensitive_pattern(&mut self, pattern: String) {
        if !self.sensitive_patterns.contains(&pattern) {
            self.sensitive_patterns.push(pattern);
        }
    }
    
    /// 添加允许的变量
    pub fn add_allowed_var(&mut self, var: String) {
        if !self.allowed_vars.contains(&var) {
            self.allowed_vars.push(var);
        }
    }
    
    /// 添加禁止的变量
    pub fn add_blocked_var(&mut self, var: String) {
        if !self.blocked_vars.contains(&var) {
            self.blocked_vars.push(var);
        }
    }
}

impl Default for EnvironmentChecker {
    fn default() -> Self {
        Self::new(
            vec![
                "PATH".to_string(),
                "HOME".to_string(),
                "USER".to_string(),
                "LANG".to_string(),
                "TERM".to_string(),
            ],
            vec![
                "API_KEY".to_string(),
                "SECRET".to_string(),
                "PASSWORD".to_string(),
                "TOKEN".to_string(),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_environment_checker() {
        let checker = EnvironmentChecker::default();
        let result = checker.check_environment();
        
        assert!(result.safe || !result.issues.is_empty());
    }
    
    #[test]
    fn test_check_variable() {
        let checker = EnvironmentChecker::default();
        
        let result = checker.check_variable("PATH", "/usr/bin");
        assert!(result.safe);
        
        let result = checker.check_variable("PASSWORD", "secret123");
        assert!(!result.safe);
    }
    
    #[test]
    fn test_looks_like_secret() {
        assert!(EnvironmentChecker::looks_like_secret("Abc123!@#defGHI"));
        assert!(!EnvironmentChecker::looks_like_secret("simple"));
    }
}
