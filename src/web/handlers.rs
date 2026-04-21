//! Request Handlers - HTTP request handlers for the web interface

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Html,
    Json,
};
use std::sync::Arc;

use super::{
    models::*,
    templates::TemplateEngine,
};

/// Application state shared across handlers
pub struct AppState {
    pub template_engine: TemplateEngine,
    pub start_time: std::time::Instant,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            template_engine: TemplateEngine::new(),
            start_time: std::time::Instant::now(),
        }
    }
}

/// Health check endpoint
pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<ApiResponse<HealthResponse>> {
    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
    };

    Json(ApiResponse::success(response))
}

/// Get marketplace statistics
pub async fn get_stats() -> Json<ApiResponse<MarketplaceStats>> {
    // In a real implementation, fetch from database
    let stats = MarketplaceStats {
        total_plugins: 156,
        total_downloads: 456_789,
        total_users: 12_345,
        plugins_this_month: 23,
        downloads_this_month: 45_678,
    };

    Json(ApiResponse::success(stats))
}

/// Get featured plugins
pub async fn get_featured() -> Json<ApiResponse<FeaturedPlugins>> {
    let featured = FeaturedPlugins {
        trending: get_sample_plugins().into_iter().take(4).collect(),
        newest: get_sample_plugins().into_iter().take(4).collect(),
        top_rated: get_sample_plugins().into_iter().take(4).collect(),
        official: get_sample_plugins()
            .into_iter()
            .filter(|p| p.is_official)
            .take(4)
            .collect(),
    };

    Json(ApiResponse::success(featured))
}

/// Search plugins
pub async fn search_plugins(
    Query(query): Query<PluginSearchQuery>,
) -> Json<ApiResponse<PluginSearchResults>> {
    let all_plugins = get_sample_plugins();
    
    // Filter by search query
    let filtered: Vec<Plugin> = if let Some(ref q) = query.q {
        all_plugins
            .into_iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&q.to_lowercase())
                    || p.description.to_lowercase().contains(&q.to_lowercase())
                    || p.tags.iter().any(|t| t.to_lowercase().contains(&q.to_lowercase()))
            })
            .collect()
    } else {
        all_plugins
    };

    // Filter by category
    let filtered: Vec<Plugin> = if let Some(category) = query.category {
        filtered
            .into_iter()
            .filter(|p| p.category == category)
            .collect()
    } else {
        filtered
    };

    let total = filtered.len();
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let total_pages = (total + per_page - 1) / per_page;

    // Paginate
    let start = (page - 1) * per_page;
    let plugins: Vec<Plugin> = filtered
        .into_iter()
        .skip(start)
        .take(per_page)
        .collect();

    let results = PluginSearchResults {
        plugins,
        total,
        page,
        per_page,
        total_pages,
    };

    Json(ApiResponse::success(results))
}

