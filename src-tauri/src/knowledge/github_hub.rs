use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::Mutex;

/* ═══════════════════════════════════════════════
   GitHub Intelligence Hub
   Trending, search, OAuth, webhooks, fusion
   ═══════════════════════════════════════════════ */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub id: u64, pub name: String, pub full_name: String, pub description: String,
    pub html_url: String, pub stars: u32, pub forks: u32, pub language: String,
    pub topics: Vec<String>, pub license: Option<String>, pub updated_at: String,
    pub pushed_at: String, pub open_issues: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingQuery {
    pub since: String,        // daily, weekly, monthly
    pub language: Option<String>,
    pub spoken_language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionDirection {
    pub pattern: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
    pub difficulty: u8,
}

pub static FUSION_DIRECTIONS: &[FusionDirection] = &[
    FusionDirection { pattern: "plugin-system", description: "将一个项目的插件系统集成到另一个项目的架构中，实现功能的可扩展性", icon: "🧩", difficulty: 6 },
    FusionDirection { pattern: "pipeline-integration", description: "将两个工具链通过 CI/CD 管道串联，形成端到端自动化工作流", icon: "🔗", difficulty: 4 },
    FusionDirection { pattern: "micro-frontend", description: "将多个前端项目通过微前端架构融合，统一用户体验", icon: "🧱", difficulty: 7 },
    FusionDirection { pattern: "api-gateway", description: "将多个服务通过统一 API 网关整合，提供一致的后端接口", icon: "🚪", difficulty: 5 },
    FusionDirection { pattern: "data-pipeline", description: "融合数据采集+处理+可视化工具，构建完整数据管道", icon: "📊", difficulty: 5 },
    FusionDirection { pattern: "mcp-server", description: "将项目封装为 MCP 服务器，让 AI 直接调用其能力", icon: "🤖", difficulty: 3 },
    FusionDirection { pattern: "cross-platform", description: "将一个平台的优秀工具适配到另一个平台，扩大用户覆盖", icon: "🔄", difficulty: 6 },
    FusionDirection { pattern: "template-generation", description: "从项目中提取脚手架模板，作为新项目的起点", icon: "🏗️", difficulty: 2 },
];

pub struct GitHubHub {
    client: reqwest::Client,
    tokens: Mutex<HashMap<String, String>>, // user_id -> token
    watched: Mutex<Vec<WatchedRepo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchedRepo {
    pub full_name: String, pub last_commit: String, pub last_checked: String,
    pub new_stars: u32, pub new_issues: u32, pub notifications: Vec<String>,
}

impl GitHubHub {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .user_agent("ClaudeDesktop-GitHubHub/2.0")
                .build().unwrap_or_default(),
            tokens: Mutex::new(HashMap::new()),
            watched: Mutex::new(Vec::new()),
        }
    }

    /// OAuth: exchange code for token
    pub async fn exchange_code(&self, code: &str) -> Result<String, String> {
        let secret = std::env::var("GITHUB_CLIENT_SECRET").unwrap_or_default();
        let params = vec![
            ("client_id", "Ov23liI6ZwqWYNjYimo3"),
            ("client_secret", &secret),
            ("code", code),
        ];
        let resp = self.client
            .post("https://github.com/login/oauth/access_token")
            .header("Accept", "application/json")
            .form(&params)
            .send().await.map_err(|e| e.to_string())?;
        let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        data.get("access_token").and_then(|t| t.as_str()).map(String::from)
            .ok_or_else(|| "No access_token in response".to_string())
    }

    /// Set a user's token
    pub async fn set_token(&self, user_id: &str, token: &str) {
        self.tokens.lock().await.insert(user_id.to_string(), token.to_string());
    }

    /// Fetch trending repos from GitHub API
    pub async fn fetch_trending(&self, since: &str, lang: Option<&str>) -> Result<Vec<GitHubRepo>, String> {
        let query = format!("created:>{}", match since {
            "weekly" => "2026-05-20", "monthly" => "2026-04-27",
            _ => "2026-05-26",
        });
        let mut q = format!("stars:>50 {}", query);
        if let Some(l) = lang { q.push_str(&format!(" language:{}", l)); }

        let url = format!("https://api.github.com/search/repositories?q={}&sort=stars&order=desc&per_page=30", urlencoding::encode(&q));
        let resp = self.client.get(&url)
            .header("Accept", "application/vnd.github.v3+json")
            .send().await.map_err(|e| format!("GitHub API: {}", e))?;

        let data: serde_json::Value = resp.json().await.map_err(|e| format!("Parse: {}", e))?;
        let items = data.get("items").and_then(|a| a.as_array()).ok_or("No items")?;

        Ok(items.iter().map(|i| GitHubRepo {
            id: i.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
            name: i.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            full_name: i.get("full_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            description: i.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            html_url: i.get("html_url").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            stars: i.get("stargazers_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            forks: i.get("forks_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            language: i.get("language").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            topics: i.get("topics").and_then(|a| a.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
            license: i.get("license").and_then(|l| l.get("spdx_id")).and_then(|v| v.as_str()).map(String::from),
            updated_at: i.get("updated_at").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            pushed_at: i.get("pushed_at").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            open_issues: i.get("open_issues_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        }).collect())
    }

    /// Search repos by query
    pub async fn search_repos(&self, query: &str) -> Result<Vec<GitHubRepo>, String> {
        let url = format!("https://api.github.com/search/repositories?q={}&sort=stars&per_page=20", urlencoding::encode(query));
        let resp = self.client.get(&url)
            .header("Accept", "application/vnd.github.v3+json")
            .send().await.map_err(|e| format!("Search error: {}", e))?;
        let data: serde_json::Value = resp.json().await.map_err(|e| format!("Parse: {}", e))?;
        let items = data.get("items").and_then(|a| a.as_array()).ok_or("No results")?;
        Ok(items.iter().map(|i| GitHubRepo {
            id: i.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
            name: i.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            full_name: i.get("full_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            description: i.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            html_url: i.get("html_url").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            stars: i.get("stargazers_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            forks: i.get("forks_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            language: i.get("language").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            topics: i.get("topics").and_then(|a| a.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
            license: i.get("license").and_then(|l| l.get("spdx_id")).and_then(|v| v.as_str()).map(String::from),
            updated_at: "".to_string(), pushed_at: "".to_string(), open_issues: 0,
        }).collect())
    }

    /// Get user's repos (requires auth)
    pub async fn get_user_repos(&self) -> Result<Vec<GitHubRepo>, String> {
        let token = {
            let tokens = self.tokens.lock().await;
            tokens.values().next().cloned()
        };
        let token = token.ok_or("Not authenticated")?;
        let resp = self.client.get("https://api.github.com/user/repos?sort=updated&per_page=50&type=owner")
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/vnd.github.v3+json")
            .send().await.map_err(|e| format!("API: {}", e))?;
        let data: Vec<serde_json::Value> = resp.json().await.map_err(|e| format!("Parse: {}", e))?;
        Ok(data.iter().map(|i| GitHubRepo {
            id: i.get("id").and_then(|v| v.as_u64()).unwrap_or(0),
            name: i.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            full_name: i.get("full_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            description: i.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            html_url: i.get("html_url").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            stars: i.get("stargazers_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            forks: i.get("forks_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            language: i.get("language").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            topics: i.get("topics").and_then(|a| a.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default(),
            license: i.get("license").and_then(|l| l.get("spdx_id")).and_then(|v| v.as_str()).map(String::from),
            updated_at: i.get("updated_at").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            pushed_at: i.get("pushed_at").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            open_issues: i.get("open_issues_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        }).collect())
    }

    /// Add repo to watch list
    pub async fn watch_repo(&self, full_name: &str) -> Result<(), String> {
        let mut watched = self.watched.lock().await;
        if !watched.iter().any(|w| w.full_name == full_name) {
            watched.push(WatchedRepo {
                full_name: full_name.to_string(),
                last_commit: String::new(), last_checked: chrono::Utc::now().to_rfc3339(),
                new_stars: 0, new_issues: 0, notifications: vec![],
            });
        }
        Ok(())
    }

    /// Get watch list
    pub async fn get_watched(&self) -> Vec<WatchedRepo> {
        self.watched.lock().await.clone()
    }

    /// Generate OAuth URL
    pub fn get_oauth_url(state: &str) -> String {
        format!("https://github.com/login/oauth/authorize?client_id=Ov23liI6ZwqWYNjYimo3&redirect_uri=http://127.0.0.1:30085/api/github/callback&scope=repo,user&state={}", state)
    }
}
