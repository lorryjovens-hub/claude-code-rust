//! 启动流程系统
//! 
//! 这个模块实现了 Claude Code 的完整启动流程，包括：
//! - 宏配置系统
//! - 入口点系统
//! - CLI 启动路径优化
//! - 初始化流程

pub mod macros;
pub mod entry;
pub mod cli;
pub mod init;

pub use macros::{MacroConfig, ensure_bootstrap_macro};
pub use init::init;
