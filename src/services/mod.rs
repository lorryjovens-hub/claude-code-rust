//! 服务层模块
//! 
//! 这个模块实现了完整的服务层架构，包括：
//! - 分析和遥测服务
//! - API 客户端服务
//! - MCP 协议服务
//! - 上下文压缩服务
//! - 其他核心服务

pub mod analytics;
pub mod mcp;
pub mod api;
pub mod auto_dream;
pub mod compact;

// 重新导出主要类型
pub use analytics::{GrowthBookClient, FeatureFlag};
pub use mcp::{McpConnectionManager, McpServerConfig, McpTransport};
pub use api::{ApiClient, ClaudeApi};
pub use auto_dream::{AutoDream, AutoDreamConfig};
pub use compact::{
    CompactLevel, CompactConfig, CompactResult,
    MicrocompactService, SessionCompactService, MemoryCompactService,
};

use crate::error::Result;

/// 初始化服务层
pub async fn init() -> Result<()> {
    tracing::info!("Initializing service layer");
    
    // 初始化 GrowthBook
    analytics::init().await?;
    
    // 初始化 MCP
    mcp::init().await?;
    
    // 初始化 API 客户端
    api::init().await?;
    
    tracing::info!("Service layer initialized successfully");
    Ok(())
}
