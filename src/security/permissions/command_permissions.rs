//! 命令权限管理器

use super::{PermissionDecision, PermissionRule, ApprovalLevel};
use crate::error::Result;
use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;

/// 危险命令级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DangerLevel {
    /// 安全
    Safe,
    /// 低危
    Low,
    /// 中危
    Medium,
    /// 高危
    High,
    /// 极危
    Critical,
}

/// 命令分类
#[derive(Debug, Clone)]
pub struct CommandCategory {
    /// 分类名称
    pub name: String,
    /// 分类描述
    pub description: String,
    /// 危险级别
    pub danger_level: DangerLevel,
    /// 命令列表
    pub commands: Vec<String>,
}

/// 命令权限管理器
/// 
/// 管理命令执行权限，实现危险命令识别与审批
#[derive(Debug)]
pub struct CommandPermissionManager {
    /// 危险命令列表
    dangerous_commands: RwLock<HashMap<String, DangerLevel>>,
    /// 命令分类
    command_categories: RwLock<Vec<CommandCategory>>,
    /// 权限规则
    rules: RwLock<HashMap<String, PermissionRule>>,
    /// 审批配置
    approval_config: RwLock<HashMap<DangerLevel, ApprovalLevel>>,
    /// 命令白名单
    whitelist: RwLock<HashSet<String>>,
}

impl CommandPermissionManager {
    /// 创建新的命令权限管理器
    pub fn new() -> Self {
        Self {
            dangerous_commands: RwLock::new(HashMap::new()),
            command_categories: RwLock::new(Vec::new()),
            rules: RwLock::new(HashMap::new()),
            approval_config: RwLock::new(HashMap::new()),
            whitelist: RwLock::new(HashSet::new()),
        }
    }
    
    /// 加载默认策略
    pub async fn load_default_policies(&mut self) -> Result<()> {
        let mut dangerous = self.dangerous_commands.write().await;
        
        dangerous.insert("rm".to_string(), DangerLevel::High);
        dangerous.insert("rmdir".to_string(), DangerLevel::Medium);
        dangerous.insert("dd".to_string(), DangerLevel::Critical);
        dangerous.insert("mkfs".to_string(), DangerLevel::Critical);
        dangerous.insert("fdisk".to_string(), DangerLevel::Critical);
        dangerous.insert("format".to_string(), DangerLevel::Critical);
        dangerous.insert("chmod".to_string(), DangerLevel::Medium);
        dangerous.insert("chown".to_string(), DangerLevel::Medium);
        dangerous.insert("shutdown".to_string(), DangerLevel::Critical);
        dangerous.insert("reboot".to_string(), DangerLevel::Critical);
        dangerous.insert("halt".to_string(), DangerLevel::Critical);
        dangerous.insert("init".to_string(), DangerLevel::Critical);
        dangerous.insert("systemctl".to_string(), DangerLevel::High);
        dangerous.insert("service".to_string(), DangerLevel::High);
        dangerous.insert("iptables".to_string(), DangerLevel::Critical);
        dangerous.insert("ufw".to_string(), DangerLevel::High);
        dangerous.insert("curl".to_string(), DangerLevel::Low);
        dangerous.insert("wget".to_string(), DangerLevel::Low);
        dangerous.insert("nc".to_string(), DangerLevel::High);
        dangerous.insert("netcat".to_string(), DangerLevel::High);
        dangerous.insert("ssh".to_string(), DangerLevel::Medium);
        dangerous.insert("scp".to_string(), DangerLevel::Medium);
        dangerous.insert("rsync".to_string(), DangerLevel::Medium);
        dangerous.insert("sudo".to_string(), DangerLevel::Critical);
        dangerous.insert("su".to_string(), DangerLevel::Critical);
        dangerous.insert("passwd".to_string(), DangerLevel::High);
        dangerous.insert("useradd".to_string(), DangerLevel::High);
        dangerous.insert("userdel".to_string(), DangerLevel::High);
        dangerous.insert("usermod".to_string(), DangerLevel::High);
        
        let mut categories = self.command_categories.write().await;
        
        categories.push(CommandCategory {
            name: "File System".to_string(),
            description: "File system operations".to_string(),
            danger_level: DangerLevel::Medium,
            commands: vec!["rm".to_string(), "rmdir".to_string(), "dd".to_string()],
        });
        
        categories.push(CommandCategory {
            name: "System Administration".to_string(),
            description: "System administration commands".to_string(),
            danger_level: DangerLevel::High,
            commands: vec![
                "shutdown".to_string(),
                "reboot".to_string(),
                "systemctl".to_string(),
                "service".to_string(),
            ],
        });
        
        categories.push(CommandCategory {
            name: "Network".to_string(),
            description: "Network operations".to_string(),
            danger_level: DangerLevel::Medium,
            commands: vec![
                "curl".to_string(),
                "wget".to_string(),
                "nc".to_string(),
                "netcat".to_string(),
            ],
        });
        
        categories.push(CommandCategory {
            name: "User Management".to_string(),
            description: "User management commands".to_string(),
            danger_level: DangerLevel::High,
            commands: vec![
                "useradd".to_string(),
                "userdel".to_string(),
                "usermod".to_string(),
                "passwd".to_string(),
            ],
        });
        
        categories.push(CommandCategory {
            name: "Privilege Escalation".to_string(),
            description: "Privilege escalation commands".to_string(),
            danger_level: DangerLevel::Critical,
            commands: vec!["sudo".to_string(), "su".to_string()],
        });
        
        let mut approval = self.approval_config.write().await;
        approval.insert(DangerLevel::Safe, ApprovalLevel::Single);
        approval.insert(DangerLevel::Low, ApprovalLevel::Single);
        approval.insert(DangerLevel::Medium, ApprovalLevel::Single);
        approval.insert(DangerLevel::High, ApprovalLevel::Double);
        approval.insert(DangerLevel::Critical, ApprovalLevel::Triple);
        
        let mut whitelist = self.whitelist.write().await;
        whitelist.insert("ls".to_string());
        whitelist.insert("cat".to_string());
        whitelist.insert("echo".to_string());
        whitelist.insert("pwd".to_string());
        whitelist.insert("whoami".to_string());
        whitelist.insert("date".to_string());
        whitelist.insert("which".to_string());
        whitelist.insert("grep".to_string());
        whitelist.insert("find".to_string());
        whitelist.insert("sort".to_string());
        whitelist.insert("uniq".to_string());
        whitelist.insert("head".to_string());
        whitelist.insert("tail".to_string());
        whitelist.insert("wc".to_string());
        
        Ok(())
    }
    
