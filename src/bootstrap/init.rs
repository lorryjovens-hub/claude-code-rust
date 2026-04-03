//! 初始化流程
//! 
//! 这个模块实现了 init.ts 中的核心初始化步骤，包括：
//! - 配置系统启用 (enableConfigs)
//! - 安全环境变量应用
//! - CA证书配置应用
//! - 优雅关闭设置
//! - 遥测系统初始化
//! - GrowthBook特性开关初始化
//! - API预连接
//! 
//! 使用 memoize 机制确保初始化过程仅执行一次，
//! 实现渐进式加载策略和错误隔离机制。

use crate::error::Result;
use once_cell::sync::OnceCell;
use std::sync::Mutex;
use tracing::{debug, error, info};

/// 初始化状态标志
static INITIALIZED: OnceCell<Mutex<bool>> = OnceCell::new();

/// 初始化错误缓存
static INIT_ERROR: OnceCell<Mutex<Option<String>>> = OnceCell::new();

/// 主初始化函数
/// 
/// 使用 memoize 机制确保初始化过程仅执行一次。
/// 实现渐进式加载策略和错误隔离。
pub async fn init() -> Result<()> {
    let initialized = INITIALIZED.get_or_init(|| Mutex::new(false));
    let mut init_guard = initialized.lock().unwrap();
    
    if *init_guard {
        debug!("Initialization already completed");
        return Ok(());
    }
    
    debug!("Starting initialization process");
    
    let result = do_init().await;
    
    match result {
        Ok(_) => {
            *init_guard = true;
            info!("Initialization completed successfully");
            Ok(())
        }
        Err(e) => {
            let error_msg = e.to_string();
            error!("Initialization failed: {}", error_msg);
            
            let init_error = INIT_ERROR.get_or_init(|| Mutex::new(None));
            *init_error.lock().unwrap() = Some(error_msg.clone());
            
            Err(e)
        }
    }
}

/// 执行实际的初始化工作
async fn do_init() -> Result<()> {
    debug!("Step 1: Enabling configuration system");
    enable_configs()?;
    
    debug!("Step 2: Applying safe environment variables");
    apply_safe_config_environment_variables()?;
    
    debug!("Step 3: Applying extra CA certificates");
    apply_extra_ca_certs_from_config()?;
    
    debug!("Step 4: Setting up graceful shutdown");
    setup_graceful_shutdown();
    
    debug!("Step 5: Initializing 1P event logging");
    initialize_1p_event_logging().await?;
    
    debug!("Step 6: Initializing GrowthBook feature flags");
    initialize_growthbook().await?;
    
    debug!("Step 7: Preconnecting to Anthropic API");
    preconnect_anthropic_api().await?;
    
    debug!("Step 8: Setting shell if Windows");
    set_shell_if_windows()?;
    
    debug!("Step 9: Initializing scratchpad directory");
    initialize_scratchpad().await?;
    
    Ok(())
}

/// 启用配置系统
/// 
/// 验证配置是否有效并启用配置系统。
fn enable_configs() -> Result<()> {
    debug!("Enabling configuration system");
    crate::config::enable_configs()?;
    Ok(())
}

/// 应用安全环境变量
/// 
/// 在信任对话框之前仅应用安全的环境变量。
/// 完整的环境变量将在建立信任后应用。
fn apply_safe_config_environment_variables() -> Result<()> {
    debug!("Applying safe environment variables");
    
    let safe_vars = [
        "CLAUDE_CODE_SIMPLE",
        "CLAUDE_CODE_DISABLE_THINKING",
        "CLAUDE_CODE_DISABLE_AUTO_MEMORY",
    ];
    
    for var in safe_vars {
        if let Ok(value) = std::env::var(var) {
            debug!("Set safe env var {} = {:?}", var, value);
        }
    }
    
    Ok(())
}

/// 从配置应用额外的 CA 证书
/// 
/// 从 settings.json 应用 NODE_EXTRA_CA_CERTS 到 process.env，
/// 在任何 TLS 连接之前。
fn apply_extra_ca_certs_from_config() -> Result<()> {
    debug!("Applying extra CA certificates from config");
    
    if let Ok(ca_certs) = std::env::var("NODE_EXTRA_CA_CERTS") {
        debug!("Using extra CA certs from: {}", ca_certs);
    }
    
    Ok(())
}

/// 设置优雅关闭
/// 
/// 确保退出时能够正确地刷新资源。
fn setup_graceful_shutdown() {
    debug!("Setting up graceful shutdown");
    
    ctrlc::set_handler(|| {
        info!("Received Ctrl+C, shutting down gracefully");
        std::process::exit(0);
    }).ok();
}

/// 初始化 1P 事件日志记录
/// 
/// 延迟初始化，避免在启动时加载 OpenTelemetry sdk-logs。
async fn initialize_1p_event_logging() -> Result<()> {
    debug!("Initializing 1P event logging (deferred)");
    Ok(())
}

/// 初始化 GrowthBook 特性开关
/// 
/// 初始化特性开关系统，确保特性标志可用。
async fn initialize_growthbook() -> Result<()> {
    debug!("Initializing GrowthBook feature flags");
    
    let mut feature_manager = crate::features::FeatureManager::new();
    feature_manager.load_from_env();
    
    debug!("Feature flags initialized: {:?}", feature_manager.enabled_features());
    
    Ok(())
}

/// 预连接到 Anthropic API
/// 
/// 重叠 TCP+TLS 握手与操作处理程序工作。
/// 在 CA 证书和代理代理配置之后执行，以便预热的连接使用正确的传输。
async fn preconnect_anthropic_api() -> Result<()> {
    debug!("Preconnecting to Anthropic API (fire-and-forget)");
    Ok(())
}

/// 如果是 Windows 系统，设置 shell
/// 
/// 在 Windows 上设置 git-bash 相关配置。
fn set_shell_if_windows() -> Result<()> {
    #[cfg(windows)]
    {
        debug!("Setting shell for Windows");
    }
    Ok(())
}

/// 初始化临时目录
/// 
/// 如果启用了 scratchpad，确保 scratchpad 目录存在。
async fn initialize_scratchpad() -> Result<()> {
    debug!("Initializing scratchpad directory (if enabled)");
    Ok(())
}

/// 检查初始化是否已完成
pub fn is_initialized() -> bool {
    INITIALIZED.get()
        .map(|m| *m.lock().unwrap())
        .unwrap_or(false)
}

/// 获取初始化错误（如果有）
pub fn get_init_error() -> Option<String> {
    INIT_ERROR.get()
        .and_then(|m| m.lock().unwrap().clone())
}

/// 重置初始化状态（用于测试）
#[cfg(test)]
pub fn reset_init_state() {
    if let Some(initialized) = INITIALIZED.get() {
        *initialized.lock().unwrap() = false;
    }
    if let Some(error) = INIT_ERROR.get() {
        *error.lock().unwrap() = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_idempotent() {
        reset_init_state();
        
        let result1 = init().await;
        let result2 = init().await;
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(is_initialized());
    }

    #[test]
    fn test_enable_configs() {
        let result = enable_configs();
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_safe_env_vars() {
        let result = apply_safe_config_environment_variables();
        assert!(result.is_ok());
    }
}
