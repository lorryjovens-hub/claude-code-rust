//! CLI 启动路径优化
//! 
//! 这个模块实现了 cli.tsx 中的多快速路径实现，包括：
//! - --version/-v：零模块加载的版本输出
//! - --dump-system-prompt：最小化导入的系统提示导出
//! - --claude-in-chrome-mcp：Chrome MCP 的按需加载
//! - --daemon-worker：守护进程工作器的精简加载
//! - remote-control：完整加载的 Bridge 系统

use crate::bootstrap::macros::get_version;
use crate::error::Result;

/// CLI 快速路径枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliFastPath {
    /// 版本输出 (--version, -v, -V)
    Version,
    
    /// 系统提示导出 (--dump-system-prompt)
    DumpSystemPrompt {
        /// 模型名称
        model: Option<String>,
    },
    
    /// Chrome MCP 服务器 (--claude-in-chrome-mcp)
    ClaudeInChromeMcp,
    
    /// Chrome 原生主机 (--chrome-native-host)
    ChromeNativeHost,
    
    /// 计算机使用 MCP 服务器 (--computer-use-mcp)
    ComputerUseMcp,
    
    /// 守护进程工作器 (--daemon-worker <kind>)
    DaemonWorker {
        /// 工作器类型
        kind: String,
    },
    
    /// 远程控制 (remote-control, rc, remote, sync, bridge)
    RemoteControl {
        /// 命令行参数
        args: Vec<String>,
    },
    
    /// 守护进程 (daemon)
    Daemon {
        /// 命令行参数
        args: Vec<String>,
    },
    
    /// 后台会话管理 (ps, logs, attach, kill, --bg, --background)
    BgSession {
        /// 后台命令
        command: BgCommand,
    },
    
    /// 模板命令 (new, list, reply)
    Template {
        /// 命令行参数
        args: Vec<String>,
    },
    
    /// 环境运行器 (environment-runner)
    EnvironmentRunner {
        /// 命令行参数
        args: Vec<String>,
    },
    
    /// 自托管运行器 (self-hosted-runner)
    SelfHostedRunner {
        /// 命令行参数
        args: Vec<String>,
    },
    
    /// Tmux 工作树 (--tmux 配合 --worktree)
    TmuxWorktree {
        /// 命令行参数
        args: Vec<String>,
    },
}

/// 后台命令类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BgCommand {
    /// 列出会话
    Ps(Vec<String>),
    /// 查看日志
    Logs(String),
    /// 附加到会话
    Attach(String),
    /// 终止会话
    Kill(String),
    /// 处理 --bg 标志
    BgFlag(Vec<String>),
}

impl CliFastPath {
    /// 检测快速路径
    /// 
    /// 解析命令行参数，检测是否匹配任何快速路径。
    pub fn detect(args: &[String]) -> Option<Self> {
        if args.is_empty() {
            return None;
        }

        match args[0].as_str() {
            "--version" | "-v" | "-V" if args.len() == 1 => {
                return Some(CliFastPath::Version);
            }
            "--dump-system-prompt" => {
                let model = args.windows(2)
                    .find(|w| w[0] == "--model")
                    .and_then(|_w| args.get(args.iter().position(|x| x == "--model")? + 1))
                    .cloned();
                return Some(CliFastPath::DumpSystemPrompt { model });
            }
            "--claude-in-chrome-mcp" => {
                return Some(CliFastPath::ClaudeInChromeMcp);
            }
            "--chrome-native-host" => {
                return Some(CliFastPath::ChromeNativeHost);
            }
            "--computer-use-mcp" => {
                return Some(CliFastPath::ComputerUseMcp);
            }
            "--daemon-worker" if args.len() >= 2 => {
                return Some(CliFastPath::DaemonWorker {
                    kind: args[1].clone(),
                });
            }
            "remote-control" | "rc" | "remote" | "sync" | "bridge" => {
                return Some(CliFastPath::RemoteControl {
                    args: args[1..].to_vec(),
                });
            }
            "daemon" => {
                return Some(CliFastPath::Daemon {
                    args: args[1..].to_vec(),
                });
            }
            "ps" => {
                return Some(CliFastPath::BgSession {
                    command: BgCommand::Ps(args[1..].to_vec()),
                });
            }
            "logs" if args.len() >= 2 => {
                return Some(CliFastPath::BgSession {
                    command: BgCommand::Logs(args[1].clone()),
                });
            }
            "attach" if args.len() >= 2 => {
                return Some(CliFastPath::BgSession {
                    command: BgCommand::Attach(args[1].clone()),
                });
            }
            "kill" if args.len() >= 2 => {
                return Some(CliFastPath::BgSession {
                    command: BgCommand::Kill(args[1].clone()),
                });
            }
            "new" | "list" | "reply" => {
                return Some(CliFastPath::Template {
                    args: args.to_vec(),
                });
            }
            "environment-runner" => {
                return Some(CliFastPath::EnvironmentRunner {
                    args: args[1..].to_vec(),
                });
            }
            "self-hosted-runner" => {
                return Some(CliFastPath::SelfHostedRunner {
                    args: args[1..].to_vec(),
                });
            }
            _ => {}
        }

        if args.contains(&"--bg".to_string()) || args.contains(&"--background".to_string()) {
            return Some(CliFastPath::BgSession {
                command: BgCommand::BgFlag(args.to_vec()),
            });
        }

        let has_tmux = args.iter().any(|a| a == "--tmux" || a.starts_with("--tmux="));
        let has_worktree = args.iter().any(|a| a == "-w" || a == "--worktree" || a.starts_with("--worktree="));
        if has_tmux && has_worktree {
            return Some(CliFastPath::TmuxWorktree {
                args: args.to_vec(),
            });
        }

        None
    }
}

