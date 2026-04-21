//! CLI Module - Command Line Interface

pub mod args;
pub mod commands;
pub mod repl;
pub mod ui;

pub use args::Cli;
pub use repl::Repl;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Claude Code - AI-powered coding assistant
#[derive(Parser, Debug)]
#[command(name = "claude-code")]
#[command(author = "Anthropic")]
#[command(version = "0.1.0")]
#[command(about = "High-performance Rust implementation of Claude Code CLI")]
#[command(disable_version_flag = true)]
#[command(disable_help_subcommand = true)]
pub struct CliArgs {
    /// Path to the project directory
    #[arg(short, long, value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Model to use (sonnet, opus, haiku)
    #[arg(short, long, default_value = "sonnet")]
    pub model: String,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Run in non-interactive mode
    #[arg(short, long)]
    pub no_interactive: bool,

    /// Print version information
    #[arg(long)]
    pub version: bool,

    /// Print system information
    #[arg(long)]
    pub info: bool,

    /// Subcommands
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start an interactive REPL session
    Repl {
        /// Initial prompt to send
        #[arg(short, long)]
        prompt: Option<String>,
    },

    /// Execute a single query
    Query {
        /// The query to execute
        #[arg(short, long)]
        prompt: String,
    },

    /// Manage configuration settings
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },

    /// Manage MCP servers
    Mcp {
        #[command(subcommand)]
        action: McpCommands,
    },

    /// Manage plugins
    Plugin {
        #[command(subcommand)]
        action: PluginCommands,
    },

    /// Manage memory and sessions
    Memory {
        #[command(subcommand)]
        action: MemoryCommands,
    },

    /// Voice input mode
    Voice {
        /// Enable push-to-talk mode
        #[arg(short, long)]
        push_to_talk: bool,
    },

    /// Initialize a new project
    Init {
        /// Project name
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Update to latest version
    Update,

    /// Show help and usage information
    Help {
        /// Topic to show help for
        #[arg(short, long)]
        topic: Option<String>,
    },

    /// Manage background services
    Services {
        #[command(subcommand)]
        action: ServiceCommands,
    },

    /// Run an agent
    Agent {
        /// Agent type (guide, explore, plan, verify, general)
        #[arg(short, long)]
        agent_type: String,
        /// Prompt for the agent
        #[arg(short, long)]
        prompt: String,
    },

    /// Manage Magic Docs
    MagicDocs {
        #[command(subcommand)]
        action: MagicDocsCommands,
    },

    /// Team memory sync
    TeamSync {
        #[command(subcommand)]
        action: TeamSyncCommands,
    },

    /// Manage skills
    Skills {
        #[command(subcommand)]
        action: SkillsCommands,
    },

    /// Run stress tests
    StressTest {
        /// Number of concurrent requests
        #[arg(short, long, default_value = "5")]
        concurrency: usize,
        /// Number of iterations per request
        #[arg(short, long, default_value = "10")]
        iterations: usize,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,

    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },

    /// Reset configuration to defaults
    Reset,
}

#[derive(Subcommand, Debug)]
pub enum McpCommands {
    /// List configured MCP servers
    List,

    /// Add a new MCP server
    Add {
        /// Server name
        name: String,
        /// Server command
        command: String,
    },

    /// Remove an MCP server
    Remove {
        /// Server name
        name: String,
    },

    /// Restart an MCP server
    Restart {
        /// Server name
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum PluginCommands {
    /// List installed plugins
    List,

    /// Install a plugin
    Install {
        /// Plugin name or URL
        plugin: String,
    },

    /// Remove a plugin
    Remove {
        /// Plugin name
        name: String,
    },

    /// Update all plugins
    Update,

    /// Search for plugins
    Search {
        /// Search query
        query: String,
    },

    /// Enable a plugin
    Enable {
        /// Plugin name
        name: String,
    },

    /// Disable a plugin
    Disable {
        /// Plugin name
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum MemoryCommands {
    /// Show memory status
    Status,

    /// Clear all memories
    Clear,

    /// Export memories
    Export {
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Import memories
    Import {
        /// Input file path
        input: PathBuf,
    },

    /// Run memory consolidation (dream)
    Dream,

    /// Force AutoDream consolidation
    AutoDream,
}

#[derive(Subcommand, Debug)]
pub enum ServiceCommands {
    /// Show status of all services
    Status,

    /// Start all services
    Start,

    /// Stop all services
    Stop,

    /// Check AutoDream status
    AutoDream,

    /// Check Voice status
    Voice,

    /// Check Magic Docs status
    MagicDocs,

    /// Check Team Sync status
    TeamSync,

    /// Check Plugins status
    Plugins,

    /// Check Agents status
    Agents,
}

#[derive(Subcommand, Debug)]
pub enum MagicDocsCommands {
    /// List tracked Magic Docs
    List,

    /// Check a file for Magic Doc header
    Check {
        /// File path to check
        file: String,
    },

    /// Update a Magic Doc
    Update {
        /// File path to update
        file: String,
        /// Context for update
        #[arg(short, long)]
        context: Option<String>,
    },

    /// Clear all tracked Magic Docs
    Clear,
}

#[derive(Subcommand, Debug)]
pub enum TeamSyncCommands {
    /// Show sync status
    Status,

    /// Authenticate with team
    Auth {
        /// Team ID
        team_id: String,
    },

    /// Sync memories
    Sync,

    /// List team memories
    List,

    /// Create a team memory
    Create {
        /// Memory title
        title: String,
        /// Memory content
        #[arg(short, long)]
        content: String,
        /// Tags (comma-separated)
        #[arg(short, long)]
        tags: Option<String>,
    },

    /// Delete a team memory
    Delete {
        /// Memory ID
        id: String,
    },
}


#[derive(Subcommand, Debug)]
pub enum SkillsCommands {
    /// List all available skills
    List,

    /// Execute a skill
    Execute {
        /// Skill name
        skill: String,
        /// Arguments for the skill
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Get help for a skill
    Help {
        /// Skill name
        skill: String,
    },

    /// Search for skills
    Search {
        /// Search query
        query: String,
    },
}
