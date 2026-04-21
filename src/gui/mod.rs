//! GUI Module - Desktop GUI using egui/eframe
//!
//! This module provides a native desktop GUI for Claude Code
//! with a modern, responsive interface.

pub mod app;
pub mod chat;
pub mod sidebar;
pub mod settings;
pub mod theme;
pub mod syntax_highlight;
pub mod tool_calls;

pub use app::ClaudeCodeApp;
pub use theme::Theme;

/// Async message type for GUI communication
#[derive(Debug, Clone)]
pub enum GuiMessage {
    /// Send a chat message to the API
    SendMessage { messages: Vec<crate::api::ChatMessage> },
    /// Received a chunk of streaming response
    StreamChunk { content: String, done: bool },
    /// API error occurred
    ApiError { error: String },
    /// Test connection result
    TestConnectionResult { success: bool, message: String },
    /// Settings updated
    SettingsUpdated,
}
