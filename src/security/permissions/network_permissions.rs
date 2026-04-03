//! 网络权限管理器

use super::{PermissionDecision, PermissionRule, ApprovalLevel};
use crate::error::Result;
use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;
use url::Url;

/// 域名访问规则
#[derive(Debug, Clone)]
pub struct DomainRule {
    /// 域名模式
    pub pattern: String,
    /// 是否允许
    pub allowed: bool,
    /// 是否需要审批
    pub requires_approval: bool,
    /// 描述
    pub description: String,
}

/// 网络权限管理器
/// 
/// 管理网络访问权限，实现域名访问控制
#[derive(Debug)]
pub struct NetworkPermissionManager {
    /// 域名白名单
    whitelist: RwLock<HashSet<String>>,
    /// 域名黑名单
    blacklist: RwLock<HashSet<String>>,
    /// 域名规则
    domain_rules: RwLock<Vec<DomainRule>>,
    /// 权限规则
    rules: RwLock<HashMap<String, PermissionRule>>,
    /// 允许的端口
    allowed_ports: RwLock<HashSet<u16>>,
    /// 禁止的端口
    blocked_ports: RwLock<HashSet<u16>>,
    /// 是否允许私有网络访问
    allow_private_networks: RwLock<bool>,
}

impl NetworkPermissionManager {
    /// 创建新的网络权限管理器
    pub fn new() -> Self {
        Self {
            whitelist: RwLock::new(HashSet::new()),
            blacklist: RwLock::new(HashSet::new()),
            domain_rules: RwLock::new(Vec::new()),
            rules: RwLock::new(HashMap::new()),
            allowed_ports: RwLock::new(HashSet::new()),
            blocked_ports: RwLock::new(HashSet::new()),
            allow_private_networks: RwLock::new(false),
        }
    }
    
    /// 加载默认策略
    pub async fn load_default_policies(&mut self) -> Result<()> {
        let mut whitelist = self.whitelist.write().await;
        
        whitelist.insert("api.anthropic.com".to_string());
        whitelist.insert("api.openai.com".to_string());
        whitelist.insert("api.github.com".to_string());
        whitelist.insert("github.com".to_string());
        whitelist.insert("registry.npmjs.org".to_string());
        whitelist.insert("crates.io".to_string());
        whitelist.insert("pypi.org".to_string());
        
        let mut blacklist = self.blacklist.write().await;
        
        blacklist.insert("malware.com".to_string());
        blacklist.insert("phishing.com".to_string());
        
        let mut domain_rules = self.domain_rules.write().await;
        
        domain_rules.push(DomainRule {
            pattern: "*.anthropic.com".to_string(),
            allowed: true,
            requires_approval: false,
            description: "Anthropic API domains".to_string(),
        });
        
        domain_rules.push(DomainRule {
            pattern: "*.github.com".to_string(),
            allowed: true,
            requires_approval: false,
            description: "GitHub domains".to_string(),
        });
        
        domain_rules.push(DomainRule {
            pattern: "*.githubusercontent.com".to_string(),
            allowed: true,
            requires_approval: true,
            description: "GitHub user content".to_string(),
        });
        
        domain_rules.push(DomainRule {
            pattern: "localhost".to_string(),
            allowed: false,
            requires_approval: true,
            description: "Local development".to_string(),
        });
        
        domain_rules.push(DomainRule {
            pattern: "127.0.0.1".to_string(),
            allowed: false,
            requires_approval: true,
            description: "Local loopback".to_string(),
        });
        
        domain_rules.push(DomainRule {
            pattern: "192.168.*".to_string(),
            allowed: false,
            requires_approval: true,
            description: "Private network".to_string(),
        });
        
        domain_rules.push(DomainRule {
            pattern: "10.*".to_string(),
            allowed: false,
            requires_approval: true,
            description: "Private network".to_string(),
        });
        
        let mut allowed_ports = self.allowed_ports.write().await;
        allowed_ports.insert(80);
        allowed_ports.insert(443);
        allowed_ports.insert(8080);
        allowed_ports.insert(8443);
        allowed_ports.insert(3000);
        allowed_ports.insert(5000);
        
        let mut blocked_ports = self.blocked_ports.write().await;
        blocked_ports.insert(22);
        blocked_ports.insert(23);
        blocked_ports.insert(25);
        blocked_ports.insert(445);
        blocked_ports.insert(3389);
        
        let mut allow_private = self.allow_private_networks.write().await;
        *allow_private = false;
        
        Ok(())
    }
    
