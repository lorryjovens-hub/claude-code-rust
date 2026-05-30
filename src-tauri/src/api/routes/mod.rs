//! API route definitions.
//!
//! All route modules follow the same pattern:
//! - Each module exports handler functions
//! - All handlers use `State(state): State<AppState>` where AppState
//!   is the structured type from `crate::api::state::AppState`
//! - Errors use `crate::api::error::ApiError`
//!
//! When migrating from the legacy tuple-based state, handlers can use
//! `AppState::from_tuple(&state)` to convert at the route level.

pub mod config;
pub mod filesystem;
pub mod health;
pub mod system;
pub mod tools;

use crate::api::state::AppState;
use axum::Router;

/// Build the complete API router with all routes registered.
///
/// This is the single entry point for all HTTP routes in the application.
pub fn build_router() -> Router<AppState> {
    Router::new()
        // System routes (no auth required)
        .merge(health::routes())
        .merge(system::routes())
        // Core API routes
        .merge(config::routes())
        .merge(filesystem::routes())
        .merge(tools::routes())
}
