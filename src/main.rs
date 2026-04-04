//! Claude Code - AI-powered coding assistant
//! 
//! Main entry point for the Claude Code CLI application.

use clap::{Parser, Subcommand};
use claude_code_rs::{commands, config, error::Result, state};
use std::process;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Claude Code - AI-powered coding assistant
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Subcommands
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
    
    /// Configuration file path
    #[arg(long, global = true)]
    config: Option<std::path::PathBuf>,
}

/// Claude Code subcommands
#[derive(Subcommand, Debug)]
enum Commands {
    /// Start interactive session
    #[command(alias = "i")]
    Interactive,
    
    /// Run a single query
    #[command(alias = "q")]
    Query {
        /// Query text
        query: Vec<String>,
    },
    
    /// Configure Claude Code
    Config {
        /// Config key
        #[arg(value_name = "KEY")]
        key: Option<String>,
        
        /// Config value
        #[arg(value_name = "VALUE")]
        value: Option<String>,
    },
    
    /// Login to Claude Code
    Login,
    
    /// Logout from Claude Code
    Logout,
    
    /// Show version information
    Version,

    /// Bridge mode (remote control)
    #[cfg(feature = "bridge")]
    Bridge,
    
    /// MCP commands
    #[cfg(feature = "mcp-support")]
    Mcp {
        #[command(subcommand)]
        subcommand: McpCommands,
    },
    
    /// Voice mode
    #[cfg(feature = "voice")]
    Voice,
    
    /// Plugin commands
    #[cfg(feature = "plugins")]
    Plugins {
        #[command(subcommand)]
        subcommand: PluginCommands,
    },
    
    /// Analytics commands
    Analytics {
        #[command(subcommand)]
        subcommand: AnalyticsCommands,
    },
    
    /// Run as daemon
    #[cfg(feature = "daemon")]
    Daemon,
}

/// Analytics subcommands
#[derive(Subcommand, Debug)]
enum AnalyticsCommands {
    /// Show performance report
    Performance,
}

/// Plugin subcommands
#[derive(Subcommand, Debug)]
enum PluginCommands {
    /// List all plugins
    List,
    
    /// Load a plugin
    Load {
        /// Plugin path
        path: String,
    },
    
    /// Unload a plugin
    Unload {
        /// Plugin name
        name: String,
    },
    
    /// Start a plugin
    Start {
        /// Plugin name
        name: String,
    },
    
    /// Stop a plugin
    Stop {
        /// Plugin name
        name: String,
    },
    
    /// Scan for plugins
    Scan,
}

/// MCP subcommands
#[derive(Subcommand, Debug)]
enum McpCommands {
    /// List MCP servers
    List,
    
    /// Enable MCP server
    Enable {
        server_name: String,
    },
    
    /// Disable MCP server
    Disable {
        server_name: String,
    },
    
    /// Reconnect MCP server
    Reconnect {
        server_name: String,
    },
}

#[tokio::main]
async fn main() {
    // Initialize logging
    if let Err(e) = init_logging() {
        eprintln!("Warning: Failed to initialize logging: {}", e);
    }
    
    // Parse CLI arguments
    let cli = Cli::parse();
    
    // Run the application
    if let Err(e) = run(cli).await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

/// Initialize logging
fn init_logging() -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .try_init()
        .map_err(|e| format!("Failed to initialize logging: {}", e))?;
    
    Ok(())
}

/// Run the application
async fn run(cli: Cli) -> Result<()> {
    // Show version if requested
    if let Some(Commands::Version) = cli.command {
        println!("{} ({}) [Rust]", env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_NAME"));
        return Ok(());
    }
    
    // Load configuration
    let settings = config::Settings::load()
        .map_err(|e| format!("Failed to load configuration: {}", e))?;
    
    // Create application state
    let state = state::new_app_state();
    
    // Handle commands
    match cli.command {
        Some(Commands::Interactive) | None => {
            commands::interactive::run(settings, state).await?;
        }
        
        Some(Commands::Query { query }) => {
            let query_text = query.join(" ");
            commands::query::run(query_text, settings, state).await?;
        }
        
        Some(Commands::Config { key, value }) => {
            commands::config::run(key, value, settings).await?;
        }
        
        Some(Commands::Login) => {
            commands::auth::login(settings).await?;
        }
        
        Some(Commands::Logout) => {
            commands::auth::logout(settings).await?;
        }
        
        #[cfg(feature = "bridge")]
        Some(Commands::Bridge) => {
            commands::bridge::run(state).await?;
        }
        
        #[cfg(feature = "mcp-support")]
        Some(Commands::Mcp { subcommand }) => {
            match subcommand {
                McpCommands::List => {
                    commands::mcp::list_servers(state).await?;
                }
                McpCommands::Enable { server_name } => {
                    commands::mcp::enable_server(server_name, state).await?;
                }
                McpCommands::Disable { server_name } => {
                    commands::mcp::disable_server(server_name, state).await?;
                }
                McpCommands::Reconnect { server_name } => {
                    commands::mcp::reconnect_server(server_name, state).await?;
                }
            }
        }
        
        #[cfg(feature = "voice")]
        Some(Commands::Voice) => {
            commands::voice::run(state).await?;
        }
        
        #[cfg(feature = "plugins")]
        Some(Commands::Plugins { subcommand }) => {
            match subcommand {
                PluginCommands::List => {
                    commands::plugins::list_plugins(state).await?;
                }
                PluginCommands::Load { path } => {
                    commands::plugins::load_plugin(path, state).await?;
                }
                PluginCommands::Unload { name } => {
                    commands::plugins::unload_plugin(name, state).await?;
                }
                PluginCommands::Start { name } => {
                    commands::plugins::start_plugin(name, state).await?;
                }
                PluginCommands::Stop { name } => {
                    commands::plugins::stop_plugin(name, state).await?;
                }
                PluginCommands::Scan => {
                    commands::plugins::scan_plugins(state).await?;
                }
            }
        }
        
        Some(Commands::Analytics { subcommand }) => {
            match subcommand {
                AnalyticsCommands::Performance => {
                    commands::analytics::show_performance_report(state).await?;
                }
            }
        }
        
        #[cfg(feature = "daemon")]
        Some(Commands::Daemon) => {
            commands::daemon::run(state).await?;
        }
        
        Some(Commands::Version) => {
            // Already handled above
        }
    }
    
    Ok(())
}

/// Print help information
fn print_help() {
    println!("Claude Code - AI-powered coding assistant");
    println!();
    println!("Usage:");
    println!("  claude [COMMAND]");
    println!();
    println!("Commands:");
    println!("  interactive, i    Start interactive session (default)");
    println!("  query, q          Run a single query");
    println!("  config            Configure Claude Code");
    println!("  login             Login to Claude Code");
    println!("  logout            Logout from Claude Code");
    println!("  version           Show version information");
    println!("  help              Show this help message");
    println!();
    #[cfg(feature = "bridge")]
    println!("  bridge            Run in bridge mode (remote control)");
    #[cfg(feature = "mcp-support")]
    println!("  mcp               MCP server management commands");
    #[cfg(feature = "voice")]
    println!("  voice             Voice interaction mode");
    #[cfg(feature = "plugins")]
    println!("  plugins           Plugin management commands");
    println!("  analytics         Analytics and performance commands");
    #[cfg(feature = "daemon")]
    println!("  daemon            Run as daemon service");
}
