//! 命令注册系统
//! 
//! 这个模块实现了命令注册表和命令发现机制，对应 TypeScript 的 commands.ts

use crate::error::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use super::types::{Command, CommandContext, CommandResult};

/// 命令执行器 trait
#[async_trait::async_trait]
pub trait CommandExecutor: Send + Sync {
    /// 执行命令
    async fn execute(&self, context: CommandContext) -> Result<CommandResult>;
    
    /// 获取命令定义
    fn command(&self) -> Command;
}

/// 命令注册表
pub struct CommandRegistry {
    /// 命令映射（名称 -> 执行器）
    commands: Arc<RwLock<HashMap<String, Arc<dyn CommandExecutor>>>>,
    
    /// 别名映射（别名 -> 命令名称）
    aliases: Arc<RwLock<HashMap<String, String>>>,
}

impl CommandRegistry {
    /// 创建新的命令注册表
    pub fn new() -> Self {
        Self {
            commands: Arc::new(RwLock::new(HashMap::new())),
            aliases: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 注册命令
    pub async fn register<E>(&self, executor: E) 
    where
        E: CommandExecutor + 'static,
    {
        let command = executor.command();
        let name = command.name().to_string();
        
        // 注册命令
        self.commands.write().await.insert(name.clone(), Arc::new(executor));
        
        // 注册别名
        if let Some(aliases) = command.aliases() {
            let mut aliases_map = self.aliases.write().await;
            for alias in aliases {
                aliases_map.insert(alias.clone(), name.clone());
            }
        }
    }
    
    /// 注销命令
    pub async fn unregister(&self, name: &str) -> Option<Arc<dyn CommandExecutor>> {
        let executor = self.commands.write().await.remove(name);
        
        // 清理别名
        if executor.is_some() {
            let mut aliases = self.aliases.write().await;
            aliases.retain(|_, v| v != name);
        }
        
        executor
    }
    
    /// 查找命令
    pub async fn find(&self, name: &str) -> Option<Arc<dyn CommandExecutor>> {
        // 先查找命令名称
        if let Some(executor) = self.commands.read().await.get(name) {
            return Some(executor.clone());
        }
        
        // 再查找别名
        if let Some(real_name) = self.aliases.read().await.get(name) {
            return self.commands.read().await.get(real_name).cloned();
        }
        
        None
    }
    
    /// 检查命令是否存在
    pub async fn has(&self, name: &str) -> bool {
        self.find(name).await.is_some()
    }
    
    /// 获取所有命令
    pub async fn list(&self) -> Vec<Arc<dyn CommandExecutor>> {
        self.commands.read().await.values().cloned().collect()
    }
    
    /// 获取所有命令名称
    pub async fn names(&self) -> Vec<String> {
        self.commands.read().await.keys().cloned().collect()
    }
    
    /// 清空注册表
    pub async fn clear(&self) {
        self.commands.write().await.clear();
        self.aliases.write().await.clear();
    }
    
    /// 命令数量
    pub async fn len(&self) -> usize {
        self.commands.read().await.len()
    }
    
    /// 是否为空
    pub async fn is_empty(&self) -> bool {
        self.commands.read().await.is_empty()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for CommandRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandRegistry")
            .finish_non_exhaustive()
    }
}

/// 命令加载器 trait
#[async_trait::async_trait]
pub trait CommandLoader: Send + Sync {
    /// 加载命令
    async fn load(&self, registry: &CommandRegistry) -> Result<()>;
    
    /// 加载器名称
    fn name(&self) -> &str;
}

/// 命令管理器
pub struct CommandManager {
    /// 命令注册表
    registry: CommandRegistry,
    
    /// 命令加载器
    loaders: Vec<Box<dyn CommandLoader>>,
}

impl CommandManager {
    /// 创建新的命令管理器
    pub fn new() -> Self {
        Self {
            registry: CommandRegistry::new(),
            loaders: Vec::new(),
        }
    }
    
    /// 注册加载器
    pub fn add_loader<L>(&mut self, loader: L)
    where
        L: CommandLoader + 'static,
    {
        self.loaders.push(Box::new(loader));
    }
    
    /// 加载所有命令
    pub async fn load_all(&self) -> Result<()> {
        for loader in &self.loaders {
            tracing::debug!("Loading commands from: {}", loader.name());
            loader.load(&self.registry).await?;
        }
        Ok(())
    }
    
    /// 获取注册表
    pub fn registry(&self) -> &CommandRegistry {
        &self.registry
    }
    
    /// 执行命令
    pub async fn execute(&self, name: &str, context: CommandContext) -> Result<CommandResult> {
        let executor = self.registry.find(name).await
            .ok_or_else(|| crate::error::ClaudeError::Command(format!("Command not found: {}", name)))?;
        
        executor.execute(context).await
    }
}

impl Default for CommandManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for CommandManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandManager")
            .field("loader_count", &self.loaders.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::types::{CommandBase, PromptCommand, CommandSource};
    
    struct TestCommand;
    
    #[async_trait::async_trait]
    impl CommandExecutor for TestCommand {
        async fn execute(&self, _context: CommandContext) -> Result<CommandResult> {
            Ok(CommandResult {
                content: "Test result".to_string(),
                ..Default::default()
            })
        }
        
        fn command(&self) -> Command {
            Command::Prompt(PromptCommand {
                base: CommandBase {
                    name: "test".to_string(),
                    description: "Test command".to_string(),
                    has_user_specified_description: None,
                    aliases: Some(vec!["t".to_string()]),
                    availability: None,
                    is_hidden: None,
                    is_mcp: None,
                    argument_hint: None,
                    when_to_use: None,
                    version: None,
                    disable_model_invocation: None,
                    user_invocable: None,
                    loaded_from: None,
                    kind: None,
                    immediate: None,
                    is_sensitive: None,
                },
                progress_message: "Testing...".to_string(),
                content_length: 100,
                arg_names: None,
                allowed_tools: None,
                model: None,
                source: CommandSource::Builtin,
                plugin_info: None,
                disable_non_interactive: None,
                context: None,
                agent: None,
                effort: None,
                paths: None,
            })
        }
    }
    
    #[tokio::test]
    async fn test_register_command() {
        let registry = CommandRegistry::new();
        registry.register(TestCommand).await;
        
        assert!(registry.has("test").await);
        assert!(registry.has("t").await);
        assert_eq!(registry.len().await, 1);
    }
    
    #[tokio::test]
    async fn test_unregister_command() {
        let registry = CommandRegistry::new();
        registry.register(TestCommand).await;
        
        assert!(registry.has("test").await);
        
        registry.unregister("test").await;
        
        assert!(!registry.has("test").await);
        assert!(!registry.has("t").await);
    }
    
    #[tokio::test]
    async fn test_find_command() {
        let registry = CommandRegistry::new();
        registry.register(TestCommand).await;
        
        let executor = registry.find("test").await;
        assert!(executor.is_some());
        
        let executor = registry.find("t").await;
        assert!(executor.is_some());
        
        let executor = registry.find("nonexistent").await;
        assert!(executor.is_none());
    }
}
