//! 核心命令模块
//! 
//! 这个模块实现了内置的核心命令

use crate::error::Result;
use crate::commands::types::{
    Command, CommandBase, CommandContext, CommandResult, 
    LocalCommand, LoadedFrom,
};
use crate::commands::registry::CommandExecutor as CmdExecutor;

/// 版本命令
pub struct VersionCommand;

#[async_trait::async_trait]
impl CmdExecutor for VersionCommand {
    async fn execute(&self, _context: CommandContext) -> Result<CommandResult> {
        let version = env!("CARGO_PKG_VERSION");
        Ok(CommandResult {
            content: format!("Claude Code v{} (Rust)", version),
            ..Default::default()
        })
    }
    
    fn command(&self) -> Command {
        Command::Local(LocalCommand {
            base: CommandBase {
                name: "version".to_string(),
                description: "Show version information".to_string(),
                has_user_specified_description: None,
                aliases: Some(vec!["v".to_string(), "V".to_string()]),
                availability: None,
                is_hidden: None,
                is_mcp: None,
                argument_hint: None,
                when_to_use: None,
                version: None,
                disable_model_invocation: None,
                user_invocable: None,
                loaded_from: Some(LoadedFrom::Bundled),
                kind: None,
                immediate: Some(true),
                is_sensitive: None,
            },
            supports_non_interactive: true,
        })
    }
}

/// 帮助命令
pub struct HelpCommand;

#[async_trait::async_trait]
impl CmdExecutor for HelpCommand {
    async fn execute(&self, _context: CommandContext) -> Result<CommandResult> {
        let help_text = r#"
Claude Code - AI-powered coding assistant

Usage: /<command> [arguments]

Available commands:
  /help, /h, /?     Show this help message
  /version, /v, /V  Show version information
  /clear            Clear the screen
  /exit, /quit, /q  Exit the application
  /config           Manage configuration
  /mcp              Manage MCP servers
  /status           Show application status

For more information, visit: https://github.com/anthropics/claude-code
"#;
        
        Ok(CommandResult {
            content: help_text.to_string(),
            ..Default::default()
        })
    }
    
    fn command(&self) -> Command {
        Command::Local(LocalCommand {
            base: CommandBase {
                name: "help".to_string(),
                description: "Show help information".to_string(),
                has_user_specified_description: None,
                aliases: Some(vec!["h".to_string(), "?".to_string()]),
                availability: None,
                is_hidden: None,
                is_mcp: None,
                argument_hint: None,
                when_to_use: None,
                version: None,
                disable_model_invocation: None,
                user_invocable: None,
                loaded_from: Some(LoadedFrom::Bundled),
                kind: None,
                immediate: Some(true),
                is_sensitive: None,
            },
            supports_non_interactive: true,
        })
    }
}

/// 清屏命令
pub struct ClearCommand;

#[async_trait::async_trait]
impl CmdExecutor for ClearCommand {
    async fn execute(&self, _context: CommandContext) -> Result<CommandResult> {
        // 在实际实现中，这里会调用 TUI 的清屏功能
        Ok(CommandResult {
            content: String::new(),
            ..Default::default()
        })
    }
    
    fn command(&self) -> Command {
        Command::Local(LocalCommand {
            base: CommandBase {
                name: "clear".to_string(),
                description: "Clear the screen".to_string(),
                has_user_specified_description: None,
                aliases: None,
                availability: None,
                is_hidden: None,
                is_mcp: None,
                argument_hint: None,
                when_to_use: None,
                version: None,
                disable_model_invocation: None,
                user_invocable: None,
                loaded_from: Some(LoadedFrom::Bundled),
                kind: None,
                immediate: Some(true),
                is_sensitive: None,
            },
            supports_non_interactive: true,
        })
    }
}

/// 退出命令
pub struct ExitCommand;

