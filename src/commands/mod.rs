//! 命令系统模块
//! 
//! 这个模块实现了完整的命令系统架构，包括：
//! - 命令类型系统
//! - 命令注册系统
//! - 命令执行流程
//! - 核心命令实现

pub mod types;
pub mod registry;
pub mod executor;
pub mod builtin;

// CLI 命令模块
pub mod interactive;
pub mod query;
pub mod config;
pub mod auth;
pub mod ultraplan;

// 重新导出主要类型
pub use types::{Command, CommandContext, CommandResult, CommandBase};
pub use registry::{CommandRegistry, CommandManager, CommandLoader, CommandExecutor};
pub use executor::{CmdExecutor, CommandRouter, UserInput, ExecuteResult};
pub use builtin::{
    VersionCommand, HelpCommand, ClearCommand, ExitCommand,
    ConfigCommand, McpCommand, StatusCommand,
};
pub use ultraplan::{
    UltraplanService, UltraplanConfig, UltraplanSession, UltraplanPhase,
    UltraplanResult, ExecutionTarget, PlanEvaluation, build_ultraplan_prompt,
};

use crate::error::Result;

/// 初始化命令系统
pub async fn init() -> Result<CommandManager> {
    let mut manager = CommandManager::new();
    
    // 注册核心命令加载器
    manager.add_loader(BuiltinCommandLoader);
    
    // 加载所有命令
    manager.load_all().await?;
    
    tracing::info!("Command system initialized with {} commands", 
        manager.registry().len().await);
    
    Ok(manager)
}

/// 内置命令加载器
struct BuiltinCommandLoader;

#[async_trait::async_trait]
impl CommandLoader for BuiltinCommandLoader {
    async fn load(&self, registry: &CommandRegistry) -> Result<()> {
        // 注册核心命令
        registry.register(VersionCommand).await;
        registry.register(HelpCommand).await;
        registry.register(ClearCommand).await;
        registry.register(ExitCommand).await;
        registry.register(ConfigCommand).await;
        registry.register(McpCommand).await;
        registry.register(StatusCommand).await;
        
        tracing::debug!("Loaded {} builtin commands", 7);
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        "builtin"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_init_command_system() {
        let manager = init().await.unwrap();
        assert!(manager.registry().len().await >= 7);
    }
    
    #[tokio::test]
    async fn test_builtin_commands_loaded() {
        let manager = init().await.unwrap();
        
        assert!(manager.registry().has("version").await);
        assert!(manager.registry().has("help").await);
        assert!(manager.registry().has("clear").await);
        assert!(manager.registry().has("exit").await);
        assert!(manager.registry().has("config").await);
        assert!(manager.registry().has("mcp").await);
        assert!(manager.registry().has("status").await);
    }
}
