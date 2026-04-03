//! 宏配置系统
//! 
//! 这个模块实现了构建时元数据注入机制，提供运行时零配置的元数据管理。

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// 宏配置类型
/// 
/// 包含构建时注入的元数据，支持运行时零配置访问。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroConfig {
    /// 版本号
    pub version: String,
    
    /// 构建时间
    pub build_time: String,
    
    /// 包 URL
    pub package_url: String,
    
    /// 原生包 URL
    pub native_package_url: String,
    
    /// 版本变更日志
    pub version_changelog: String,
    
    /// 问题说明
    pub issues_explainer: String,
    
    /// 反馈渠道
    pub feedback_channel: String,
}

impl Default for MacroConfig {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_time: String::new(),
            package_url: env!("CARGO_PKG_NAME").to_string(),
            native_package_url: env!("CARGO_PKG_NAME").to_string(),
            version_changelog: String::new(),
            issues_explainer: "file an issue at https://github.com/anthropics/claude-code/issues".to_string(),
            feedback_channel: "github".to_string(),
        }
    }
}

/// 全局宏配置实例
static MACRO_CONFIG: OnceCell<Mutex<MacroConfig>> = OnceCell::new();

/// 确保宏配置已初始化
/// 
/// 这个函数确保全局宏配置已设置，使用默认值（如果尚未设置）。
/// 类似于 TypeScript 版本的 `ensureBootstrapMacro()`。
pub fn ensure_bootstrap_macro() {
    MACRO_CONFIG.get_or_init(|| Mutex::new(MacroConfig::default()));
}

/// 获取全局宏配置
/// 
/// 返回当前的宏配置。如果尚未初始化，会自动初始化。
pub fn get_macro_config() -> MacroConfig {
    ensure_bootstrap_macro();
    MACRO_CONFIG.get().unwrap().lock().unwrap().clone()
}

/// 设置全局宏配置
/// 
/// 允许在运行时覆盖宏配置。主要用于测试和特殊场景。
pub fn set_macro_config(config: MacroConfig) {
    ensure_bootstrap_macro();
    *MACRO_CONFIG.get().unwrap().lock().unwrap() = config;
}

/// 获取版本号
/// 
/// 零依赖快速路径获取版本号。
pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_macro_config() {
        let config = MacroConfig::default();
        assert!(!config.version.is_empty());
        assert_eq!(config.issues_explainer, "file an issue at https://github.com/anthropics/claude-code/issues");
    }

    #[test]
    fn test_ensure_bootstrap_macro() {
        ensure_bootstrap_macro();
        let config = get_macro_config();
        assert!(!config.version.is_empty());
    }

    #[test]
    fn test_get_version() {
        let version = get_version();
        assert!(!version.is_empty());
    }
}
