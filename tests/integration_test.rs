//! Integration Tests for Claude Code Rust

use claude_code_rs::{
    cli::Cli,
    config::Settings,
    state::AppState,
    tools::ToolRegistry,
    skills::{SkillRegistry, SkillExecutor, SkillContext, BuiltinSkills, SkillCategory},
};
use std::sync::Arc;

#[test]
fn test_cli_initialization() {
    // Test that CLI can be parsed
    let cli = Cli::parse_from(vec!["claude-code"]);
    // Should not panic
}

#[test]
fn test_settings_load() {
    // Settings should load with defaults
    let settings = Settings::load();
    // May fail if no config file exists, but should not panic
}

#[tokio::test]
async fn test_tool_system_integration() {
    let registry = ToolRegistry::new();

    // Test that all expected tools are registered
    let tools = registry.list();
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name()).collect();

    assert!(tool_names.contains(&"file_read"));
    assert!(tool_names.contains(&"file_edit"));
    assert!(tool_names.contains(&"file_write"));
    assert!(tool_names.contains(&"execute_command"));
    assert!(tool_names.contains(&"search"));
    assert!(tool_names.contains(&"list_files"));
    assert!(tool_names.contains(&"git_operations"));
    assert!(tool_names.contains(&"task_management"));
    assert!(tool_names.contains(&"note_edit"));
}

#[tokio::test]
async fn test_skill_system_integration() {
    let mut registry = SkillRegistry::new();

    // Register all built-in skills
    for (skill, categories) in BuiltinSkills::all() {
        registry.register(Arc::new(skill), categories);
    }

    // Verify all categories are represented
    let categories = registry.get_categories();
    assert!(categories.contains(&SkillCategory::Git));
    assert!(categories.contains(&SkillCategory::Utility));

    // Test skill execution
    let registry_arc = Arc::new(registry);
    let executor = SkillExecutor::new(registry_arc);

    let context = SkillContext {
        cwd: std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .to_string(),
        env: std::collections::HashMap::new(),
        tool_registry: None,
        data: std::collections::HashMap::new(),
    };

    // Test each skill
    for skill_name in vec!["commit", "review", "test", "document", "build"] {
        let result = executor.execute(skill_name, "", context.clone()).await;
        assert!(
            result.is_ok(),
            "Skill {} should execute successfully",
            skill_name
        );
    }
}

#[test]
fn test_lib_exports() {
    // Verify all public types are exported
    use claude_code_rs::{
        Skill, SkillRegistry, SkillExecutor, SkillContext, SkillParams,
        SkillResult, SkillError, SkillCategory
    };
    // If this compiles, exports are correct
}

#[tokio::test]
async fn test_new_tools_functionality() {
    use serde_json::json;

    let registry = ToolRegistry::new();

    // Test git status
    let git_result = registry.execute("git_operations", json!({
        "operation": "status"
    })).await;

    // May fail if not in git repo, but should not panic
    let _ = git_result;

    // Test task creation
    let task_result = registry.execute("task_management", json!({
        "operation": "create",
        "subject": "Integration Test Task",
        "description": "Testing task management tool",
        "priority": "high"
    })).await;

    assert!(task_result.is_ok(), "Task creation should succeed");

    // Test note creation
    let note_result = registry.execute("note_edit", json!({
        "operation": "create",
        "title": "Integration Test Note",
        "content": "Testing note edit tool",
        "tags": ["test", "integration"]
    })).await;

    assert!(note_result.is_ok(), "Note creation should succeed");
}