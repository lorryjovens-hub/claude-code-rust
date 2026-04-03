//! 单次查询命令
//! 
//! 这个模块实现了单次查询功能

use crate::config::Settings;
use crate::error::Result;
use crate::state::AppState;

/// 运行单次查询
pub async fn run(query: String, settings: Settings, state: AppState) -> Result<()> {
    tracing::info!("Running query: {}", query);
    
    // TODO: 实现实际的查询逻辑
    // 1. 准备消息
    // 2. 调用 API
    // 3. 显示响应
    
    println!("Query: {}", query);
    println!("(Query processing not yet implemented)");
    
    Ok(())
}