    /// 检查网络权限
    pub async fn check_permission(
        &self,
        url_str: &str,
        user_id: &str,
    ) -> Result<PermissionDecision> {
        let url = match Url::parse(url_str) {
            Ok(u) => u,
            Err(_) => {
                return Ok(PermissionDecision::Deny(
                    format!("Invalid URL: {}", url_str)
                ));
            }
        };
        
        let host = match url.host_str() {
            Some(h) => h,
            None => {
                return Ok(PermissionDecision::Deny(
                    "URL has no host".to_string()
                ));
            }
        };
        
        let port = url.port().unwrap_or(match url.scheme() {
            "http" => 80,
            "https" => 443,
            _ => 0,
        });
        
        let blacklist = self.blacklist.read().await;
        if blacklist.contains(host) {
            return Ok(PermissionDecision::Deny(
                format!("Domain '{}' is blacklisted", host)
            ));
        }
        
        let blocked_ports = self.blocked_ports.read().await;
        if blocked_ports.contains(&port) {
            return Ok(PermissionDecision::Deny(
                format!("Port {} is blocked", port)
            ));
        }
        
        if Self::is_private_network(host) {
            let allow_private = self.allow_private_networks.read().await;
            if !*allow_private {
                return Ok(PermissionDecision::RequireApproval {
                    level: ApprovalLevel::Double,
                    reason: format!("Access to private network '{}' requires approval", host),
                });
            }
        }
        
        let domain_rules = self.domain_rules.read().await;
        for rule in domain_rules.iter() {
            if Self::matches_domain_pattern(host, &rule.pattern) {
                if !rule.allowed {
                    if rule.requires_approval {
                        return Ok(PermissionDecision::RequireApproval {
                            level: ApprovalLevel::Single,
                            reason: format!("Domain '{}' requires approval", host),
                        });
                    } else {
                        return Ok(PermissionDecision::Deny(
                            format!("Domain '{}' is not allowed", host)
                        ));
                    }
                }
                
                if rule.requires_approval {
                    return Ok(PermissionDecision::RequireApproval {
                        level: ApprovalLevel::Single,
                        reason: format!("Domain '{}' requires approval", host),
                    });
                }
                
                return Ok(PermissionDecision::Allow);
            }
        }
        
        let whitelist = self.whitelist.read().await;
        if whitelist.contains(host) {
            return Ok(PermissionDecision::Allow);
        }
        
        let allowed_ports = self.allowed_ports.read().await;
        if !allowed_ports.contains(&port) {
            return Ok(PermissionDecision::RequireApproval {
                level: ApprovalLevel::Single,
                reason: format!("Port {} is not in allowed list", port),
            });
        }
        
        Ok(PermissionDecision::RequireApproval {
            level: ApprovalLevel::Single,
            reason: format!("Domain '{}' is not in whitelist", host),
        })
    }
    
    /// 检查是否为私有网络
    fn is_private_network(host: &str) -> bool {
        if host == "localhost" || host == "127.0.0.1" || host == "::1" {
            return true;
        }
        
        if host.starts_with("192.168.") || 
           host.starts_with("10.") || 
           host.starts_with("172.16.") ||
           host.starts_with("172.17.") ||
           host.starts_with("172.18.") ||
           host.starts_with("172.19.") ||
           host.starts_with("172.20.") ||
           host.starts_with("172.21.") ||
           host.starts_with("172.22.") ||
           host.starts_with("172.23.") ||
           host.starts_with("172.24.") ||
           host.starts_with("172.25.") ||
           host.starts_with("172.26.") ||
           host.starts_with("172.27.") ||
           host.starts_with("172.28.") ||
           host.starts_with("172.29.") ||
           host.starts_with("172.30.") ||
           host.starts_with("172.31.") {
            return true;
        }
        
        if host.ends_with(".local") || host.ends_with(".localhost") {
            return true;
        }
        
        false
    }
    
    /// 匹配域名模式
    fn matches_domain_pattern(host: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        
        if pattern == host {
            return true;
        }
        
        if pattern.starts_with("*.") {
            let suffix = &pattern[2..];
            return host.ends_with(suffix) || host == suffix;
        }
        
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return host.starts_with(parts[0]) && host.ends_with(parts[1]);
            }
        }
        
        false
    }
    
    /// 添加白名单域名
    pub async fn add_whitelist_domain(&self, domain: String) -> Result<()> {
        let mut whitelist = self.whitelist.write().await;
        whitelist.insert(domain);
        Ok(())
    }
    
    /// 添加黑名单域名
    pub async fn add_blacklist_domain(&self, domain: String) -> Result<()> {
        let mut blacklist = self.blacklist.write().await;
        blacklist.insert(domain);
        Ok(())
    }
    
    /// 添加域名规则
    pub async fn add_domain_rule(&self, rule: DomainRule) -> Result<()> {
        let mut domain_rules = self.domain_rules.write().await;
        domain_rules.push(rule);
        Ok(())
    }
    
    /// 添加允许端口
    pub async fn add_allowed_port(&self, port: u16) -> Result<()> {
        let mut allowed_ports = self.allowed_ports.write().await;
        allowed_ports.insert(port);
        Ok(())
    }
    
    /// 添加禁止端口
    pub async fn add_blocked_port(&self, port: u16) -> Result<()> {
        let mut blocked_ports = self.blocked_ports.write().await;
        blocked_ports.insert(port);
        Ok(())
    }
    
    /// 设置是否允许私有网络访问
    pub async fn set_allow_private_networks(&self, allow: bool) -> Result<()> {
        let mut allow_private = self.allow_private_networks.write().await;
        *allow_private = allow;
        Ok(())
    }
}

impl Default for NetworkPermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_network_permission_manager() {
        let mut manager = NetworkPermissionManager::new();
        manager.load_default_policies().await.unwrap();
        
        let decision = manager.check_permission(
            "https://api.anthropic.com/v1/messages",
            "user1"
        ).await.unwrap();
        
        assert_eq!(decision, PermissionDecision::Allow);
    }
    
    #[tokio::test]
    async fn test_blacklisted_domain() {
        let mut manager = NetworkPermissionManager::new();
        manager.load_default_policies().await.unwrap();
        
        let decision = manager.check_permission(
            "https://malware.com/payload",
            "user1"
        ).await.unwrap();
        
        match decision {
            PermissionDecision::Deny(_) => (),
            _ => panic!("Expected Deny for blacklisted domain"),
        }
    }
    
    #[tokio::test]
    async fn test_private_network() {
        let mut manager = NetworkPermissionManager::new();
        manager.load_default_policies().await.unwrap();
        
        let decision = manager.check_permission(
            "http://localhost:8080/api",
            "user1"
        ).await.unwrap();
        
        match decision {
            PermissionDecision::RequireApproval { .. } => (),
            _ => panic!("Expected RequireApproval for private network"),
        }
    }
}
