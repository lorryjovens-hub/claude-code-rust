//! Web Models - Data structures for the web interface

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Plugin information for the marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub author_url: Option<String>,
    pub repository_url: Option<String>,
    pub documentation_url: Option<String>,
    pub icon_url: Option<String>,
    pub tags: Vec<String>,
    pub category: PluginCategory,
    pub downloads: u64,
    pub rating: f32,
    pub rating_count: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub license: String,
    pub is_official: bool,
    pub is_verified: bool,
    pub dependencies: Vec<String>,
    pub min_api_version: String,
    pub max_api_version: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginCategory {
    Tools,
    Integrations,
    Themes,
    LanguageSupport,
    Productivity,
    Development,
    Other,
}

impl std::fmt::Display for PluginCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginCategory::Tools => write!(f, "Tools"),
            PluginCategory::Integrations => write!(f, "Integrations"),
            PluginCategory::Themes => write!(f, "Themes"),
            PluginCategory::LanguageSupport => write!(f, "Language Support"),
            PluginCategory::Productivity => write!(f, "Productivity"),
            PluginCategory::Development => write!(f, "Development"),
            PluginCategory::Other => write!(f, "Other"),
        }
    }
}

/// Plugin search query
#[derive(Debug, Clone, Deserialize)]
pub struct PluginSearchQuery {
    pub q: Option<String>,
    pub category: Option<PluginCategory>,
    pub sort: Option<SortOption>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum SortOption {
    Relevance,
    Downloads,
    Rating,
    Newest,
    Updated,
}

/// Plugin search results
#[derive(Debug, Clone, Serialize)]
pub struct PluginSearchResults {
    pub plugins: Vec<Plugin>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
}

/// Plugin review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginReview {
    pub id: String,
    pub plugin_id: String,
    pub user_name: String,
    pub user_avatar: Option<String>,
    pub rating: u8,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub helpful_count: u32,
}

/// User profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub website: Option<String>,
    pub github: Option<String>,
    pub twitter: Option<String>,
    pub plugins_published: Vec<String>,
    pub joined_at: DateTime<Utc>,
    pub is_verified: bool,
}

/// API Response wrapper
#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            message: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
            message: None,
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}

/// Plugin installation request
#[derive(Debug, Clone, Deserialize)]
pub struct InstallRequest {
    pub plugin_id: String,
    pub version: Option<String>,
}

/// Plugin installation response
#[derive(Debug, Clone, Serialize)]
pub struct InstallResponse {
    pub success: bool,
    pub message: String,
    pub plugin: Option<Plugin>,
}

/// Health check response
#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub uptime_seconds: u64,
}

/// Statistics for the marketplace
#[derive(Debug, Clone, Serialize)]
pub struct MarketplaceStats {
    pub total_plugins: usize,
    pub total_downloads: u64,
    pub total_users: usize,
    pub plugins_this_month: usize,
    pub downloads_this_month: u64,
}

/// Featured plugins collection
#[derive(Debug, Clone, Serialize)]
pub struct FeaturedPlugins {
    pub trending: Vec<Plugin>,
    pub newest: Vec<Plugin>,
    pub top_rated: Vec<Plugin>,
    pub official: Vec<Plugin>,
}
