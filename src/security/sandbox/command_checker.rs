//! 命令检查器

use crate::error::Result;
use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;

/// 命令危险级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandDangerLevel {
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

/// 命令检查规则
#[derive(Debug, Clone)]
pub struct CommandRule {
    /// 命令模式
    pub pattern: String,
    /// 危险级别
    pub danger_level: CommandDangerLevel,
    /// 描述
    pub description: String,
    /// 是否需要沙箱
    pub requires_sandbox: bool,
}

/// 命令检查器
/// 
/// 检查命令的危险程度并决定是否需要沙箱执行
#[derive(Debug)]
pub struct CommandChecker {
    /// 命令规则
    rules: RwLock<Vec<CommandRule>>,
    /// 危险命令模式
    dangerous_patterns: RwLock<Vec<String>>,
    /// 安全命令白名单
    safe_commands: RwLock<HashSet<String>>,
}

impl CommandChecker {
    /// 创建新的命令检查器
    pub fn new() -> Self {
        Self {
            rules: RwLock::new(Vec::new()),
            dangerous_patterns: RwLock::new(Vec::new()),
            safe_commands: RwLock::new(HashSet::new()),
        }
    }
    
    /// 加载默认规则
    pub async fn load_default_rules(&mut self) -> Result<()> {
        let mut rules = self.rules.write().await;
        
        rules.push(CommandRule {
            pattern: "rm".to_string(),
            danger_level: CommandDangerLevel::High,
            description: "File removal command".to_string(),
            requires_sandbox: true,
        });
        
        rules.push(CommandRule {
            pattern: "dd".to_string(),
            danger_level: CommandDangerLevel::Critical,
            description: "Disk duplication command".to_string(),
            requires_sandbox: true,
        });
        
        rules.push(CommandRule {
            pattern: "mkfs".to_string(),
            danger_level: CommandDangerLevel::Critical,
            description: "Filesystem creation command".to_string(),
            requires_sandbox: true,
        });
        
        rules.push(CommandRule {
            pattern: "sudo".to_string(),
            danger_level: CommandDangerLevel::Critical,
            description: "Privilege escalation command".to_string(),
            requires_sandbox: true,
        });
        
        rules.push(CommandRule {
            pattern: "chmod".to_string(),
            danger_level: CommandDangerLevel::Medium,
            description: "Permission change command".to_string(),
            requires_sandbox: true,
        });
        
        rules.push(CommandRule {
            pattern: "chown".to_string(),
            danger_level: CommandDangerLevel::Medium,
            description: "Ownership change command".to_string(),
            requires_sandbox: true,
        });
        
        rules.push(CommandRule {
            pattern: "curl".to_string(),
            danger_level: CommandDangerLevel::Low,
            description: "Network request command".to_string(),
            requires_sandbox: false,
        });
        
        rules.push(CommandRule {
            pattern: "wget".to_string(),
            danger_level: CommandDangerLevel::Low,
            description: "Network download command".to_string(),
            requires_sandbox: false,
        });
        
        let mut patterns = self.dangerous_patterns.write().await;
        
        *patterns = vec![
            "rm -rf /".to_string(),
            "rm -rf /*".to_string(),
            ":(){ :|:& };:".to_string(),
            "> /dev/sd".to_string(),
            "mkfs.ext4".to_string(),
            "dd if=".to_string(),
            "chmod -R 777 /".to_string(),
            "chown -R".to_string(),
            "| sh".to_string(),
            "| bash".to_string(),
            "$(curl".to_string(),
            "$(wget".to_string(),
            "`curl".to_string(),
            "`wget".to_string(),
            "&& rm".to_string(),
            "&& sudo".to_string(),
            "> /dev/null".to_string(),
            "2>&1".to_string(),
        ];
        
        let mut safe = self.safe_commands.write().await;
        
        *safe = [
            "ls", "cat", "echo", "pwd", "whoami", "date", "which",
            "grep", "find", "sort", "uniq", "head", "tail", "wc",
            "mkdir", "touch", "cp", "mv", "ln", "diff", "tree",
        ].iter().map(|s| s.to_string()).collect();
        
        Ok(())
    }
    