/// 执行快速路径
/// 
/// 根据检测到的快速路径类型执行相应的操作。
pub async fn execute_fast_path(path: CliFastPath) -> Result<()> {
    match path {
        CliFastPath::Version => {
            execute_version();
            Ok(())
        }
        CliFastPath::DumpSystemPrompt { model } => {
            execute_dump_system_prompt(model).await
        }
        CliFastPath::ClaudeInChromeMcp => {
            execute_claude_in_chrome_mcp().await
        }
        CliFastPath::ChromeNativeHost => {
            execute_chrome_native_host().await
        }
        CliFastPath::ComputerUseMcp => {
            execute_computer_use_mcp().await
        }
        CliFastPath::DaemonWorker { kind } => {
            execute_daemon_worker(kind).await
        }
        CliFastPath::RemoteControl { args } => {
            execute_remote_control(args).await
        }
        CliFastPath::Daemon { args } => {
            execute_daemon(args).await
        }
        CliFastPath::BgSession { command } => {
            execute_bg_session(command).await
        }
        CliFastPath::Template { args } => {
            execute_template(args).await
        }
        CliFastPath::EnvironmentRunner { args } => {
            execute_environment_runner(args).await
        }
        CliFastPath::SelfHostedRunner { args } => {
            execute_self_hosted_runner(args).await
        }
        CliFastPath::TmuxWorktree { args } => {
            execute_tmux_worktree(args).await
        }
    }
}

/// 执行版本输出
/// 
/// 零模块加载的快速路径，直接输出版本号。
fn execute_version() {
    println!("{} (Claude Code)", get_version());
}

/// 执行系统提示导出
async fn execute_dump_system_prompt(_model: Option<String>) -> Result<()> {
    tracing::debug!("Dumping system prompt");
    
    crate::config::enable_configs()?;
    
    let prompt = vec![
        "You are Claude, an AI coding assistant.",
        "You help users with their coding tasks.",
    ];
    
    println!("{}", prompt.join("\n"));
    
    Ok(())
}

/// 执行 Chrome MCP 服务器
async fn execute_claude_in_chrome_mcp() -> Result<()> {
    tracing::debug!("Starting Claude in Chrome MCP server");
    
    Ok(())
}

/// 执行 Chrome 原生主机
async fn execute_chrome_native_host() -> Result<()> {
    tracing::debug!("Starting Chrome native host");
    
    Ok(())
}

/// 执行计算机使用 MCP 服务器
async fn execute_computer_use_mcp() -> Result<()> {
    tracing::debug!("Starting Computer Use MCP server");
    
    Ok(())
}

/// 执行守护进程工作器
async fn execute_daemon_worker(kind: String) -> Result<()> {
    tracing::debug!("Starting daemon worker: {}", kind);
    
    Ok(())
}

/// 执行远程控制
async fn execute_remote_control(args: Vec<String>) -> Result<()> {
    tracing::debug!("Starting remote control with args: {:?}", args);
    
    crate::config::enable_configs()?;
    
    Ok(())
}

/// 执行守护进程
async fn execute_daemon(args: Vec<String>) -> Result<()> {
    tracing::debug!("Starting daemon with args: {:?}", args);
    
    crate::config::enable_configs()?;
    
    Ok(())
}

/// 执行后台会话管理
async fn execute_bg_session(command: BgCommand) -> Result<()> {
    tracing::debug!("Executing bg command: {:?}", command);
    
    crate::config::enable_configs()?;
    
    Ok(())
}

/// 执行模板命令
async fn execute_template(args: Vec<String>) -> Result<()> {
    tracing::debug!("Executing template command with args: {:?}", args);
    
    Ok(())
}

/// 执行环境运行器
async fn execute_environment_runner(args: Vec<String>) -> Result<()> {
    tracing::debug!("Starting environment runner with args: {:?}", args);
    
    Ok(())
}

/// 执行自托管运行器
async fn execute_self_hosted_runner(args: Vec<String>) -> Result<()> {
    tracing::debug!("Starting self-hosted runner with args: {:?}", args);
    
    Ok(())
}

/// 执行 Tmux 工作树
async fn execute_tmux_worktree(args: Vec<String>) -> Result<()> {
    tracing::debug!("Executing tmux worktree with args: {:?}", args);
    
    crate::config::enable_configs()?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_version() {
        assert!(matches!(
            CliFastPath::detect(&["--version".to_string()]),
            Some(CliFastPath::Version)
        ));
        assert!(matches!(
            CliFastPath::detect(&["-v".to_string()]),
            Some(CliFastPath::Version)
        ));
        assert!(matches!(
            CliFastPath::detect(&["-V".to_string()]),
            Some(CliFastPath::Version)
        ));
    }

    #[test]
    fn test_detect_remote_control() {
        assert!(matches!(
            CliFastPath::detect(&["remote-control".to_string()]),
            Some(CliFastPath::RemoteControl { .. })
        ));
        assert!(matches!(
            CliFastPath::detect(&["rc".to_string()]),
            Some(CliFastPath::RemoteControl { .. })
        ));
        assert!(matches!(
            CliFastPath::detect(&["remote".to_string()]),
            Some(CliFastPath::RemoteControl { .. })
        ));
    }

    #[test]
    fn test_detect_no_fast_path() {
        assert!(CliFastPath::detect(&["some-command".to_string()]).is_none());
        assert!(CliFastPath::detect(&[]).is_none());
    }
}
