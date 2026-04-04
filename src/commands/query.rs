//! 单次查询命令
//!
//! 这个模块实现了单次查询功能，使用 QueryEngine 处理查询。

use crate::config::Settings;
use crate::error::Result;
use crate::state::AppState;
use crate::tools;
use std::sync::Arc;

/// 运行单次查询
pub async fn run(query: String, settings: Settings, state: AppState) -> Result<()> {
    tracing::info!("Running query: {}", query);

    // 初始化工具系统
    let tool_manager = Arc::new(tools::init().await?);

    // 创建查询引擎
    let query_engine = crate::query::engine::QueryEngine::new(
        settings,
        state,
        tool_manager,
    ).await?;

    // 提交查询
    let result = query_engine.submit_message(&query).await?;

    // 显示结果
    if let Some(response) = result.response {
        if let Some(text) = response.text_content() {
            println!("{}", text);
        } else {
            println!("(Received non-text response)");
        }
    } else {
        println!("(No response generated)");
    }

    Ok(())
}
