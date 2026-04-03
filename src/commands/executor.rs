//! 命令执行流程
//! 
//! 这个模块实现了命令执行的完整生命周期

use crate::error::Result;
use crate::state::AppState;
use crate::config::Config;
use super::types::{Command, CommandContext, CommandResult};
use super::registry::CommandRegistry;

/// 用户输入
#[derive(Debug, Clone)]
pub struct UserInput {
    /// 原始输入
    pub raw: String,
    
    /// 命令名称（如果是命令）
    pub command_name: Option<String>,
    
    /// 命令参数
    pub command_args: Option<String>,
    
    /// 是否为命令
    pub is_command: bool,
}

impl UserInput {
    /// 解析用户输入
    pub fn parse(input: String) -> Self {
        // 先获取 trimmed 后的引用
        let starts_with_slash = input.trim().starts_with('/');
        
        if starts_with_slash {
            let without_slash = input.trim()[1..].to_string();
            
            // 查找第一个空格
            if let Some(space_pos) = without_slash.find(' ') {
                let name = without_slash[..space_pos].to_string();
                let args = without_slash[space_pos + 1..].trim().to_string();
                Self {
                    raw: input,
                    command_name: Some(name),
                    command_args: Some(args),
                    is_command: true,
                }
            } else {
                Self {
                    raw: input,
                    command_name: Some(without_slash),
                    command_args: None,
                    is_command: true,
                }
            }
        } else {
            Self {
                raw: input,
                command_name: None,
                command_args: None,
                is_command: false,
            }
        }
    }
}

/// 命令路由器
pub struct CommandRouter {
    /// 命令注册表
    registry: CommandRegistry,
}

impl CommandRouter {
    /// 创建新的命令路由器
    pub fn new(registry: CommandRegistry) -> Self {
        Self { registry }
    }
    
    /// 路由命令
    pub async fn route(&self, input: UserInput) -> Option<(String, String)> {
        if !input.is_command {
            return None;
        }
        
        let name = input.command_name?;
        
        // 检查命令是否存在
        if self.registry.has(&name).await {
            Some((name, input.command_args.unwrap_or_default()))
        } else {
            None
        }
    }
    
    /// 获取注册表
    pub fn registry(&self) -> &CommandRegistry {
        &self.registry
    }
}

/// 权限检查器
pub struct PermissionChecker;

impl PermissionChecker {
    /// 检查命令执行权限
    pub fn check(_command: &Command, _context: &CommandContext) -> Result<bool> {
        // TODO: 实现权限检查逻辑
        Ok(true)
    }
}

/// 命令执行器
pub struct CmdExecutor {
    /// 命令路由器
    router: CommandRouter,
}

impl CmdExecutor {
    /// 创建新的命令执行器
    pub fn new(registry: CommandRegistry) -> Self {
        Self {
            router: CommandRouter::new(registry),
        }
    }
    
    /// 执行用户输入
    pub async fn execute(
        &self,
        input: String,
        cwd: std::path::PathBuf,
        config: Config,
        state: AppState,
    ) -> Result<ExecuteResult> {
        // 1. 解析用户输入
        let user_input = UserInput::parse(input);
        
        // 2. 路由命令
        let route_result = self.router.route(user_input.clone()).await;
        
        match route_result {
            Some((name, args)) => {
                // 3. 构建执行上下文
                let context = CommandContext {
                    cwd,
                    config,
                    state,
                    args,
                };
                
                // 4. 查找命令
                let executor = self.router.registry().find(&name).await
                    .ok_or_else(|| crate::error::ClaudeError::Command(format!("Command not found: {}", name)))?;
                
                let command = executor.command();
                
                // 5. 权限检查
                PermissionChecker::check(&command, &context)?;
                
                // 6. 执行命令
                let result = executor.execute(context).await?;
                
                Ok(ExecuteResult::Command(result))
            }
            None => {
                // 不是命令或命令不存在，作为普通消息处理
                Ok(ExecuteResult::Message(user_input.raw))
            }
        }
    }
    
    /// 获取路由器
    pub fn router(&self) -> &CommandRouter {
        &self.router
    }
}

/// 执行结果
#[derive(Debug)]
pub enum ExecuteResult {
    /// 命令结果
    Command(CommandResult),
    /// 普通消息
    Message(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_input_parse_command() {
        let input = UserInput::parse("/test arg1 arg2".to_string());
        assert!(input.is_command);
        assert_eq!(input.command_name, Some("test".to_string()));
        assert_eq!(input.command_args, Some("arg1 arg2".to_string()));
    }
    
    #[test]
    fn test_user_input_parse_command_no_args() {
        let input = UserInput::parse("/test".to_string());
        assert!(input.is_command);
        assert_eq!(input.command_name, Some("test".to_string()));
        assert_eq!(input.command_args, None);
    }
    
    #[test]
    fn test_user_input_parse_message() {
        let input = UserInput::parse("hello world".to_string());
        assert!(!input.is_command);
        assert_eq!(input.command_name, None);
        assert_eq!(input.command_args, None);
    }
}