    /// 检查命令
    pub async fn check_command(&self, command: &str) -> CommandDangerLevel {
        let base_command = self.extract_base_command(command);
        
        let safe = self.safe_commands.read().await;
        if safe.contains(base_command) {
            return CommandDangerLevel::Safe;
        }
        
        let patterns = self.dangerous_patterns.read().await;
        for pattern in patterns.iter() {
            if command.contains(pattern) {
                return CommandDangerLevel::Critical;
            }
        }
        
        let rules = self.rules.read().await;
        for rule in rules.iter() {
            if self.matches_pattern(&rule.pattern, base_command) {
                return rule.danger_level;
            }
        }
        
        CommandDangerLevel::Low
    }
    
    /// 判断是否为危险命令
    pub fn is_dangerous_command(&self, command: &str) -> bool {
        let base_command = self.extract_base_command(command);
        
        let dangerous_commands = [
            "rm", "dd", "mkfs", "fdisk", "format",
            "sudo", "su", "chmod", "chown",
            "shutdown", "reboot", "halt", "init",
            "systemctl", "service", "iptables", "ufw",
            "useradd", "userdel", "usermod", "passwd",
        ];
        
        if dangerous_commands.contains(&base_command) {
            return true;
        }
        
        let dangerous_patterns = [
            "rm -rf /",
            "rm -rf /*",
            "mkfs",
            "> /dev/sd",
            "dd if=",
            "chmod -R 777",
            "chown -R",
            "| sh",
            "| bash",
            "$(curl",
            "$(wget",
        ];
        
        dangerous_patterns.iter().any(|pattern| command.contains(pattern))
    }
    
    /// 判断是否需要沙箱
    pub async fn requires_sandbox(&self, command: &str) -> bool {
        let base_command = self.extract_base_command(command);
        
        let rules = self.rules.read().await;
        for rule in rules.iter() {
            if self.matches_pattern(&rule.pattern, base_command) {
                return rule.requires_sandbox;
            }
        }
        
        self.is_dangerous_command(command)
    }
    
    /// 提取基础命令
    fn extract_base_command<'a>(&self, command: &'a str) -> &'a str {
        command
            .split_whitespace()
            .next()
            .unwrap_or(command)
    }
    
    /// 匹配模式
    fn matches_pattern(&self, pattern: &str, command: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        
        if pattern == command {
            return true;
        }
        
        if pattern.starts_with("*.") {
            let suffix = &pattern[2..];
            return command.ends_with(suffix);
        }
        
        if pattern.ends_with(".*") {
            let prefix = &pattern[..pattern.len() - 2];
            return command.starts_with(prefix);
        }
        
        false
    }
    
    /// 添加规则
    pub async fn add_rule(&self, rule: CommandRule) -> Result<()> {
        let mut rules = self.rules.write().await;
        rules.push(rule);
        Ok(())
    }
    
    /// 添加危险模式
    pub async fn add_dangerous_pattern(&self, pattern: String) -> Result<()> {
        let mut patterns = self.dangerous_patterns.write().await;
        patterns.push(pattern);
        Ok(())
    }
    
    /// 添加安全命令
    pub async fn add_safe_command(&self, command: String) -> Result<()> {
        let mut safe = self.safe_commands.write().await;
        safe.insert(command);
        Ok(())
    }
}

impl Default for CommandChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_command_checker() {
        let mut checker = CommandChecker::new();
        checker.load_default_rules().await.unwrap();
        
        let level = checker.check_command("ls -la").await;
        assert_eq!(level, CommandDangerLevel::Safe);
        
        let level = checker.check_command("rm -rf /").await;
        assert_eq!(level, CommandDangerLevel::Critical);
    }
    
    #[test]
    fn test_is_dangerous_command() {
        let checker = CommandChecker::new();
        
        assert!(checker.is_dangerous_command("rm -rf /"));
        assert!(checker.is_dangerous_command("sudo su"));
        assert!(!checker.is_dangerous_command("ls -la"));
    }
}
