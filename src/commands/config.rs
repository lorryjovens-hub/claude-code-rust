//! 配置命令
//! 
//! 这个模块实现了配置管理功能

use crate::config::Settings;
use crate::error::Result;

/// 运行配置命令
pub async fn run(key: Option<String>, value: Option<String>, settings: Settings) -> Result<()> {
    match (key, value) {
        (None, None) => {
            // 显示所有配置
            println!("Current configuration:");
            if let Some(api_key) = settings.api.get_api_key() {
                let masked = if api_key.len() > 8 {
                    format!("{}...", &api_key[..8])
                } else {
                    "(set)".to_string()
                };
                println!("  API Key: {}", masked);
            } else {
                println!("  API Key: (not set)");
            }
            println!("  Model: {}", settings.model);
        }
        (Some(key), None) => {
            // 显示特定配置
            match key.as_str() {
                "api_key" => {
                    if let Some(api_key) = settings.api.get_api_key() {
                        let masked = if api_key.len() > 8 {
                            format!("{}...", &api_key[..8])
                        } else {
                            "(set)".to_string()
                        };
                        println!("api_key: {}", masked);
                    } else {
                        println!("api_key: (not set)");
                    }
                }
                "model" => {
                    println!("model: {}", settings.model);
                }
                "base_url" => {
                    println!("base_url: {}", settings.api.get_base_url());
                }
                _ => {
                    println!("Unknown config key: {}", key);
                    println!("Available keys: api_key, model, base_url");
                }
            }
        }
        (Some(key), Some(value)) => {
            // 设置配置
            println!("Setting {} = {}", key, value);
            // TODO: 实际保存配置
            println!("(Configuration saving not yet implemented)");
        }
        (None, Some(_)) => {
            println!("Error: Cannot set value without key");
        }
    }
    
    Ok(())
}