    /// 检查命令权限
    pub async fn check_permission(
        &self,
        command: &str,
        user_id: &str,
    ) -> Result<PermissionDecision> {
        let parsed = Self::parse_command(command);
        let base_command = parsed.first().unwrap_or(&command);
        
        let whitelist = self.whitelist.read().await;
        if whitelist.contains(*base_command) {
            return Ok(PermissionDecision::Allow);
        }
        
        let dangerous = self.dangerous_commands.read().await;
        if let Some(&danger_level) = dangerous.get(*base_command) {
            let approval_config = self.approval_config.read().await;
            let approval_level = approval_config.get(&danger_level)
                .copied()
                .unwrap_or(ApprovalLevel::Single);
            
            return Ok(PermissionDecision::RequireApproval {
                level: approval_level,
                reason: format!(
                    "Command '{}' is classified as {:?} danger level",
                    base_command, danger_level
                ),
            });
        }
        
        if Self::contains_dangerous_patterns(command) {
            return Ok(PermissionDecision::RequireApproval {
                level: ApprovalLevel::Double,
                reason: "Command contains potentially dangerous patterns".to_string(),
            });
        }
        
        Ok(PermissionDecision::Allow)
    }
    
    /// 解析命令
    fn parse_command(command: &str) -> Vec<&str> {
        command
            .split_whitespace()
            .collect()
    }
    
    /// 检查危险模式
    fn contains_dangerous_patterns(command: &str) -> bool {
        let dangerous_patterns = [
            "rm -rf /",
            "rm -rf /*",
            ":(){ :|:& };:",
            "mkfs",
            "> /dev/sd",
            "dd if=",
            "chmod -R 777",
            "chown -R",
            "| sh",
            "| bash",
            "$(curl",
            "$(wget",
            "`curl",
            "`wget",
            "&& rm",
            "&& sudo",
        ];
        
        dangerous_patterns.iter().any(|pattern| command.contains(pattern))
    }
    
    /// 添加危险命令
    pub async fn add_dangerous_command(
        &self,
        command: String,
        level: DangerLevel,
    ) -> Result<()> {
        let mut dangerous = self.dangerous_commands.write().await;
        dangerous.insert(command, level);
        Ok(())
    }
    
    /// 添加白名单命令
    pub async fn add_whitelist_command(&self, command: String) -> Result<()> {
        let mut whitelist = self.whitelist.write().await;
        whitelist.insert(command);
        Ok(())
    }
    
    /// 移除白名单命令
    pub async fn remove_whitelist_command(&self, command: &str) -> Result<()> {
        let mut whitelist = self.whitelist.write().await;
        whitelist.remove(command);
        Ok(())
    }
    
    /// 获取命令危险级别
    pub async fn get_danger_level(&self, command: &str) -> Option<DangerLevel> {
        let parsed = Self::parse_command(command);
        let base_command = parsed.first()?;
        
        let dangerous = self.dangerous_commands.read().await;
        dangerous.get(*base_command).copied()
    }
    
    /// 检查命令是否需要沙箱
    pub async fn requires_sandbox(&self, command: &str) -> bool {
        let parsed = Self::parse_command(command);
        let base_command = parsed.first().unwrap_or(&command);
        
        let whitelist = self.whitelist.read().await;
        if whitelist.contains(*base_command) {
            return false;
        }
        
        let dangerous = self.dangerous_commands.read().await;
        dangerous.contains_key(*base_command) || Self::contains_dangerous_patterns(command)
    }
}

impl Default for CommandPermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_command_permission_manager() {
        let mut manager = CommandPermissionManager::new();
        manager.load_default_policies().await.unwrap();
        
        let decision = manager.check_permission("ls -la", "user1").await.unwrap();
        assert_eq!(decision, PermissionDecision::Allow);
    }
    
    #[tokio::test]
    async fn test_dangerous_command() {
        let mut manager = CommandPermissionManager::new();
        manager.load_default_policies().await.unwrap();
        
        let decision = manager.check_permission("rm -rf /", "user1").await.unwrap();
        
        match decision {
            PermissionDecision::RequireApproval { .. } => (),
            _ => panic!("Expected RequireApproval for dangerous command"),
        }
    }
    
    #[tokio::test]
    async fn test_danger_level() {
        let mut manager = CommandPermissionManager::new();
        manager.load_default_policies().await.unwrap();
        
        let level = manager.get_danger_level("rm").await;
        assert_eq!(level, Some(DangerLevel::High));
    }
}
