//! 代理系统模块
//! 
//! 这个模块实现了多代理系统，对应 TypeScript 的 tools/AgentTool

pub mod types;
pub mod agent;
pub mod fork;
pub mod memory;
pub mod color;
pub mod built_in;

// 重新导出主要类型
pub use types::{AgentDefinition, AgentType, AgentConfig, AgentResult};
pub use agent::{AgentExecutor, AgentManager};
pub use fork::{ForkedAgent, ForkedAgentParams, CacheSafeParams};
pub use memory::AgentMemory;
pub use color::AgentColorManager;

use crate::error::Result;

/// 初始化代理系统
pub async fn init() -> Result<()> {
    tracing::info!("Initializing agent system");
    
    // 初始化内置代理
    built_in::init().await?;
    
    tracing::info!("Agent system initialized successfully");
    Ok(())
}
