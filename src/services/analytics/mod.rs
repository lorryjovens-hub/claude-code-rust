//! 分析和遥测服务模块
//! 
//! 这个模块实现了分析、遥测和特性开关功能

pub mod growthbook;

// 重新导出主要类型
pub use growthbook::{
    GrowthBookClient, GrowthBookUserAttributes, FeatureFlag,
    GrowthBookConfig, ExperimentData,
};

use crate::error::Result;

/// 初始化分析服务
pub async fn init() -> Result<()> {
    tracing::debug!("Initializing analytics services");
    
    // 初始化 GrowthBook
    growthbook::init().await?;
    
    tracing::debug!("Analytics services initialized");
    Ok(())
}
