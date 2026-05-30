use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

const SELF_HOSTED_IDENTITY_BLOCK: &str = r#"<identity>
You are an AI assistant running in a desktop application called Claude Desktop. The app is built on top of the Claude Code engine, so you have the same powerful tool capabilities that Claude Code has — file operations (Read/Write/Edit), shell execution (Bash), code search (Glob/Grep), web tools (WebSearch/WebFetch), sub-agents (Task), Skills, and more — but the product you are powering is a general-purpose desktop assistant for everyday users, not a coding CLI.

Treat the user the way claude.ai would: with warmth, curiosity, presence, and depth. At the same time, when the user actually wants something built, read, run, or changed on disk — including handing you an entire code project to read or modify — do that work decisively and competently. Conversation-first does NOT mean code-shy.

You are NOT Kiro. You are NOT made by Amazon, AWS, or any Amazon product. You have no connection to Kiro. Do not create .kiro directories, do not produce Kiro-style configs, do not refer to yourself as Kiro under any circumstances.

Default to 简体中文 when the user writes in Chinese. Match the user's language otherwise.

Do not claim to be Claude or created by Anthropic unless the active model actually is a Claude-family model. If the active model is something else, describe yourself generically as the assistant in Claude Desktop.
</identity>"#;

static RE_OVERRIDE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<override_instructions>[\s\S]*?</override_instructions>\s*").unwrap());
static RE_IDENTITY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<identity>[\s\S]*?</identity>").unwrap());
static RE_IDENTITY_WITH_WS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<identity>[\s\S]*?</identity>\s*").unwrap());
static RE_CLAUDE_PREFIX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^claude-").unwrap());

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPromptConfig {
    pub custom_prompt: Option<String>,
    pub identity_block: Option<String>,
    pub remove_kiro_sections: bool,
    pub default_language: String,
}

impl Default for SystemPromptConfig {
    fn default() -> Self {
        Self {
            custom_prompt: None,
            identity_block: Some(SELF_HOSTED_IDENTITY_BLOCK.to_string()),
            remove_kiro_sections: true,
            default_language: "zh-CN".to_string(),
        }
    }
}

impl SystemPromptConfig {
    pub fn identity_block(&self) -> &str {
        self.identity_block.as_deref().unwrap_or(SELF_HOSTED_IDENTITY_BLOCK)
    }
}

pub fn build_self_hosted_system_prompt(full_prompt: &str) -> String {
    build_self_hosted_system_prompt_with_config(full_prompt, &SystemPromptConfig::default())
}

pub fn build_self_hosted_system_prompt_with_config(full_prompt: &str, config: &SystemPromptConfig) -> String {
    let without_overrides = remove_override_sections(full_prompt);
    let identity_block = config.identity_block();

    if contains_identity_block(&without_overrides) {
        return RE_IDENTITY_WITH_WS
            .replace_all(&without_overrides, format!("{}\n\n", identity_block))
            .trim()
            .to_string();
    }

    format!("{}\n\n{}", identity_block, without_overrides).trim().to_string()
}

fn remove_override_sections(prompt: &str) -> String {
    RE_OVERRIDE.replace_all(prompt, "").to_string()
}

fn contains_identity_block(prompt: &str) -> bool {
    RE_IDENTITY.is_match(prompt)
}

