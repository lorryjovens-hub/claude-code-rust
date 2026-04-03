//! API 客户端服务
//! 
//! 这个模块实现了 API 客户端功能

pub mod claude;
pub mod client;
pub mod usage;

// 重新导出主要类型
pub use claude::ClaudeApi;
pub use client::ApiClient;
pub use usage::UsageStats;

use crate::error::Result;

/// 初始化 API 服务
pub async fn init() -> Result<()> {
    tracing::info!("Initializing API service");
    
    // TODO: 实现 API 初始化逻辑
    
    tracing::info!("API service initialized successfully");
    Ok(())
}
