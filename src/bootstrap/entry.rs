//! 入口点系统
//! 
//! 这个模块实现了 bootstrap-entry.ts 的 Rust 等效版本，包含：
//! - 宏注入机制
//! - 延迟加载功能
//! - 动态导入 CLI 入口

use crate::bootstrap::macros::ensure_bootstrap_macro;
use crate::bootstrap::cli::{CliFastPath, execute_fast_path};
use crate::error::Result;

/// 主入口点函数
/// 
/// 对应 TypeScript 版本的 bootstrap-entry.ts 主逻辑。
/// 确保宏配置已初始化，然后执行 CLI 主流程。
pub async fn bootstrap_entry() -> Result<()> {
    ensure_bootstrap_macro();
    execute_cli().await
}

/// 执行 CLI 主流程
/// 
/// 对应 TypeScript 版本的动态 import('./entrypoints/cli.tsx')。
async fn execute_cli() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    
    if let Some(fast_path) = CliFastPath::detect(&args) {
        execute_fast_path(fast_path).await?;
        return Ok(());
    }
    
    execute_full_cli(args).await
}

/// 执行完整的 CLI
/// 
/// 当没有快速路径匹配时，执行完整的 CLI 应用程序。
async fn execute_full_cli(args: Vec<String>) -> Result<()> {
    tracing::debug!("Executing full CLI with args: {:?}", args);
    
    crate::init().await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_macro_ensured() {
        ensure_bootstrap_macro();
        let config = crate::bootstrap::macros::get_macro_config();
        assert!(!config.version.is_empty());
    }
}
