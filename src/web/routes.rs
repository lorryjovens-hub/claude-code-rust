//! Route Configuration - Axum route definitions

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use super::{
    handlers::*,
};

/// Create the application router
pub fn create_router(state: Arc<AppState>) -> Router {
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // HTML Pages
        .route("/", get(index))
        .route("/search", get(search_page))
        .route("/plugin/:id", get(plugin_detail))
        
        // API Routes
        .route("/api/health", get(health_check))
        .route("/api/stats", get(get_stats))
        .route("/api/featured", get(get_featured))
        .route("/api/plugins", get(search_plugins))
        .route("/api/plugins/:id", get(get_plugin))
        .route("/api/plugins/:id/reviews", get(get_plugin_reviews))
        .route("/api/plugins/:id/install", post(install_plugin))
        .route("/api/categories", get(get_categories))
        .route("/api/tags", get(get_tags))
        
        // Static files
        .nest_service("/static", ServeDir::new("static"))
        
        // Add middleware
        .layer(cors)
        .with_state(state)
}
