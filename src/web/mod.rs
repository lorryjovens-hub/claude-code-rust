//! Web Module - Plugin Marketplace Web Interface
//!
//! This module provides a web server for the plugin marketplace
//! using Axum framework.

pub mod server;
pub mod routes;
pub mod handlers;
pub mod models;
pub mod templates;

pub use server::WebServer;
pub use models::*;
