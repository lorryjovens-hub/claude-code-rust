use anyhow::{anyhow, Result};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub token: Option<String>,
    pub base_url: String,
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            token: None,
            base_url: "https://api.github.com".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub html_url: String,
    pub clone_url: String,
    pub ssh_url: String,
    pub default_branch: String,
    pub language: Option<String>,
    pub stargazers_count: i64,
    pub forks_count: i64,
    pub open_issues_count: i64,
    pub owner: String,
    pub is_private: bool,
    pub is_fork: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: String,
    pub html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubPullRequest {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub html_url: String,
    pub user: String,
    pub head_branch: String,
    pub base_branch: String,
    pub can_merge: bool,
    pub is_draft: bool,
    pub created_at: String,
    pub updated_at: String,
    pub merged_at: Option<String>,
    pub mergeable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub id: i64,
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub html_url: String,
    pub user: String,
    pub labels: Vec<String>,
    pub assignees: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub comments_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubCommit {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub author_email: String,
    pub date: String,
    pub html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubComment {
    pub id: i64,
    pub body: String,
    pub user: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePullRequestRequest {
    pub owner: String,
    pub repo: String,
    pub title: String,
    pub body: Option<String>,
    pub head: String,
    pub base: String,
    pub draft: Option<bool>,
    pub maintainer_can_modify: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssueRequest {
    pub owner: String,
    pub repo: String,
    pub title: String,
    pub body: Option<String>,
    pub labels: Option<Vec<String>>,
    pub assignees: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubReviewRequest {
    pub body: Option<String>,
    pub event: String,
    pub comments: Option<Vec<ReviewComment>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewComment {
    pub path: String,
    pub position: Option<i64>,
    pub body: String,
}

pub struct GitHubIntegration {
    client: Client,
    config: Arc<RwLock<GitHubConfig>>,
}

impl GitHubIntegration {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            config: Arc::new(RwLock::new(GitHubConfig::default())),
        }
    }

    pub async fn set_token(&self, token: String) {
        let mut config = self.config.write().await;
        config.token = Some(token);
    }

    pub async fn get_token(&self) -> Option<String> {
        self.config.read().await.token.clone()
    }

    fn auth_headers(&self, token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", token)) {
            headers.insert(AUTHORIZATION, val);
        }
        headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.github.v3+json"));
        headers.insert(USER_AGENT, HeaderValue::from_static("Claude-Desktop-Tauri"));
        headers
    }

    pub async fn get_current_user(&self) -> Result<GitHubUser> {
        let token = self.get_token().await.ok_or_else(|| anyhow!("No token set"))?;
        let headers = self.auth_headers(&token);
        
        let base_url = self.config.read().await.base_url.clone();
        let url = format!("{}/user", base_url);
        
        let response = self.client.get(&url).headers(headers.clone()).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to get user: {}", response.status()));
        }
        
        let data: serde_json::Value = response.json().await?;
        
        Ok(GitHubUser {
            id: data["id"].as_i64().unwrap_or(0),
            login: data["login"].as_str().unwrap_or("").to_string(),
            name: data["name"].as_str().map(String::from),
            email: data["email"].as_str().map(String::from),
            avatar_url: data["avatar_url"].as_str().unwrap_or("").to_string(),
            html_url: data["html_url"].as_str().unwrap_or("").to_string(),
        })
    }

    pub async fn get_repo(&self, owner: &str, repo: &str) -> Result<GitHubRepo> {
        let token = self.get_token().await.ok_or_else(|| anyhow!("No token set"))?;
        let headers = self.auth_headers(&token);
        
        let base_url = self.config.read().await.base_url.clone();
        let url = format!("{}/repos/{}/{}", base_url, owner, repo);
        
        let response = self.client.get(&url).headers(headers.clone()).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to get repo: {}", response.status()));
        }
        
        let data: serde_json::Value = response.json().await?;
        
        Ok(GitHubRepo {
            id: data["id"].as_i64().unwrap_or(0),
            name: data["name"].as_str().unwrap_or("").to_string(),
            full_name: data["full_name"].as_str().unwrap_or("").to_string(),
            description: data["description"].as_str().map(String::from),
            html_url: data["html_url"].as_str().unwrap_or("").to_string(),
            clone_url: data["clone_url"].as_str().unwrap_or("").to_string(),
            ssh_url: data["ssh_url"].as_str().unwrap_or("").to_string(),
            default_branch: data["default_branch"].as_str().unwrap_or("main").to_string(),
            language: data["language"].as_str().map(String::from),
            stargazers_count: data["stargazers_count"].as_i64().unwrap_or(0),
            forks_count: data["forks_count"].as_i64().unwrap_or(0),
            open_issues_count: data["open_issues_count"].as_i64().unwrap_or(0),
            owner: data["owner"]["login"].as_str().unwrap_or("").to_string(),
            is_private: data["private"].as_bool().unwrap_or(false),
            is_fork: data["fork"].as_bool().unwrap_or(false),
        })
    }

    pub async fn list_repos(&self, page: Option<i32>, per_page: Option<i32>) -> Result<Vec<GitHubRepo>> {
        let token = self.get_token().await.ok_or_else(|| anyhow!("No token set"))?;
        let headers = self.auth_headers(&token);
        
        let base_url = self.config.read().await.base_url.clone();
        let url = format!("{}/user/repos?page={}&per_page={}", 
            base_url, 
            page.unwrap_or(1), 
            per_page.unwrap_or(30)
        );
        
        let response = self.client.get(&url).headers(headers.clone()).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to list repos: {}", response.status()));
        }
        
        let data: Vec<serde_json::Value> = response.json().await?;
        
        Ok(data.into_iter().map(|r| GitHubRepo {
            id: r["id"].as_i64().unwrap_or(0),
            name: r["name"].as_str().unwrap_or("").to_string(),
            full_name: r["full_name"].as_str().unwrap_or("").to_string(),
            description: r["description"].as_str().map(String::from),
            html_url: r["html_url"].as_str().unwrap_or("").to_string(),
            clone_url: r["clone_url"].as_str().unwrap_or("").to_string(),
            ssh_url: r["ssh_url"].as_str().unwrap_or("").to_string(),
            default_branch: r["default_branch"].as_str().unwrap_or("main").to_string(),
            language: r["language"].as_str().map(String::from),
            stargazers_count: r["stargazers_count"].as_i64().unwrap_or(0),
            forks_count: r["forks_count"].as_i64().unwrap_or(0),
            open_issues_count: r["open_issues_count"].as_i64().unwrap_or(0),
            owner: r["owner"]["login"].as_str().unwrap_or("").to_string(),
            is_private: r["private"].as_bool().unwrap_or(false),
            is_fork: r["fork"].as_bool().unwrap_or(false),
        }).collect())
    }

    pub async fn create_pull_request(&self, req: CreatePullRequestRequest) -> Result<GitHubPullRequest> {
        let token = self.get_token().await.ok_or_else(|| anyhow!("No token set"))?;
        let headers = self.auth_headers(&token);
        
        let base_url = self.config.read().await.base_url.clone();
        let url = format!("{}/repos/{}/{}/pulls", base_url, req.owner, req.repo);
        
        let body = serde_json::json!({
            "title": req.title,
            "body": req.body,
            "head": req.head,
            "base": req.base,
            "draft": req.draft.unwrap_or(false),
            "maintainer_can_modify": req.maintainer_can_modify.unwrap_or(true),
        });
        
        let response = self.client.post(&url)
            .headers(headers.clone())
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Failed to create PR: {} - {}", status, error_text));
        }

        let data: serde_json::Value = response.json().await?;
        
        Ok(GitHubPullRequest {
            id: data["id"].as_i64().unwrap_or(0),
            number: data["number"].as_i64().unwrap_or(0),
            title: data["title"].as_str().unwrap_or("").to_string(),
            body: data["body"].as_str().map(String::from),
            state: data["state"].as_str().unwrap_or("open").to_string(),
            html_url: data["html_url"].as_str().unwrap_or("").to_string(),
            user: data["user"]["login"].as_str().unwrap_or("").to_string(),
            head_branch: data["head"]["ref"].as_str().unwrap_or("").to_string(),
            base_branch: data["base"]["ref"].as_str().unwrap_or("").to_string(),
            can_merge: data["maintainer_can_modify"].as_bool().unwrap_or(false),
            is_draft: data["draft"].as_bool().unwrap_or(false),
            created_at: data["created_at"].as_str().unwrap_or("").to_string(),
            updated_at: data["updated_at"].as_str().unwrap_or("").to_string(),
            merged_at: data["merged_at"].as_str().map(String::from),
            mergeable: data["mergeable"].as_bool(),
        })
    }

    pub async fn get_pull_request(&self, owner: &str, repo: &str, number: i64) -> Result<GitHubPullRequest> {
        let token = self.get_token().await.ok_or_else(|| anyhow!("No token set"))?;
        let headers = self.auth_headers(&token);
        
        let base_url = self.config.read().await.base_url.clone();
        let url = format!("{}/repos/{}/{}/pulls/{}", base_url, owner, repo, number);
        
        let response = self.client.get(&url).headers(headers.clone()).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to get PR: {}", response.status()));
        }
        
        let data: serde_json::Value = response.json().await?;
        
        Ok(GitHubPullRequest {
            id: data["id"].as_i64().unwrap_or(0),
            number: data["number"].as_i64().unwrap_or(0),
            title: data["title"].as_str().unwrap_or("").to_string(),
            body: data["body"].as_str().map(String::from),
            state: data["state"].as_str().unwrap_or("open").to_string(),
            html_url: data["html_url"].as_str().unwrap_or("").to_string(),
            user: data["user"]["login"].as_str().unwrap_or("").to_string(),
            head_branch: data["head"]["ref"].as_str().unwrap_or("").to_string(),
            base_branch: data["base"]["ref"].as_str().unwrap_or("").to_string(),
            can_merge: data["maintainer_can_modify"].as_bool().unwrap_or(false),
            is_draft: data["draft"].as_bool().unwrap_or(false),
            created_at: data["created_at"].as_str().unwrap_or("").to_string(),
            updated_at: data["updated_at"].as_str().unwrap_or("").to_string(),
            merged_at: data["merged_at"].as_str().map(String::from),
            mergeable: data["mergeable"].as_bool(),
        })
    }

    pub async fn list_pull_requests(&self, owner: &str, repo: &str, state: Option<&str>) -> Result<Vec<GitHubPullRequest>> {
        let token = self.get_token().await.ok_or_else(|| anyhow!("No token set"))?;
        let headers = self.auth_headers(&token);
        
        let base_url = self.config.read().await.base_url.clone();
        let state_param = state.unwrap_or("open");
        let url = format!("{}/repos/{}/{}/pulls?state={}", base_url, owner, repo, state_param);
        
        let response = self.client.get(&url).headers(headers.clone()).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to list PRs: {}", response.status()));
        }
        
        let data: Vec<serde_json::Value> = response.json().await?;
        
        Ok(data.into_iter().map(|pr| GitHubPullRequest {
            id: pr["id"].as_i64().unwrap_or(0),
            number: pr["number"].as_i64().unwrap_or(0),
            title: pr["title"].as_str().unwrap_or("").to_string(),
            body: pr["body"].as_str().map(String::from),
            state: pr["state"].as_str().unwrap_or("open").to_string(),
            html_url: pr["html_url"].as_str().unwrap_or("").to_string(),
            user: pr["user"]["login"].as_str().unwrap_or("").to_string(),
            head_branch: pr["head"]["ref"].as_str().unwrap_or("").to_string(),
            base_branch: pr["base"]["ref"].as_str().unwrap_or("").to_string(),
            can_merge: pr["maintainer_can_modify"].as_bool().unwrap_or(false),
            is_draft: pr["draft"].as_bool().unwrap_or(false),
            created_at: pr["created_at"].as_str().unwrap_or("").to_string(),
            updated_at: pr["updated_at"].as_str().unwrap_or("").to_string(),
            merged_at: pr["merged_at"].as_str().map(String::from),
            mergeable: pr["mergeable"].as_bool(),
        }).collect())
    }

    pub async fn merge_pull_request(&self, owner: &str, repo: &str, number: i64, commit_title: Option<&str>, commit_message: Option<&str>) -> Result<bool> {
        let token = self.get_token().await.ok_or_else(|| anyhow!("No token set"))?;
        let headers = self.auth_headers(&token);
        
        let base_url = self.config.read().await.base_url.clone();
        let url = format!("{}/repos/{}/{}/pulls/{}/merge", base_url, owner, repo, number);
        
        let mut body = serde_json::json!({});
        if let Some(title) = commit_title {
            body["merge_title"] = serde_json::json!(title);
        }
        if let Some(msg) = commit_message {
            body["merge_message"] = serde_json::json!(msg);
        }
        
        let response = self.client.put(&url)
            .headers(headers.clone())
            .json(&body)
            .send()
            .await?;
        
        Ok(response.status().is_success())
    }

    pub async fn create_issue(&self, req: CreateIssueRequest) -> Result<GitHubIssue> {
        let token = self.get_token().await.ok_or_else(|| anyhow!("No token set"))?;
        let headers = self.auth_headers(&token);
        
        let base_url = self.config.read().await.base_url.clone();
        let url = format!("{}/repos/{}/{}/issues", base_url, req.owner, req.repo);
        
        let mut body = serde_json::json!({
            "title": req.title,
        });
        
        if let Some(b) = req.body {
            body["body"] = serde_json::json!(b);
        }
        if let Some(labels) = req.labels {
            body["labels"] = serde_json::json!(labels);
        }
        if let Some(assignees) = req.assignees {
            body["assignees"] = serde_json::json!(assignees);
        }
        
        let response = self.client.post(&url)
            .headers(headers.clone())
            .json(&body)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to create issue: {}", response.status()));
        }
        
        let data: serde_json::Value = response.json().await?;
        
        let labels: Vec<String> = data["labels"].as_array()
            .map(|arr| arr.iter().filter_map(|l| l["name"].as_str().map(String::from)).collect())
            .unwrap_or_default();
        
        let assignees: Vec<String> = data["assignees"].as_array()
            .map(|arr| arr.iter().filter_map(|a| a["login"].as_str().map(String::from)).collect())
            .unwrap_or_default();
        
        Ok(GitHubIssue {
            id: data["id"].as_i64().unwrap_or(0),
            number: data["number"].as_i64().unwrap_or(0),
            title: data["title"].as_str().unwrap_or("").to_string(),
            body: data["body"].as_str().map(String::from),
            state: data["state"].as_str().unwrap_or("open").to_string(),
            html_url: data["html_url"].as_str().unwrap_or("").to_string(),
            user: data["user"]["login"].as_str().unwrap_or("").to_string(),
            labels,
            assignees,
            created_at: data["created_at"].as_str().unwrap_or("").to_string(),
            updated_at: data["updated_at"].as_str().unwrap_or("").to_string(),
            comments_count: data["comments"].as_i64().unwrap_or(0),
        })
    }

    pub async fn list_issues(&self, owner: &str, repo: &str, state: Option<&str>, labels: Option<&str>) -> Result<Vec<GitHubIssue>> {
        let token = self.get_token().await.ok_or_else(|| anyhow!("No token set"))?;
        let headers = self.auth_headers(&token);
        
        let base_url = self.config.read().await.base_url.clone();
        let mut url = format!("{}/repos/{}/{}/issues?state={}", base_url, owner, repo, state.unwrap_or("open"));
        
        if let Some(l) = labels {
            url.push_str(&format!("&labels={}", l));
        }
        
        let response = self.client.get(&url).headers(headers.clone()).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to list issues: {}", response.status()));
        }
        
        let data: Vec<serde_json::Value> = response.json().await?;
        
        Ok(data.into_iter().filter(|i| i.get("pull_request").is_none()).map(|issue| {
            let labels_list: Vec<String> = issue["labels"].as_array()
                .map(|arr| arr.iter().filter_map(|l| l["name"].as_str().map(String::from)).collect())
                .unwrap_or_default();
            
            let assignees_list: Vec<String> = issue["assignees"].as_array()
                .map(|arr| arr.iter().filter_map(|a| a["login"].as_str().map(String::from)).collect())
                .unwrap_or_default();
            
            GitHubIssue {
                id: issue["id"].as_i64().unwrap_or(0),
                number: issue["number"].as_i64().unwrap_or(0),
                title: issue["title"].as_str().unwrap_or("").to_string(),
                body: issue["body"].as_str().map(String::from),
                state: issue["state"].as_str().unwrap_or("open").to_string(),
                html_url: issue["html_url"].as_str().unwrap_or("").to_string(),
                user: issue["user"]["login"].as_str().unwrap_or("").to_string(),
                labels: labels_list,
                assignees: assignees_list,
                created_at: issue["created_at"].as_str().unwrap_or("").to_string(),
                updated_at: issue["updated_at"].as_str().unwrap_or("").to_string(),
                comments_count: issue["comments"].as_i64().unwrap_or(0),
            }
        }).collect())
    }

    pub async fn get_issue_comments(&self, owner: &str, repo: &str, number: i64) -> Result<Vec<GitHubComment>> {
        let token = self.get_token().await.ok_or_else(|| anyhow!("No token set"))?;
        let headers = self.auth_headers(&token);
        
        let base_url = self.config.read().await.base_url.clone();
        let url = format!("{}/repos/{}/{}/issues/{}/comments", base_url, owner, repo, number);
        
        let response = self.client.get(&url).headers(headers.clone()).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to get comments: {}", response.status()));
        }
        
        let data: Vec<serde_json::Value> = response.json().await?;
        
        Ok(data.into_iter().map(|c| GitHubComment {
            id: c["id"].as_i64().unwrap_or(0),
            body: c["body"].as_str().unwrap_or("").to_string(),
            user: c["user"]["login"].as_str().unwrap_or("").to_string(),
            created_at: c["created_at"].as_str().unwrap_or("").to_string(),
            updated_at: c["updated_at"].as_str().unwrap_or("").to_string(),
        }).collect())
    }

    pub async fn create_issue_comment(&self, owner: &str, repo: &str, number: i64, body: &str) -> Result<GitHubComment> {
        let token = self.get_token().await.ok_or_else(|| anyhow!("No token set"))?;
        let headers = self.auth_headers(&token);
        
        let base_url = self.config.read().await.base_url.clone();
        let url = format!("{}/repos/{}/{}/issues/{}/comments", base_url, owner, repo, number);
        
        let response = self.client.post(&url)
            .headers(headers.clone())
            .json(&serde_json::json!({ "body": body }))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to create comment: {}", response.status()));
        }
        
        let data: serde_json::Value = response.json().await?;
        
        Ok(GitHubComment {
            id: data["id"].as_i64().unwrap_or(0),
            body: data["body"].as_str().unwrap_or("").to_string(),
            user: data["user"]["login"].as_str().unwrap_or("").to_string(),
            created_at: data["created_at"].as_str().unwrap_or("").to_string(),
            updated_at: data["updated_at"].as_str().unwrap_or("").to_string(),
        })
    }

    pub async fn search_repos(&self, query: &str, page: Option<i32>, per_page: Option<i32>) -> Result<Vec<GitHubRepo>> {
        let token = self.get_token().await.ok_or_else(|| anyhow!("No token set"))?;
        let headers = self.auth_headers(&token);
        
        let base_url = self.config.read().await.base_url.clone();
        let url = format!("{}/search/repositories?q={}&page={}&per_page={}",
            base_url,
            urlencoding::encode(query),
            page.unwrap_or(1),
            per_page.unwrap_or(30)
        );
        
        let response = self.client.get(&url).headers(headers.clone()).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to search repos: {}", response.status()));
        }
        
        let data: serde_json::Value = response.json().await?;
        
        let items = data["items"].as_array().cloned().unwrap_or_default();
        
        Ok(items.into_iter().map(|r| GitHubRepo {
            id: r["id"].as_i64().unwrap_or(0),
            name: r["name"].as_str().unwrap_or("").to_string(),
            full_name: r["full_name"].as_str().unwrap_or("").to_string(),
            description: r["description"].as_str().map(String::from),
            html_url: r["html_url"].as_str().unwrap_or("").to_string(),
            clone_url: r["clone_url"].as_str().unwrap_or("").to_string(),
            ssh_url: r["ssh_url"].as_str().unwrap_or("").to_string(),
            default_branch: r["default_branch"].as_str().unwrap_or("main").to_string(),
            language: r["language"].as_str().map(String::from),
            stargazers_count: r["stargazers_count"].as_i64().unwrap_or(0),
            forks_count: r["forks_count"].as_i64().unwrap_or(0),
            open_issues_count: r["open_issues_count"].as_i64().unwrap_or(0),
            owner: r["owner"]["login"].as_str().unwrap_or("").to_string(),
            is_private: r["private"].as_bool().unwrap_or(false),
            is_fork: r["fork"].as_bool().unwrap_or(false),
        }).collect())
    }

    pub async fn get_file_content(&self, owner: &str, repo: &str, path: &str, ref_: Option<&str>) -> Result<String> {
        let token = self.get_token().await.ok_or_else(|| anyhow!("No token set"))?;
        let headers = self.auth_headers(&token);
        
        let base_url = self.config.read().await.base_url.clone();
        let ref_param = ref_.unwrap_or("main");
        let url = format!("{}/repos/{}/{}/contents/{}?ref={}", base_url, owner, repo, path, ref_param);
        
        let response = self.client.get(&url).headers(headers.clone()).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to get file content: {}", response.status()));
        }
        
        let data: serde_json::Value = response.json().await?;

        if let Some(content) = data["content"].as_str() {
            let decoded = base64_decode(&content.replace('\n', ""));
            String::from_utf8(decoded).map_err(|e| anyhow!("Failed to decode content: {}", e))
        } else {
            Err(anyhow!("No content found"))
        }
    }

    pub async fn fork_repo(&self, owner: &str, repo: &str) -> Result<GitHubRepo> {
        let token = self.get_token().await.ok_or_else(|| anyhow!("No token set"))?;
        let headers = self.auth_headers(&token);
        
        let base_url = self.config.read().await.base_url.clone();
        let url = format!("{}/repos/{}/{}/forks", base_url, owner, repo);
        
        let response = self.client.post(&url).headers(headers.clone()).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Failed to fork repo: {}", response.status()));
        }
        
        let data: serde_json::Value = response.json().await?;
        
        Ok(GitHubRepo {
            id: data["id"].as_i64().unwrap_or(0),
            name: data["name"].as_str().unwrap_or("").to_string(),
            full_name: data["full_name"].as_str().unwrap_or("").to_string(),
            description: data["description"].as_str().map(String::from),
            html_url: data["html_url"].as_str().unwrap_or("").to_string(),
            clone_url: data["clone_url"].as_str().unwrap_or("").to_string(),
            ssh_url: data["ssh_url"].as_str().unwrap_or("").to_string(),
            default_branch: data["default_branch"].as_str().unwrap_or("main").to_string(),
            language: data["language"].as_str().map(String::from),
            stargazers_count: data["stargazers_count"].as_i64().unwrap_or(0),
            forks_count: data["forks_count"].as_i64().unwrap_or(0),
            open_issues_count: data["open_issues_count"].as_i64().unwrap_or(0),
            owner: data["owner"]["login"].as_str().unwrap_or("").to_string(),
            is_private: data["private"].as_bool().unwrap_or(false),
            is_fork: data["fork"].as_bool().unwrap_or(false),
        })
    }

    pub async fn is_connected(&self) -> bool {
        self.get_token().await.is_some()
    }

    pub fn get_auth_url(&self) -> String {
        "https://github.com/login/oauth/authorize?client_id=github_client_id&scope=repo,user".to_string()
    }

    pub async fn handle_callback(&self, code: &str, _state: &str) -> Result<String> {
        let client = &self.client;
        let response = client
            .post("https://github.com/login/oauth/access_token")
            .header(ACCEPT, "application/json")
            .json(&serde_json::json!({
                "client_id": "github_client_id",
                "client_secret": "github_client_secret",
                "code": code,
            }))
            .send()
            .await?;

        let data: serde_json::Value = response.json().await?;
        let token = data["access_token"].as_str()
            .ok_or_else(|| anyhow!("No access token in response"))?
            .to_string();

        self.set_token(token.clone()).await;
        Ok(token)
    }

    pub async fn disconnect(&self) {
        let mut config = self.config.write().await;
        config.token = None;
    }

    pub async fn get_tree(&self, owner: &str, repo: &str, ref_: Option<&str>) -> Result<Vec<serde_json::Value>> {
        let token = self.get_token().await.ok_or_else(|| anyhow!("No token set"))?;
        let headers = self.auth_headers(&token);
        let base_url = self.config.read().await.base_url.clone();

        let default_branch = self.get_repo(owner, repo).await.map(|r| r.default_branch).unwrap_or_else(|_| "main".to_string());
        let branch = ref_.unwrap_or(&default_branch);

        let url = format!("{}/repos/{}/{}/git/trees/{}?recursive=1", base_url, owner, repo, branch);
        let response = self.client.get(&url).headers(headers).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to get tree: {}", response.status()));
        }

        let data: serde_json::Value = response.json().await?;
        let tree = data["tree"].as_array()
            .map(|arr| arr.iter().map(|item| {
                serde_json::json!({
                    "path": item["path"],
                    "type": item["type"],
                    "size": item["size"],
                    "url": item["url"],
                })
            }).collect())
            .unwrap_or_default();

        Ok(tree)
    }

    pub async fn get_contents(&self, owner: &str, repo: &str, path: &str, ref_: Option<&str>) -> Result<Vec<serde_json::Value>> {
        let token = self.get_token().await.ok_or_else(|| anyhow!("No token set"))?;
        let headers = self.auth_headers(&token);
        let base_url = self.config.read().await.base_url.clone();

        let mut url = format!("{}/repos/{}/{}/contents/{}", base_url, owner, repo, path.trim_start_matches('/'));
        if let Some(r) = ref_ {
            url = format!("{}?ref={}", url, r);
        }

        let response = self.client.get(&url).headers(headers).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to get contents: {}", response.status()));
        }

        let data: serde_json::Value = response.json().await?;

        if data.is_array() {
            Ok(data.as_array().map(|arr| arr.iter()).into_iter().flatten().map(|item| {
                serde_json::json!({
                    "name": item["name"],
                    "path": item["path"],
                    "type": item["type"],
                    "size": item["size"],
                    "url": item["url"],
                    "download_url": item["download_url"],
                })
            }).collect())
        } else {
            Ok(vec![serde_json::json!({
                "name": data["name"],
                "path": data["path"],
                "type": data["type"],
                "size": data["size"],
                "url": data["url"],
                "download_url": data["download_url"],
                "content": data["content"],
                "encoding": data["encoding"],
            })])
        }
    }
}

fn base64_decode(input: &str) -> Vec<u8> {
    let table = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let input = input.as_bytes();
    let mut output = Vec::new();
    
    for chunk in input.chunks(4) {
        let mut buf = [0u8; 4];
        let mut padding = 0;
        
        for (i, &byte) in chunk.iter().enumerate() {
            if byte == b'=' {
                padding += 1;
                buf[i] = 0;
            } else {
                buf[i] = table.iter().position(|&b| b == byte).unwrap_or(0) as u8;
            }
        }
        
        output.push((buf[0] << 2) | (buf[1] >> 4));
        if padding < 2 {
            output.push((buf[1] << 4) | (buf[2] >> 2));
        }
        if padding < 1 {
            output.push((buf[2] << 6) | buf[3]);
        }
    }
    
    output
}

impl Default for GitHubIntegration {
    fn default() -> Self {
        Self::new()
    }
}
