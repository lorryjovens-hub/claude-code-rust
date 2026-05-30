pub mod engine;
pub mod open_design;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub use engine::{SkillExecutionContext, SkillExecutionEngine};
pub use open_design::{OpenDesignMetadata, OpenDesignSkill};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub content: Option<String>,
    #[serde(rename = "whenToUse")]
    pub when_to_use: Option<String>,
    pub enabled: bool,
    pub source: SkillSource,
    #[serde(rename = "loadedFrom")]
    pub loaded_from: String,
    #[serde(rename = "sourceDir")]
    pub source_dir: Option<String>,
    #[serde(rename = "isExample")]
    pub is_example: bool,
    pub files: Vec<SkillFile>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "odMetadata")]
    pub od_metadata: Option<OpenDesignMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SkillSource {
    User,
    Project,
    Plugin,
    Bundled,
    Mcp,
}

impl SkillSource {
    pub fn as_str(&self) -> &str {
        match self {
            SkillSource::User => "user",
            SkillSource::Project => "project",
            SkillSource::Plugin => "plugin",
            SkillSource::Bundled => "bundled",
            SkillSource::Mcp => "mcp",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFile {
    pub name: String,
    #[serde(rename = "type")]
    pub file_type: String,
    pub children: Option<Vec<SkillFile>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontmatterData {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "whenToUse")]
    pub when_to_use: Option<String>,
    #[serde(rename = "allowedTools")]
    pub allowed_tools: Option<Vec<String>>,
    pub model: Option<String>,
    #[serde(rename = "userInvocable")]
    pub user_invokable: Option<bool>,
    pub triggers: Option<Vec<String>>,
    pub od: Option<OpenDesignMetadata>,
}

pub struct SkillsManager {
    skills_dir: PathBuf,
}

impl SkillsManager {
    pub fn new() -> Self {
        let skills_dir = Self::get_skills_base_path();
        Self { skills_dir }
    }

    fn get_skills_base_path() -> PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".claude").join("skills")
    }

    pub fn get_skills_path(&self, source: &SkillSource) -> PathBuf {
        match source {
            SkillSource::User => self.skills_dir.clone(),
            SkillSource::Project => PathBuf::from(".claude/skills"),
            SkillSource::Plugin => PathBuf::from("plugin"),
            SkillSource::Bundled => self.skills_dir.join("bundled"),
            SkillSource::Mcp => self.skills_dir.join("mcp"),
        }
    }

    pub async fn load_skills(&self) -> Result<Vec<Skill>> {
        let mut skills = Vec::new();

        let user_skills_path = self.get_skills_path(&SkillSource::User);
        if user_skills_path.exists() {
            skills.extend(self.scan_skills_dir(&user_skills_path, SkillSource::User)?);
            // 扫描用户技能目录下的子目录（如gstack）
            skills.extend(self.scan_skills_subdirs(&user_skills_path)?);
        }

        let bundled_skills_path = self.get_skills_path(&SkillSource::Bundled);
        if bundled_skills_path.exists() {
            skills.extend(self.scan_skills_dir(&bundled_skills_path, SkillSource::Bundled)?);
        } else {
            self.install_bundled_skills()?;
            if bundled_skills_path.exists() {
                skills.extend(self.scan_skills_dir(&bundled_skills_path, SkillSource::Bundled)?);
            }
        }

        Ok(skills)
    }