/// Get a single plugin by ID
pub async fn get_plugin(
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Plugin>>, StatusCode> {
    let plugins = get_sample_plugins();
    
    match plugins.into_iter().find(|p| p.id == id) {
        Some(plugin) => Ok(Json(ApiResponse::success(plugin))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Get plugin reviews
pub async fn get_plugin_reviews(
    Path(id): Path<String>,
) -> Json<ApiResponse<Vec<PluginReview>>> {
    let reviews = vec![
        PluginReview {
            id: "1".to_string(),
            plugin_id: id.clone(),
            user_name: "Alice Developer".to_string(),
            user_avatar: None,
            rating: 5,
            title: "Excellent plugin!".to_string(),
            content: "This plugin has significantly improved my workflow. Highly recommended!".to_string(),
            created_at: chrono::Utc::now(),
            helpful_count: 42,
        },
        PluginReview {
            id: "2".to_string(),
            plugin_id: id,
            user_name: "Bob Coder".to_string(),
            user_avatar: None,
            rating: 4,
            title: "Great but needs improvement".to_string(),
            content: "Works well overall, but could use better documentation.".to_string(),
            created_at: chrono::Utc::now(),
            helpful_count: 15,
        },
    ];

    Json(ApiResponse::success(reviews))
}

/// Install a plugin
pub async fn install_plugin(
    Json(request): Json<InstallRequest>,
) -> Json<ApiResponse<InstallResponse>> {
    // In a real implementation, this would download and install the plugin
    let response = InstallResponse {
        success: true,
        message: format!("Plugin {} installed successfully", request.plugin_id),
        plugin: None,
    };

    Json(ApiResponse::success(response))
}

/// Get categories
pub async fn get_categories() -> Json<ApiResponse<Vec<serde_json::Value>>> {
    let categories: Vec<serde_json::Value> = vec![
        serde_json::json!({
            "id": "tools",
            "name": "Tools",
            "description": "Utility plugins for common tasks",
            "icon": "🛠️",
            "plugin_count": 45
        }),
        serde_json::json!({
            "id": "integrations",
            "name": "Integrations",
            "description": "Connect with external services",
            "icon": "🔌",
            "plugin_count": 32
        }),
        serde_json::json!({
            "id": "themes",
            "name": "Themes",
            "description": "Customize the appearance",
            "icon": "🎨",
            "plugin_count": 18
        }),
        serde_json::json!({
            "id": "language-support",
            "name": "Language Support",
            "description": "Additional language support",
            "icon": "🌐",
            "plugin_count": 24
        }),
        serde_json::json!({
            "id": "productivity",
            "name": "Productivity",
            "description": "Boost your productivity",
            "icon": "⚡",
            "plugin_count": 28
        }),
        serde_json::json!({
            "id": "development",
            "name": "Development",
            "description": "Tools for developers",
            "icon": "💻",
            "plugin_count": 9
        }),
    ];

    Json(ApiResponse::success(categories))
}

/// Get tags
pub async fn get_tags() -> Json<ApiResponse<Vec<String>>> {
    let tags = vec![
        "git".to_string(),
        "files".to_string(),
        "search".to_string(),
        "automation".to_string(),
        "ai".to_string(),
        "productivity".to_string(),
        "code-quality".to_string(),
        "testing".to_string(),
        "documentation".to_string(),
        "debugging".to_string(),
    ];

    Json(ApiResponse::success(tags))
}

/// Render the main page (HTML)
pub async fn index(State(state): State<Arc<AppState>>) -> Html<String> {
    let html = state.template_engine.render_index();
    Html(html)
}

/// Render the plugin detail page
pub async fn plugin_detail(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Html<String> {
    let html = state.template_engine.render_plugin_detail(&id);
    Html(html)
}

/// Render the search page
pub async fn search_page(State(state): State<Arc<AppState>>) -> Html<String> {
    let html = state.template_engine.render_search();
    Html(html)
}

/// Sample plugins for demonstration
fn get_sample_plugins() -> Vec<Plugin> {
    vec![
        Plugin {
            id: "file-system".to_string(),
            name: "File System".to_string(),
            description: "Advanced file operations including search, batch rename, and directory sync".to_string(),
            version: "1.2.0".to_string(),
            author: "Claude Code Team".to_string(),
            author_url: Some("https://github.com/claude-code".to_string()),
            repository_url: Some("https://github.com/claude-code/file-system-plugin".to_string()),
            documentation_url: Some("https://docs.claude-code.dev/plugins/file-system".to_string()),
            icon_url: None,
            tags: vec!["files".to_string(), "filesystem".to_string(), "utilities".to_string()],
            category: PluginCategory::Tools,
            downloads: 125_432,
            rating: 4.8,
            rating_count: 234,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            license: "MIT".to_string(),
            is_official: true,
            is_verified: true,
            dependencies: vec![],
            min_api_version: "0.1.0".to_string(),
            max_api_version: None,
        },
        Plugin {
            id: "git-integration".to_string(),
            name: "Git Integration".to_string(),
            description: "Full Git support with commit, branch, merge, and history visualization".to_string(),
            version: "2.0.1".to_string(),
            author: "Claude Code Team".to_string(),
            author_url: Some("https://github.com/claude-code".to_string()),
            repository_url: Some("https://github.com/claude-code/git-plugin".to_string()),
            documentation_url: Some("https://docs.claude-code.dev/plugins/git".to_string()),
            icon_url: None,
            tags: vec!["git".to_string(), "version-control".to_string(), "scm".to_string()],
            category: PluginCategory::Development,
            downloads: 98_765,
            rating: 4.9,
            rating_count: 189,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            license: "MIT".to_string(),
            is_official: true,
            is_verified: true,
            dependencies: vec![],
            min_api_version: "0.1.0".to_string(),
            max_api_version: None,
        },
        Plugin {
            id: "code-analyzer".to_string(),
            name: "Code Analyzer".to_string(),
            description: "Static code analysis with support for multiple languages and custom rules".to_string(),
            version: "1.5.0".to_string(),
            author: "Code Quality Inc".to_string(),
            author_url: Some("https://codequality.dev".to_string()),
            repository_url: Some("https://github.com/codequality/analyzer".to_string()),
            documentation_url: Some("https://docs.codequality.dev".to_string()),
            icon_url: None,
            tags: vec!["analysis".to_string(), "linting".to_string(), "quality".to_string()],
            category: PluginCategory::Development,
            downloads: 45_678,
            rating: 4.5,
            rating_count: 123,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            license: "Apache-2.0".to_string(),
            is_official: false,
            is_verified: true,
            dependencies: vec![],
            min_api_version: "0.1.0".to_string(),
            max_api_version: None,
        },
        Plugin {
            id: "dark-theme".to_string(),
            name: "Midnight Theme".to_string(),
            description: "A beautiful dark theme with customizable accent colors".to_string(),
            version: "1.0.0".to_string(),
            author: "Theme Designer".to_string(),
            author_url: Some("https://themedesigner.dev".to_string()),
            repository_url: Some("https://github.com/themedesigner/midnight".to_string()),
            documentation_url: None,
            icon_url: None,
            tags: vec!["theme".to_string(), "dark".to_string(), "ui".to_string()],
            category: PluginCategory::Themes,
            downloads: 67_890,
            rating: 4.7,
            rating_count: 156,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            license: "MIT".to_string(),
            is_official: false,
            is_verified: false,
            dependencies: vec![],
            min_api_version: "0.1.0".to_string(),
            max_api_version: None,
        },
        Plugin {
            id: "slack-integration".to_string(),
            name: "Slack Integration".to_string(),
            description: "Send notifications and interact with Slack from Claude Code".to_string(),
            version: "1.1.0".to_string(),
            author: "Integration Expert".to_string(),
            author_url: Some("https://integrations.dev".to_string()),
            repository_url: Some("https://github.com/integrations/slack".to_string()),
            documentation_url: Some("https://docs.integrations.dev/slack".to_string()),
            icon_url: None,
            tags: vec!["slack".to_string(), "notifications".to_string(), "integration".to_string()],
            category: PluginCategory::Integrations,
            downloads: 23_456,
            rating: 4.3,
            rating_count: 89,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            license: "MIT".to_string(),
            is_official: false,
            is_verified: true,
            dependencies: vec![],
            min_api_version: "0.1.0".to_string(),
            max_api_version: None,
        },
    ]
}
