use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::knowledge::KnowledgeBase;

/* ═══════════════════════════════════════════════
   Project Intelligence Engine
   Fetches, analyzes web content (GitHub repos,
   papers, articles), generates skills, and
   recommends project fusion opportunities.
   ═══════════════════════════════════════════════ */

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectProfile {
    pub url: String,
    pub name: String,
    pub description: String,
    pub tech_stack: Vec<String>,
    pub stars: u32,
    pub language: String,
    pub topics: Vec<String>,
    pub recent_commits: u32,
    pub license: String,
    pub readme_preview: String,
    pub analyzed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionSuggestion {
    pub projects: Vec<String>,
    pub rationale: String,
    pub innovation_score: u8,
    pub difficulty: String,
    pub approach: String,
    pub potential_applications: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillBlueprint {
    pub name: String,
    pub description: String,
    pub source_url: String,
    pub category: String,
    pub content: String,
}

pub struct ProjectIntel {
    client: reqwest::Client,
    kb: Arc<KnowledgeBase>,
}

impl ProjectIntel {
    pub fn new(kb: Arc<KnowledgeBase>) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .user_agent("Claude-Desktop-Intel/1.0")
                .build().unwrap_or_default(),
            kb,
        }
    }

    /// Fetch and analyze a GitHub repo or article URL
    pub async fn analyze_url(&self, url: &str) -> Result<ProjectProfile, String> {
        if url.contains("github.com") {
            self.analyze_github(url).await
        } else {
            self.analyze_webpage(url).await
        }
    }

    async fn analyze_github(&self, url: &str) -> Result<ProjectProfile, String> {
        // Extract owner/repo from URL
        let parts: Vec<&str> = url.trim_end_matches('/').split('/').collect();
        if parts.len() < 2 { return Err("Invalid GitHub URL".to_string()); }
        let owner = parts[parts.len() - 2];
        let repo = parts[parts.len() - 1];

        // Fetch GitHub API data
        let api_url = format!("https://api.github.com/repos/{}/{}", owner, repo);
        let resp = self.client.get(&api_url).send().await
            .map_err(|e| format!("GitHub API error: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("GitHub API returned {}", resp.status()));
        }

        let data: serde_json::Value = resp.json().await
            .map_err(|e| format!("Parse error: {}", e))?;

        // Fetch README for analysis
        let readme_url = format!("https://api.github.com/repos/{}/{}/readme", owner, repo);
        let readme_text = match self.client.get(&readme_url)
            .header("Accept", "application/vnd.github.v3.raw")
            .send().await
        {
            Ok(r) => r.text().await.unwrap_or_default(),
            Err(_) => String::new(),
        };

        // Extract topics from data
        let topics: Vec<String> = data.get("topics")
            .and_then(|t| t.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let tech_stack = Self::detect_tech_stack(&readme_text, &topics);

        Ok(ProjectProfile {
            url: url.to_string(),
            name: data.get("full_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            description: data.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            tech_stack,
            stars: data.get("stargazers_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            language: data.get("language").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            topics,
            recent_commits: data.get("size").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            license: data.get("license").and_then(|l| l.get("spdx_id")).and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
            readme_preview: readme_text.chars().take(2000).collect(),
            analyzed_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    async fn analyze_webpage(&self, url: &str) -> Result<ProjectProfile, String> {
        let resp = self.client.get(url).send().await
            .map_err(|e| format!("Fetch error: {}", e))?;
        let html = resp.text().await.unwrap_or_default();

        // Simple title extraction
        let title = extract_html_tag(&html, "title").unwrap_or_else(|| url.to_string());
        let desc = extract_meta(&html, "description").unwrap_or_default();

        Ok(ProjectProfile {
            url: url.to_string(),
            name: title,
            description: desc,
            tech_stack: vec![],
            stars: 0,
            language: "".to_string(),
            topics: vec![],
            recent_commits: 0,
            license: "".to_string(),
            readme_preview: html.chars().take(2000).collect(),
            analyzed_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Find fusion opportunities between multiple projects
    pub fn find_fusions(&self, profiles: &[ProjectProfile]) -> Vec<FusionSuggestion> {
        let mut suggestions = Vec::new();

        if profiles.len() < 2 { return suggestions; }

        for i in 0..profiles.len() {
            for j in (i + 1)..profiles.len() {
                let a = &profiles[i];
                let b = &profiles[j];

                // Check tech stack overlap
                let common_techs: Vec<&String> = a.tech_stack.iter()
                    .filter(|t| b.tech_stack.contains(t))
                    .collect();

                if common_techs.len() >= 2 || (!a.language.is_empty() && a.language == b.language) {
                    let score = ((common_techs.len() as f32 / a.tech_stack.len().max(1) as f32) * 5.0
                        + (common_techs.len() as f32 / b.tech_stack.len().max(1) as f32) * 5.0) as u8;
                    let techs_str = common_techs.iter().map(|s| s.as_str()).collect::<Vec<&str>>().join(", ");

                    suggestions.push(FusionSuggestion {
                        projects: vec![a.name.clone(), b.name.clone()],
                        rationale: format!(
                            "{} 和 {} 都使用 {}，技术栈兼容。融合后可实现 {} + {} 的协同效应。",
                            a.name, b.name,
                            techs_str,
                            a.description.chars().take(50).collect::<String>(),
                            b.description.chars().take(50).collect::<String>()
                        ),
                        innovation_score: score.min(10),
                        difficulty: if score > 7 { "中等" } else { "较高" }.to_string(),
                        approach: format!("将 {} 的核心功能作为模块集成到 {} 的架构中，或反之。", a.name, b.name),
                        potential_applications: vec![
                            format!("{}+{} 融合版本", a.name, b.name),
                            "自动化工作流集成".to_string(),
                            "统一 API 网关".to_string(),
                        ],
                    });
                }
            }
        }

        // Sort by innovation score
        suggestions.sort_by(|a, b| b.innovation_score.cmp(&a.innovation_score));
        suggestions
    }

    /// Generate a skill blueprint from a project profile
    pub async fn generate_skill(&self, profile: &ProjectProfile) -> SkillBlueprint {
        let name = format!("use-{}", profile.name.replace('/', "-").to_lowercase());
        SkillBlueprint {
            name: name.clone(),
            description: format!("Integrate and utilize {} — {}", profile.name, profile.description),
            source_url: profile.url.clone(),
            category: if profile.language == "Python" { "data-science" }
                      else if profile.language == "Rust" { "systems" }
                      else if profile.language == "TypeScript" || profile.language == "JavaScript" { "frontend" }
                      else { "general" }.to_string(),
            content: format!("# {}\n\n## Overview\n{}\n\n## Tech Stack\n- {}\n\n## Integration Guide\n\nThis skill provides tools to work with {}.\n\n## Source\n{}",
                profile.name, profile.description, profile.tech_stack.join(", "), profile.name, profile.url),
        }
    }

    fn detect_tech_stack(readme: &str, topics: &[String]) -> Vec<String> {
        let mut techs: Vec<String> = topics.to_vec();
        let signals = [
            ("React", "react"), ("Vue", "vue"), ("Angular", "angular"),
            ("Node.js", "node"), ("Python", "python"), ("Rust", "rust"),
            ("TypeScript", "typescript"), ("Docker", "docker"), ("Kubernetes", "k8s"),
            ("PostgreSQL", "postgres"), ("Redis", "redis"), ("GraphQL", "graphql"),
            ("WebAssembly", "wasm"), ("TensorFlow", "tensorflow"), ("PyTorch", "pytorch"),
        ];
        let lower = readme.to_lowercase();
        for (name, signal) in &signals {
            if lower.contains(signal) && !techs.contains(&name.to_string()) {
                techs.push(name.to_string());
            }
        }
        techs
    }
}

fn extract_html_tag(html: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    html.find(&open).and_then(|s| {
        let start = s + open.len();
        html[start..].find(&close).map(|e| html[start..start + e].to_string())
    })
}

fn extract_meta(html: &str, name: &str) -> Option<String> {
    let pattern = format!(r#"name="{}""#, name);
    html.find(&pattern).and_then(|s| {
        let content_start = s + pattern.len();
        if let Some(cpos) = html[content_start..].find("content=\"") {
            let val_start = content_start + cpos + 9;
            html[val_start..].find('"').map(|e| html[val_start..val_start + e].to_string())
        } else { None }
    })
}