    fn scan_skills_subdirs(&self, base_path: &Path) -> Result<Vec<Skill>> {
        let mut skills = Vec::new();

        if let Ok(entries) = std::fs::read_dir(base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    // 跳过隐藏目录和已处理的目录
                    if dir_name.starts_with('.') || dir_name == "bundled" || dir_name == "mcp" {
                        continue;
                    }
                    // 扫描子目录中的技能（如gstack）
                    skills.extend(self.scan_skills_dir(&path, SkillSource::User)?);
                }
            }
        }

        Ok(skills)
    }

    pub fn install_bundled_skills(&self) -> Result<()> {
        let bundled_path = self.get_skills_path(&SkillSource::Bundled);
        std::fs::create_dir_all(&bundled_path)?;

        let skill_creator_md = r#"---
name: skill-creator
description: A toolkit for creating, evaluating, and improving Claude skills
whenToUse: When you want to build a new skill, evaluate an existing one, or generate synthetic training data for skills
---

# Skill Creator

A comprehensive toolkit for creating, evaluating, and improving Claude skills.

## Usage

This skill provides utilities for:
- Creating new skills from templates
- Evaluating skill quality
- Generating training data
- Packaging skills for distribution

## Files

- `agents/` - Agent definitions for different evaluation tasks
- `scripts/` - Utility scripts for skill development
- `eval-viewer/` - Evaluation result visualization
- `references/` - Schema definitions and documentation
"#;

        std::fs::write(bundled_path.join("skill-creator").join("SKILL.md"), skill_creator_md)?;
        std::fs::create_dir_all(bundled_path.join("skill-creator").join("agents"))?;
        std::fs::write(bundled_path.join("skill-creator").join("agents").join("analyzer.md"), "# Analyzer\n\nAnalyzes skill structure and quality.")?;
        std::fs::write(bundled_path.join("skill-creator").join("agents").join("grader.md"), "# Grader\n\nGrades skill performance.")?;

        Ok(())
    }

    fn scan_skills_dir(&self, base_path: &Path, source: SkillSource) -> Result<Vec<Skill>> {
        let mut skills = Vec::new();

        if !base_path.exists() {
            return Ok(skills);
        }

        for entry in WalkDir::new(base_path)
            .max_depth(3)
            .follow_links(false)
        {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            if path.is_file() && path.file_name().map(|n| n == "SKILL.md").unwrap_or(false) {
                if let Some(skill) = self.parse_skill_file(path, &source)? {
                    skills.push(skill);
                }
            }
        }

        Ok(skills)
    }

    fn parse_skill_file(&self, skill_md_path: &Path, source: &SkillSource) -> Result<Option<Skill>> {
        let path_display = skill_md_path.display().to_string();
        tracing::debug!(module = "SkillsManager::parse", "🔍 解析 SKILL.md: {}", path_display);

        let content = match std::fs::read_to_string(skill_md_path) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(module = "SkillsManager::parse", "⚠️ 无法读取文件: {} ({})", path_display, e);
                return Ok(None);
            }
        };

        let frontmatter = self.parse_frontmatter(&content);

        let name = frontmatter.name.or_else(|| {
            skill_md_path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        });

        let name = match name {
            Some(n) => n,
            None => {
                tracing::warn!(module = "SkillsManager::parse", "⚠️ 无法确定技能名称: {}", path_display);
                return Ok(None);
            }
        };

        if frontmatter.od.is_some() {
            tracing::info!(
                module = "SkillsManager::parse",
                "🎨 检测到设计技能: {} (mode={:?}, scenario={:?})",
                name,
                frontmatter.od.as_ref().and_then(|od| od.mode.as_ref()),
                frontmatter.od.as_ref().and_then(|od| od.scenario.as_ref()),
            );
        }

        let skill_dir = skill_md_path.parent();
        let files = skill_dir.map(|d| self.build_file_tree(d)).unwrap_or_default();

        let skill = Skill {
            id: Self::generate_skill_id(skill_md_path),
            name,
            description: frontmatter.description.unwrap_or_default(),
            content: Some(content),
            when_to_use: frontmatter.when_to_use,
            enabled: true,
            source: source.clone(),
            loaded_from: source.as_str().to_string(),
            source_dir: skill_md_path
                .parent()
                .map(|p| p.to_string_lossy().to_string()),
            is_example: false,
            files,
            created_at: None,
            od_metadata: frontmatter.od,
        };

        Ok(Some(skill))
    }

    fn parse_frontmatter(&self, content: &str) -> FrontmatterData {
        let content_trimmed = content.trim_start();
        if !content_trimmed.starts_with("---") {
            return FrontmatterData::default();
        }

        if let Some(end_pos) = content_trimmed[3..].find("---") {
            let yaml_block = &content_trimmed[3..end_pos + 3];

            match serde_yaml::from_str(yaml_block) {
                Ok(frontmatter) => return frontmatter,
                Err(e) => {
                    tracing::warn!("Failed to parse frontmatter with serde_yaml: {}, falling back to manual parsing", e);
                }
            }
        }

        self.parse_frontmatter_fallback(content)
    }

    fn parse_frontmatter_fallback(&self, content: &str) -> FrontmatterData {
        let mut frontmatter = FrontmatterData::default();

        let content_trimmed = content.trim_start();
        if !content_trimmed.starts_with("---") {
            return frontmatter;
        }

        if let Some(end_pos) = content_trimmed[3..].find("---") {
            let yaml_block = &content_trimmed[3..end_pos + 3];

            let mut lines = yaml_block.lines().peekable();
            let mut current_key: Option<String> = None;
            let mut current_value_lines: Vec<String> = Vec::new();

            while let Some(line) = lines.next() {
                let line = line.trim();
                if line.is_empty() || line == "---" {
                    if let Some(key) = current_key.take() {
                        let value = current_value_lines.join("\n").trim().to_string();
                        self.set_frontmatter_field(&mut frontmatter, &key, &value);
                        current_value_lines.clear();
                    }
                    continue;
                }

                if line.starts_with('#') {
                    continue;
                }

                if let Some((key_part, value_part)) = line.split_once(':') {
                    if let Some(key) = current_key.take() {
                        let value = current_value_lines.join("\n").trim().to_string();
                        self.set_frontmatter_field(&mut frontmatter, &key, &value);
                        current_value_lines.clear();
                    }

                    let key = key_part.trim();
                    let value_part = value_part.trim();

                    if value_part.starts_with('|') {
                        current_key = Some(key.to_string());
                        while let Some(next_line) = lines.peek() {
                            let next_line_trimmed = next_line.trim();
                            if !next_line_trimmed.is_empty() &&
                               !next_line_trimmed.starts_with(' ') &&
                               !next_line_trimmed.starts_with('-') &&
                               next_line_trimmed.contains(':') {
                                break;
                            }
                            let consumed_line = lines.next().unwrap();
                            if !consumed_line.trim().starts_with('|') {
                                current_value_lines.push(consumed_line.trim().to_string());
                            }
                        }
                    } else {
                        let value = value_part.trim_matches('"').trim_matches('\'');
                        self.set_frontmatter_field(&mut frontmatter, key, value);
                    }
                } else if line.starts_with('-') && current_key.is_some() {
                    current_value_lines.push(line.to_string());
                } else if current_key.is_some() {
                    current_value_lines.push(line.to_string());
                }
            }

            if let Some(key) = current_key {
                let value = current_value_lines.join("\n").trim().to_string();
                self.set_frontmatter_field(&mut frontmatter, &key, &value);
            }
        }

        frontmatter
    }

    fn set_frontmatter_field(&self, frontmatter: &mut FrontmatterData, key: &str, value: &str) {
        match key {
            "name" => frontmatter.name = Some(value.to_string()),
            "description" => frontmatter.description = Some(value.replace('\n', " ").trim().to_string()),
            "when" | "whenToUse" | "when_to_use" => frontmatter.when_to_use = Some(value.to_string()),
            "model" => frontmatter.model = Some(value.to_string()),
            "userInvocable" | "user_invocable" => {
                frontmatter.user_invokable = Some(value == "true" || value == "yes");
            }
            _ => {}
        }
    }

    fn build_file_tree(&self, dir: &Path) -> Vec<SkillFile> {
        let mut files = Vec::new();

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string();

                if name.starts_with('.') || name == "node_modules" {
                    continue;
                }

                let file_type = if path.is_dir() { "folder" } else { "file" };

                let children = if path.is_dir() {
                    Some(self.build_file_tree(&path))
                } else {
                    None
                };

                files.push(SkillFile {
                    name,
                    file_type: file_type.to_string(),
                    children,
                });
            }
        }

        files.sort_by(|a, b| {
            match (&a.file_type[..], &b.file_type[..]) {
                ("folder", "file") => std::cmp::Ordering::Less,
                ("file", "folder") => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        files
    }

    fn generate_skill_id(skill_path: &Path) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        skill_path.to_string_lossy().hash(&mut hasher);
        format!("skill_{:x}", hasher.finish())
    }

    pub fn get_skill_content(&self, skill_id: &str, file_path: &str) -> Result<String> {
        let skill_path = self.find_skill_path(skill_id)?;
        let full_path = skill_path.join(file_path);
        let content = std::fs::read_to_string(&full_path)?;
        Ok(content)
    }

    pub fn create_skill(&self, name: &str, description: &str, content: &str) -> Result<Skill> {
        let skill_dir = self.skills_dir.join(name);
        std::fs::create_dir_all(&skill_dir)?;

        let skill_md = format!(
            "---\nname: {}\ndescription: {}\n---\n\n{}",
            name, description, content
        );
        std::fs::write(skill_dir.join("SKILL.md"), &skill_md)?;

        let skill = Skill {
            id: Self::generate_skill_id(&skill_dir.join("SKILL.md")),
            name: name.to_string(),
            description: description.to_string(),
            content: Some(skill_md),
            when_to_use: None,
            enabled: true,
            source: SkillSource::User,
            loaded_from: "user".to_string(),
            source_dir: Some(skill_dir.to_string_lossy().to_string()),
            is_example: false,
            files: vec![SkillFile {
                name: "SKILL.md".to_string(),
                file_type: "file".to_string(),
                children: None,
            }],
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            od_metadata: None,
        };

        Ok(skill)
    }

    pub fn update_skill(&self, skill_id: &str, updates: HashMap<String, serde_json::Value>) -> Result<()> {
        let skill_path = self.find_skill_path(skill_id)?;
        let skill_md_path = skill_path.join("SKILL.md");

        if !skill_md_path.exists() {
            anyhow::bail!("SKILL.md not found");
        }

        let content = std::fs::read_to_string(&skill_md_path)?;

        let mut new_content = content.clone();
        if let Some(name) = updates.get("name").and_then(|v| v.as_str()) {
            new_content = Self::update_yaml_field(&new_content, "name", name);
        }
        if let Some(description) = updates.get("description").and_then(|v| v.as_str()) {
            new_content = Self::update_yaml_field(&new_content, "description", description);
        }

        std::fs::write(&skill_md_path, new_content)?;
        Ok(())
    }

    fn update_yaml_field(content: &str, field: &str, value: &str) -> String {
        let pattern = format!("^{}:", field);
        if let Some(idx) = content.lines().position(|l| l.trim().starts_with(&pattern)) {
            let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
            lines[idx] = format!("{}: {}", field, value);
            lines.join("\n")
        } else {
            content.to_string()
        }
    }

    fn find_skill_path(&self, skill_id: &str) -> Result<PathBuf> {
        for entry in WalkDir::new(&self.skills_dir)
            .max_depth(2)
            .follow_links(false)
        {
            if let Ok(entry) = entry {
                if entry.path().file_name().map(|n| n == "SKILL.md").unwrap_or(false) {
                    let id = Self::generate_skill_id(entry.path());
                    if id == skill_id {
                        return Ok(entry.path().parent()
                            .ok_or_else(|| anyhow!("Skill path has no parent directory"))?
                            .to_path_buf());
                    }
                }
            }
        }
        anyhow::bail!("Skill not found: {}", skill_id)
    }

    pub fn delete_skill(&self, skill_id: &str) -> Result<()> {
        let skill_path = self.find_skill_path(skill_id)?;
        std::fs::remove_dir_all(skill_path)?;
        Ok(())
    }

    pub async fn get_skill_by_id(&self, skill_id: &str) -> Result<Option<Skill>> {
        let skills = self.load_skills().await?;
        Ok(skills.into_iter().find(|s| s.id == skill_id))
    }

    pub async fn install_skill_from_url(&self, url: &str) -> Result<Skill> {
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to download skill: HTTP {}", response.status());
        }

        let content = response.text().await?;
        let frontmatter = self.parse_frontmatter(&content);

        let name = frontmatter.name.ok_or_else(|| anyhow::anyhow!("Skill name not found in downloaded content"))?;
        let description = frontmatter.description.unwrap_or_default();

        self.create_skill(&name, &description, &content)
    }

    pub async fn execute_skill(&self, skill_id: &str, _input: &str, context: Option<SkillExecutionContext>) -> Result<String> {
        let skills = self.load_skills().await?;
        let skill = skills.into_iter()
            .find(|s| s.id == skill_id && s.enabled)
            .ok_or_else(|| anyhow::anyhow!("Skill not found or disabled"))?;

        let content = skill.content.ok_or_else(|| anyhow::anyhow!("Skill has no content"))?;

        let ctx = context.unwrap_or_default();
        let result = SkillExecutionEngine::execute(&content, &ctx).await?;

        Ok(result.summary)
    }

    pub async fn list_enabled_skills(&self) -> Result<Vec<Skill>> {
        let skills = self.load_skills().await?;
        Ok(skills.into_iter().filter(|s| s.enabled).collect())
    }

    pub async fn toggle_skill(&self, skill_id: &str) -> Result<bool> {
        let skills = self.load_skills().await?;
        let skill = skills.into_iter()
            .find(|s| s.id == skill_id)
            .ok_or_else(|| anyhow::anyhow!("Skill not found"))?;

        let new_state = !skill.enabled;
        let skill_path = self.find_skill_path(skill_id)?;
        let skill_md_path = skill_path.join("SKILL.md");

        if skill_md_path.exists() {
            let content = std::fs::read_to_string(&skill_md_path)?;
            let updated = content.replace(
                &format!("enabled: {}", !new_state),
                &format!("enabled: {}", new_state)
            );
            std::fs::write(&skill_md_path, updated)?;
        }

        Ok(new_state)
    }

    pub async fn install_superpowers_skills(&self) -> Result<SuperpowersInstallResult> {
        let temp_dir = std::env::temp_dir().join("superpowers_install");
        std::fs::create_dir_all(&temp_dir)?;

        let npm_path = self.find_npm_path();

        let output = tokio::process::Command::new(npm_path)
            .args(&["pack", "@complexthings/superpowers-agent", "--pack-destination", temp_dir.to_str().unwrap_or(".")])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("npm pack failed: {}", stderr));
        }

        let package_dir = temp_dir.join("package");
        std::fs::create_dir_all(&package_dir)?;

        let tarball = std::fs::read_dir(&temp_dir)?
            .filter_map(|e| e.ok())
            .find(|e| {
                e.path().file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("complexthings-superpowers-agent"))
                    .unwrap_or(false)
            })
            .map(|e| e.path());

        if let Some(tarball) = tarball {
            let tar_path = self.find_tar_path();
            let _ = tokio::process::Command::new(tar_path)
                .args(&["-xzf", tarball.to_str().unwrap_or(""), "-C", package_dir.to_str().unwrap_or(".")])
                .output()
                .await;
        }

        let mut result = SuperpowersInstallResult {
            installed_count: 0,
            failed_count: 0,
            errors: Vec::new(),
        };

        let skills_path = package_dir.join("skills");
        if skills_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&skills_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        let category = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        if let Ok(sub_entries) = std::fs::read_dir(&path) {
                            for sub_entry in sub_entries.filter_map(|e| e.ok()) {
                                let sub_path = sub_entry.path();
                                if sub_path.is_dir() {
                                    let skill_md_path = sub_path.join("SKILL.md");
                                    if skill_md_path.exists() {
                                        if let Ok(content) = std::fs::read_to_string(&skill_md_path) {
                                            let skill_name = Self::extract_skill_name_from_content(&content)
                                                .unwrap_or_else(|| category.clone());
                                            let skill_id = format!("superpowers-{}-{}", category, skill_name.to_lowercase().replace(' ', "-"));

                                            let dest_dir = self.skills_dir.join("bundled").join(&skill_id);
                                            match std::fs::create_dir_all(&dest_dir) {
                                                Ok(_) => {
                                                    if std::fs::write(dest_dir.join("SKILL.md"), &content).is_ok() {
                                                        result.installed_count += 1;
                                                    } else {
                                                        result.failed_count += 1;
                                                        result.errors.push(format!("Failed to write skill: {}", skill_id));
                                                    }
                                                }
                                                Err(e) => {
                                                    result.failed_count += 1;
                                                    result.errors.push(format!("Failed to create directory for {}: {}", skill_id, e));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let _ = std::fs::remove_dir_all(&temp_dir);

        Ok(result)
    }

    fn extract_skill_name_from_content(content: &str) -> Option<String> {
        let content_trimmed = content.trim_start();
        if !content_trimmed.starts_with("---") {
            return None;
        }

        if let Some(end_pos) = content_trimmed[3..].find("---") {
            let yaml_block = &content_trimmed[3..end_pos];
            for line in yaml_block.lines() {
                let line = line.trim();
                if line.starts_with("name:") {
                    let value = line.split(':').nth(1)?.trim();
                    return Some(value.trim_matches('"').trim_matches('\'').to_string());
                }
            }
        }
        None
    }

    pub fn get_superpowers_categories() -> Vec<SuperpowersCategory> {
        vec![
            SuperpowersCategory {
                id: "testing".to_string(),
                name: "Testing Skills".to_string(),
                description: "TDD, async testing, anti-patterns, verification".to_string(),
                skills_count: 4,
            },
            SuperpowersCategory {
                id: "debugging".to_string(),
                name: "Debugging Skills".to_string(),
                description: "Systematic debugging, root cause tracing, defense in depth".to_string(),
                skills_count: 3,
            },
            SuperpowersCategory {
                id: "collaboration".to_string(),
                name: "Collaboration Skills".to_string(),
                description: "Brainstorming, planning, code review, parallel agents".to_string(),
                skills_count: 6,
            },
            SuperpowersCategory {
                id: "development".to_string(),
                name: "Development Skills".to_string(),
                description: "Git worktrees, finishing branches, subagent workflows".to_string(),
                skills_count: 4,
            },
            SuperpowersCategory {
                id: "meta".to_string(),
                name: "Meta Skills".to_string(),
                description: "Creating, testing, and sharing skills".to_string(),
                skills_count: 3,
            },
        ]
    }

    fn find_npm_path(&self) -> String {
        #[cfg(target_os = "windows")]
        {
            if let Ok(path) = std::env::var("PATH") {
                for dir in path.split(';') {
                    let npm_path = std::path::PathBuf::from(dir).join("npm.cmd");
                    if npm_path.exists() {
                        return npm_path.to_string_lossy().to_string();
                    }
                    let npm_path = std::path::PathBuf::from(dir).join("npm");
                    if npm_path.exists() {
                        return npm_path.to_string_lossy().to_string();
                    }
                }
            }
            if let Ok(home) = std::env::var("USERPROFILE") {
                let npm_path = std::path::PathBuf::from(home)
                    .join("AppData")
                    .join("Roaming")
                    .join("npm")
                    .join("npm.cmd");
                if npm_path.exists() {
                    return npm_path.to_string_lossy().to_string();
                }
            }
        }
        "npm".to_string()
    }

    fn find_tar_path(&self) -> String {
        #[cfg(target_os = "windows")]
        {
            if let Ok(path) = std::env::var("PATH") {
                for dir in path.split(';') {
                    let tar_path = std::path::PathBuf::from(dir).join("tar.exe");
                    if tar_path.exists() {
                        return tar_path.to_string_lossy().to_string();
                    }
                }
            }
            if let Ok(program_files) = std::env::var("PROGRAMFILES") {
                let tar_path = std::path::PathBuf::from(program_files)
                    .join("Git")
                    .join("usr")
                    .join("bin")
                    .join("tar.exe");
                if tar_path.exists() {
                    return tar_path.to_string_lossy().to_string();
                }
            }
        }
        "tar".to_string()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SuperpowersInstallResult {
    pub installed_count: usize,
    pub failed_count: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SuperpowersCategory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub skills_count: usize,
}

impl Default for SkillsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for FrontmatterData {
    fn default() -> Self {
        FrontmatterData {
            name: None,
            description: None,
            when_to_use: None,
            allowed_tools: None,
            model: None,
            user_invokable: None,
            triggers: None,
            od: None,
        }
    }
}
