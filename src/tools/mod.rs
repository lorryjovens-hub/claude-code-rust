//! 工具系统模块
//! 
//! 这个模块实现了完整的工具系统架构，包括：
//! - 工具类型系统
//! - 工具权限系统
//! - 工具注册系统
//! - 核心工具实现

pub mod types;
pub mod permissions;
pub mod base;
pub mod registry;
pub mod file_tools;
pub mod search_tools;
pub mod command_tools;

// 重新导出主要类型
pub use types::{
    ToolMetadata, ToolResult, ToolUseContext, ToolInputSchema,
    ToolCategory, ToolPermissionLevel, ValidationResult, PermissionResult,
    PermissionMode, PermissionBehavior, ToolPermissionContext,
};
pub use base::{Tool, ToolBuilder};
pub use registry::{ToolRegistry, ToolManager, ToolLoader};
pub use permissions::{PermissionChecker, ModeChecker};
pub use file_tools::{FileReadTool, FileEditTool, FileWriteTool};
pub use search_tools::{GlobTool, GrepTool};
pub use command_tools::{BashTool, PowerShellTool};

use crate::error::Result;

/// 初始化工具系统
pub async fn init() -> Result<ToolManager> {
    let mut manager = ToolManager::new();
    
    // 注册核心工具加载器
    manager.add_loader(BuiltinToolLoader);
    
    // 加载所有工具
    manager.load_all().await?;
    
    tracing::info!("Tool system initialized with {} tools", 
        manager.registry().len().await);
    
    Ok(manager)
}

/// 内置工具加载器
struct BuiltinToolLoader;

#[async_trait::async_trait]
impl ToolLoader for BuiltinToolLoader {
    async fn load(&self, registry: &ToolRegistry) -> Result<()> {
        // 注册文件操作工具
        registry.register(FileReadTool).await;
        registry.register(FileEditTool).await;
        registry.register(FileWriteTool).await;
        
        // 注册代码搜索工具
        registry.register(GlobTool).await;
        registry.register(GrepTool).await;
        
        // 注册命令执行工具
        registry.register(BashTool).await;
        registry.register(PowerShellTool).await;
        
        tracing::debug!("Loaded {} builtin tools", 7);
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        "builtin"
    }
}

/// 获取所有工具名称
pub fn get_tool_names() -> Vec<String> {
    vec![
        "Read".to_string(),
        "Edit".to_string(),
        "Write".to_string(),
        "Glob".to_string(),
        "Grep".to_string(),
        "Bash".to_string(),
        "PowerShell".to_string(),
    ]
}

/// 工具预设
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolPreset {
    /// 默认预设
    Default,
    /// 简单预设（只读工具）
    Simple,
    /// 完整预设（所有工具）
    Full,
}

impl ToolPreset {
    /// 获取预设的工具名称
    pub fn tool_names(&self) -> Vec<String> {
        match self {
            ToolPreset::Default => vec![
                "Read".to_string(),
                "Edit".to_string(),
                "Write".to_string(),
                "Glob".to_string(),
                "Grep".to_string(),
                "Bash".to_string(),
            ],
            ToolPreset::Simple => vec![
                "Read".to_string(),
                "Glob".to_string(),
                "Grep".to_string(),
            ],
            ToolPreset::Full => get_tool_names(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_init_tool_system() {
        let manager = init().await.unwrap();
        assert!(manager.registry().len().await >= 7);
    }
    
    #[tokio::test]
    async fn test_builtin_tools_loaded() {
        let manager = init().await.unwrap();
        
        assert!(manager.registry().has("Read").await);
        assert!(manager.registry().has("Edit").await);
        assert!(manager.registry().has("Write").await);
        assert!(manager.registry().has("Glob").await);
        assert!(manager.registry().has("Grep").await);
        assert!(manager.registry().has("Bash").await);
        assert!(manager.registry().has("PowerShell").await);
    }
    
    #[tokio::test]
    async fn test_tool_aliases() {
        let manager = init().await.unwrap();
        
        assert!(manager.registry().has("read").await);
        assert!(manager.registry().has("cat").await);
        assert!(manager.registry().has("edit").await);
        assert!(manager.registry().has("bash").await);
    }
    
    #[test]
    fn test_tool_preset() {
        let default = ToolPreset::Default;
        assert_eq!(default.tool_names().len(), 6);
        
        let simple = ToolPreset::Simple;
        assert_eq!(simple.tool_names().len(), 3);
        
        let full = ToolPreset::Full;
        assert!(full.tool_names().len() >= 7);
    }
}