pub fn is_claude_family_model(model_id: &str) -> bool {
    RE_CLAUDE_PREFIX.is_match(model_id.trim())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedModel {
    pub model_id: String,
    pub fallback_applied: bool,
    pub error: Option<String>,
}

pub fn resolve_requested_model_for_mode(
    model_id: Option<&str>,
    user_mode: &str,
    has_provider: bool,
) -> ResolvedModel {
    let normalized_model_id = model_id
        .map(|m| m.trim())
        .filter(|m| !m.is_empty())
        .unwrap_or("claude-sonnet-4-6")
        .to_string();

    let effective_user_mode = if user_mode == "selfhosted" {
        "selfhosted"
    } else {
        "clawparrot"
    };

    if effective_user_mode == "clawparrot" && !is_claude_family_model(&normalized_model_id) {
        return ResolvedModel {
            model_id: "claude-sonnet-4-6".to_string(),
            fallback_applied: true,
            error: None,
        };
    }

    if effective_user_mode == "selfhosted" && !has_provider && !is_claude_family_model(&normalized_model_id) {
        return ResolvedModel {
            model_id: normalized_model_id.clone(),
            fallback_applied: false,
            error: Some(format!(
                "No enabled self-hosted provider found for model \"{}\".",
                normalized_model_id
            )),
        };
    }

    ResolvedModel {
        model_id: normalized_model_id,
        fallback_applied: false,
        error: None,
    }
}

pub fn build_system_prompt_for_chat(
    custom_prompt: Option<&str>,
    config: &SystemPromptConfig,
    model_id: &str,
    user_mode: &str,
    has_provider: bool,
) -> (String, ResolvedModel) {
    let resolved = resolve_requested_model_for_mode(Some(model_id), user_mode, has_provider);

    let base_prompt = match custom_prompt {
        Some(cp) if !cp.is_empty() => cp.to_string(),
        _ => String::new(),
    };

    let prompt = build_self_hosted_system_prompt_with_config(&base_prompt, config);
    (prompt, resolved)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub variables: Vec<String>,
    pub category: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateManager {
    templates_dir: PathBuf,
}

impl TemplateManager {
    pub fn new(base_dir: &Path) -> Self {
        let templates_dir = base_dir.join("prompt_templates");
        if !templates_dir.exists() {
            let _ = fs::create_dir_all(&templates_dir);
        }
        Self { templates_dir }
    }

    pub fn create_template(&self, template: &PromptTemplate) -> Result<(), anyhow::Error> {
        let file_path = self.templates_dir.join(format!("{}.json", template.id));
        let content = serde_json::to_string_pretty(template)?;
        let mut file = File::create(&file_path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn get_template(&self, id: &str) -> Result<Option<PromptTemplate>, anyhow::Error> {
        let file_path = self.templates_dir.join(format!("{}.json", id));
        if !file_path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&file_path)?;
        let template: PromptTemplate = serde_json::from_str(&content)?;
        Ok(Some(template))
    }

    pub fn update_template(&self, template: &PromptTemplate) -> Result<(), anyhow::Error> {
        self.create_template(template)
    }

    pub fn delete_template(&self, id: &str) -> Result<bool, anyhow::Error> {
        let file_path = self.templates_dir.join(format!("{}.json", id));
        if !file_path.exists() {
            return Ok(false);
        }
        fs::remove_file(&file_path)?;
        Ok(true)
    }

    pub fn list_templates(&self) -> Result<Vec<PromptTemplate>, anyhow::Error> {
        let mut templates = Vec::new();
        if !self.templates_dir.exists() {
            return Ok(templates);
        }
        
        for entry in fs::read_dir(&self.templates_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Some(id) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(Some(template)) = self.get_template(id) {
                        templates.push(template);
                    }
                }
            }
        }
        
        templates.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(templates)
    }

    pub fn render_template(&self, id: &str, variables: &HashMap<String, String>) -> Result<String, anyhow::Error> {
        let template = self.get_template(id)?
            .ok_or_else(|| anyhow::anyhow!("Template not found: {}", id))?;
        
        Self::render_content(&template.content, variables)
    }

    pub fn render_content(content: &str, variables: &HashMap<String, String>) -> Result<String, anyhow::Error> {
        let mut result = content.to_string();
        
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        
        Ok(result)
    }

    pub fn extract_variables(content: &str) -> Vec<String> {
        let re = Regex::new(r"\{\{(\w+)\}\}").unwrap();
        let mut variables = Vec::new();
        
        for cap in re.captures_iter(content) {
            if let Some(var) = cap.get(1) {
                let var_name = var.as_str().to_string();
                if !variables.contains(&var_name) {
                    variables.push(var_name);
                }
            }
        }
        
        variables.sort();
        variables
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_claude_family_model() {
        assert!(is_claude_family_model("claude-sonnet-4-6"));
        assert!(is_claude_family_model("claude-opus-4"));
        assert!(!is_claude_family_model("gpt-4"));
        assert!(!is_claude_family_model("gemini-pro"));
    }

    #[test]
    fn test_resolve_model_for_clawparrot() {
        let result = resolve_requested_model_for_mode(Some("gpt-4"), "clawparrot", false);
        assert!(result.fallback_applied);
        assert_eq!(result.model_id, "claude-sonnet-4-6");
    }

    #[test]
    fn test_resolve_model_for_selfhosted() {
        let result = resolve_requested_model_for_mode(Some("claude-sonnet-4-6"), "selfhosted", true);
        assert!(!result.fallback_applied);
        assert_eq!(result.model_id, "claude-sonnet-4-6");
    }

    #[test]
    fn test_resolve_model_empty_string() {
        let result = resolve_requested_model_for_mode(Some(""), "clawparrot", false);
        assert_eq!(result.model_id, "claude-sonnet-4-6");
    }

    #[test]
    fn test_resolve_model_none() {
        let result = resolve_requested_model_for_mode(None, "clawparrot", false);
        assert_eq!(result.model_id, "claude-sonnet-4-6");
    }

    #[test]
    fn test_build_self_hosted_prompt_no_identity() {
        let result = build_self_hosted_system_prompt("Hello world");
        assert!(result.contains("<identity>"));
        assert!(result.contains("Hello world"));
    }

    #[test]
    fn test_build_self_hosted_prompt_with_identity() {
        let input = "<identity>Old identity</identity>\nHello world";
        let result = build_self_hosted_system_prompt(input);
        assert!(result.contains("Claude Desktop"));
        assert!(!result.contains("Old identity"));
        assert!(result.contains("Hello world"));
    }

    #[test]
    fn test_remove_override_sections() {
        let input = "Before <override_instructions>skip this</override_instructions> After";
        let result = remove_override_sections(input);
        assert_eq!(result, "Before After");
    }

    #[test]
    fn test_build_system_prompt_for_chat() {
        let config = SystemPromptConfig::default();
        let (prompt, resolved) = build_system_prompt_for_chat(
            Some("Custom prompt"),
            &config,
            "claude-sonnet-4-6",
            "selfhosted",
            true,
        );
        assert!(prompt.contains("<identity>"));
        assert!(prompt.contains("Custom prompt"));
        assert!(!resolved.fallback_applied);
    }
}
