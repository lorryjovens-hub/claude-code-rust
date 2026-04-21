//! Tests for Skills Framework

use claude_code_rs::skills::{SkillRegistry, SkillExecutor, SkillContext, SkillCategory, BuiltinSkills};
use std::sync::Arc;

#[test]
fn test_skill_registry_creation() {
    let registry = SkillRegistry::new();
    assert!(registry.list_names().is_empty());
}

#[test]
fn test_skill_registration() {
    let mut registry = SkillRegistry::new();

    // Register built-in skills
    for (skill, categories) in BuiltinSkills::all() {
        registry.register(Arc::new(skill), categories);
    }

    // Should have 5 skills now
    assert_eq!(registry.list_names().len(), 5);

    // Check specific skills exist
    assert!(registry.has("commit"));
    assert!(registry.has("review"));
    assert!(registry.has("test"));
    assert!(registry.has("document"));
    assert!(registry.has("build"));
}

#[test]
fn test_skill_search() {
    let mut registry = SkillRegistry::new();

    // Register built-in skills
    for (skill, categories) in BuiltinSkills::all() {
        registry.register(Arc::new(skill), categories);
    }

    // Search for "commit"
    let results = registry.search("commit");
    assert!(!results.is_empty());
}

#[test]
fn test_skill_categories() {
    let mut registry = SkillRegistry::new();

    // Register built-in skills
    for (skill, categories) in BuiltinSkills::all() {
        registry.register(Arc::new(skill), categories);
    }

    // Check Git category
    let git_skills = registry.list_by_category(SkillCategory::Git);
    assert!(!git_skills.is_empty());

    // Check Utility category (should have multiple)
    let utility_skills = registry.list_by_category(SkillCategory::Utility);
    assert_eq!(utility_skills.len(), 5);
}

#[tokio::test]
async fn test_skill_executor() {
    let mut registry = SkillRegistry::new();

    // Register built-in skills
    for (skill, categories) in BuiltinSkills::all() {
        registry.register(Arc::new(skill), categories);
    }

    let registry_arc = Arc::new(registry);
    let executor = SkillExecutor::new(registry_arc);

    // List skills
    let skills = executor.list_skills();
    assert_eq!(skills.len(), 5);

    // Execute commit skill
    let context = SkillContext {
        cwd: ".".to_string(),
        env: std::collections::HashMap::new(),
        tool_registry: None,
        data: std::collections::HashMap::new(),
    };

    let result = executor.execute("commit", "", context).await;
    assert!(result.is_ok());
}

#[test]
fn test_skill_help() {
    let mut registry = SkillRegistry::new();

    // Register built-in skills
    for (skill, categories) in BuiltinSkills::all() {
        registry.register(Arc::new(skill), categories);
    }

    let registry_arc = Arc::new(registry);
    let executor = SkillExecutor::new(registry_arc);

    // Get help for commit skill
    let help = executor.get_help("commit");
    assert!(help.is_ok());
    let help_text = help.unwrap();
    assert!(help_text.contains("Skill: commit"));
    assert!(help_text.contains("Examples:"));
}

#[test]
fn test_skill_parameter_parsing() {
    let mut registry = SkillRegistry::new();

    // Register built-in skills
    for (skill, categories) in BuiltinSkills::all() {
        registry.register(Arc::new(skill), categories);
    }

    let registry_arc = Arc::new(registry);
    let executor = SkillExecutor::new(registry_arc);

    // Parse input with flags
    let params = executor.parse_input("--message=\"test message\" --verbose");
    assert_eq!(params.named_params.get("message"), Some(&"test message".to_string()));
    assert!(params.flags.contains_key("verbose"));

    // Parse input with positional args
    let params2 = executor.parse_input("file1.rs file2.rs");
    assert_eq!(params2.args.len(), 2);
    assert_eq!(params2.args[0], "file1.rs");
    assert_eq!(params2.args[1], "file2.rs");
}