#[async_trait::async_trait]
impl CmdExecutor for ExitCommand {
    async fn execute(&self, _context: CommandContext) -> Result<CommandResult> {
        Ok(CommandResult {
            content: "Goodbye!".to_string(),
            ..Default::default()
        })
    }
    
    fn command(&self) -> Command {
        Command::Local(LocalCommand {
            base: CommandBase {
                name: "exit".to_string(),
                description: "Exit the application".to_string(),
                has_user_specified_description: None,
                aliases: Some(vec!["quit".to_string(), "q".to_string()]),
                availability: None,
                is_hidden: None,
                is_mcp: None,
                argument_hint: None,
                when_to_use: None,
                version: None,
                disable_model_invocation: None,
                user_invocable: None,
                loaded_from: Some(LoadedFrom::Bundled),
                kind: None,
                immediate: Some(true),
                is_sensitive: None,
            },
            supports_non_interactive: true,
        })
    }
}

/// 配置命令
pub struct ConfigCommand;

#[async_trait::async_trait]
impl CmdExecutor for ConfigCommand {
    async fn execute(&self, context: CommandContext) -> Result<CommandResult> {
        let args = context.args.trim();
        
        if args.is_empty() {
            // 显示当前配置
            let config_info = format!(
                "Current configuration:\n\
                 Model: {}\n\
                 Working directory: {}\n\
                 Verbose: {}",
                context.config.model,
                context.cwd.display(),
                context.config.verbose
            );
            
            Ok(CommandResult {
                content: config_info,
                ..Default::default()
            })
        } else {
            // 解析配置命令
            let parts: Vec<&str> = args.splitn(2, ' ').collect();
            match parts[0] {
                "set" => {
                    if parts.len() < 2 {
                        return Ok(CommandResult {
                            content: "Usage: /config set <key> <value>".to_string(),
                            ..Default::default()
                        });
                    }
                    
                    let kv: Vec<&str> = parts[1].splitn(2, ' ').collect();
                    if kv.len() < 2 {
                        return Ok(CommandResult {
                            content: "Usage: /config set <key> <value>".to_string(),
                            ..Default::default()
                        });
                    }
                    
                    // TODO: 实现配置设置
                    Ok(CommandResult {
                        content: format!("Set {} = {}", kv[0], kv[1]),
                        ..Default::default()
                    })
                }
                "get" => {
                    if parts.len() < 2 {
                        return Ok(CommandResult {
                            content: "Usage: /config get <key>".to_string(),
                            ..Default::default()
                        });
                    }
                    
                    // TODO: 实现配置获取
                    Ok(CommandResult {
                        content: format!("Get configuration: {}", parts[1]),
                        ..Default::default()
                    })
                }
                "list" => {
                    // TODO: 实现配置列表
                    Ok(CommandResult {
                        content: "List all configurations".to_string(),
                        ..Default::default()
                    })
                }
                _ => {
                    Ok(CommandResult {
                        content: format!("Unknown config command: {}", parts[0]),
                        ..Default::default()
                    })
                }
            }
        }
    }
    
    fn command(&self) -> Command {
        Command::Local(LocalCommand {
            base: CommandBase {
                name: "config".to_string(),
                description: "Manage configuration".to_string(),
                has_user_specified_description: None,
                aliases: None,
                availability: None,
                is_hidden: None,
                is_mcp: None,
                argument_hint: Some("[set|get|list] [key] [value]".to_string()),
                when_to_use: None,
                version: None,
                disable_model_invocation: None,
                user_invocable: None,
                loaded_from: Some(LoadedFrom::Bundled),
                kind: None,
                immediate: Some(true),
                is_sensitive: None,
            },
            supports_non_interactive: true,
        })
    }
}

/// MCP 命令
pub struct McpCommand;

