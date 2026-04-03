//! Claude Code - AI-powered coding assistant (Rust implementation)
//! 
//! This is a Rust port of the Claude Code project, providing the same
//! AI-assisted coding capabilities with improved performance and type safety.

#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]
#![forbid(unsafe_code)]

pub mod commands;
pub mod config;
pub mod error;
pub mod tools;
pub mod state;
pub mod bridge;
pub mod mcp;
pub mod agents;
pub mod analytics;
pub mod utils;
pub mod voice;
pub mod daemon;
 pub mod features;
pub mod bootstrap;
pub mod services;
pub mod performance;
pub mod security;

// Re-export commonly used types
pub use error::{ClaudeError, Result};
pub use state::AppState;

// Re-export bootstrap types
pub use bootstrap::macros::{MacroConfig, ensure_bootstrap_macro, get_macro_config};
pub use bootstrap::init::init;

// Re-export security types
pub use security::{SecurityManager, PermissionManager, SandboxManager, AuditLogger};