#[async_trait::async_trait]
impl CmdExecutor for McpCommand {
    async fn execute(&self, context: CommandContext) -> Result<CommandResult> {
        let args = context.args.trim();
        
        if args.is_empty() {
            // 显示 MCP 服务器列表
            let servers = &context.config.mcp_servers;
            if servers.is_empty() {
                return Ok(CommandResult {
                    content: "No MCP servers configured.".to_string(),
                    ..Default::default()
                });
            }
            
            let mut output = String::from("Configured MCP servers:\n");
            for server in servers {
                output.push_str(&format!("  - {} ({})\n", server.name, server.status));
            }
            
            Ok(CommandResult {
                content: output,
                ..Default::default()
            })
        } else {
            let parts: Vec<&str> = args.splitn(2, ' ').collect();
            match parts[0] {
                "list" => {
                    // 列出服务器
                    Ok(CommandResult {
                        content: "List MCP servers".to_string(),
                        ..Default::default()
                    })
                }
                "enable" => {
                    if parts.len() < 2 {
                        return Ok(CommandResult {
                            content: "Usage: /mcp enable <server-name>".to_string(),
                            ..Default::default()
                        });
                    }
                    
                    Ok(CommandResult {
                        content: format!("Enable MCP server: {}", parts[1]),
                        ..Default::default()
                    })
                }
                "disable" => {
                    if parts.len() < 2 {
                        return Ok(CommandResult {
                            content: "Usage: /mcp disable <server-name>".to_string(),
                            ..Default::default()
                        });
                    }
                    
                    Ok(CommandResult {
                        content: format!("Disable MCP server: {}", parts[1]),
                        ..Default::default()
                    })
                }
                _ => {
                    Ok(CommandResult {
                        content: format!("Unknown MCP command: {}", parts[0]),
                        ..Default::default()
                    })
                }
            }
        }
    }
    
    fn command(&self) -> Command {
        Command::Local(LocalCommand {
            base: CommandBase {
                name: "mcp".to_string(),
                description: "Manage MCP servers".to_string(),
                has_user_specified_description: None,
                aliases: None,
                availability: None,
                is_hidden: None,
                is_mcp: None,
                argument_hint: Some("[list|enable|disable] [server-name]".to_string()),
                when_to_use: None,
                version: None,
                disable_model_invocation: None,
                user_invocable: None,
                loaded_from: Some(LoadedFrom::Bundled),
                kind: None,
                immediate: Some(true),
                is_sensitive: None,
            },
            supports_non_interactive: true,
        })
    }
}

/// 状态命令
pub struct StatusCommand;

#[async_trait::async_trait]
impl CmdExecutor for StatusCommand {
    async fn execute(&self, context: CommandContext) -> Result<CommandResult> {
        let status = format!(
            "Application Status:\n\
             Model: {}\n\
             Working Directory: {}\n\
             Verbose Mode: {}\n\
             Memory Enabled: {}\n\
             Voice Enabled: {}",
            context.config.model,
            context.cwd.display(),
            context.config.verbose,
            context.config.memory.enabled,
            context.config.voice.enabled
        );
        
        Ok(CommandResult {
            content: status,
            ..Default::default()
        })
    }
    
    fn command(&self) -> Command {
        Command::Local(LocalCommand {
            base: CommandBase {
                name: "status".to_string(),
                description: "Show application status".to_string(),
                has_user_specified_description: None,
                aliases: None,
                availability: None,
                is_hidden: None,
                is_mcp: None,
                argument_hint: None,
                when_to_use: None,
                version: None,
                disable_model_invocation: None,
                user_invocable: None,
                loaded_from: Some(LoadedFrom::Bundled),
                kind: None,
                immediate: Some(true),
                is_sensitive: None,
            },
            supports_non_interactive: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_command() {
        let cmd = VersionCommand;
        let command = cmd.command();
        
        assert_eq!(command.name(), "version");
        assert_eq!(command.description(), "Show version information");
    }
    
    #[test]
    fn test_help_command() {
        let cmd = HelpCommand;
        let command = cmd.command();
        
        assert_eq!(command.name(), "help");
        assert_eq!(command.description(), "Show help information");
    }
}
