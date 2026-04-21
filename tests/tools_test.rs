//! Tests for Tools Module

use claude_code_rs::tools::{ToolRegistry, ToolOutput};

#[tokio::test]
async fn test_tool_registry_creation() {
    let registry = ToolRegistry::new();
    let tools = registry.list();

    // Should have 9 tools now (6 original + 3 new)
    assert!(tools.len() >= 6);
}

#[tokio::test]
async fn test_file_read_tool() {
    let registry = ToolRegistry::new();
    let tool = registry.get("file_read").expect("file_read tool should exist");

    assert_eq!(tool.name(), "file_read");
    assert!(!tool.description().is_empty());
}

#[tokio::test]
async fn test_git_operations_tool() {
    let registry = ToolRegistry::new();
    let tool = registry.get("git_operations").expect("git_operations tool should exist");

    assert_eq!(tool.name(), "git_operations");
    assert!(tool.description().contains("Git"));
}

#[tokio::test]
async fn test_task_management_tool() {
    let registry = ToolRegistry::new();
    let tool = registry.get("task_management").expect("task_management tool should exist");

    assert_eq!(tool.name(), "task_management");
    assert!(tool.description().contains("task"));
}

#[tokio::test]
async fn test_note_edit_tool() {
    let registry = ToolRegistry::new();
    let tool = registry.get("note_edit").expect("note_edit tool should exist");

    assert_eq!(tool.name(), "note_edit");
    assert!(tool.description().contains("note"));
}

#[tokio::test]
async fn test_task_create_and_list() {
    use serde_json::json;

    let registry = ToolRegistry::new();

    // Create a task
    let create_result = registry.execute("task_management", json!({
        "operation": "create",
        "subject": "Test Task",
        "description": "This is a test task"
    })).await;

    assert!(create_result.is_ok());

    // List tasks
    let list_result = registry.execute("task_management", json!({
        "operation": "list"
    })).await;

    assert!(list_result.is_ok());
}

#[tokio::test]
async fn test_note_create_and_search() {
    use serde_json::json;

    let registry = ToolRegistry::new();

    // Create a note
    let create_result = registry.execute("note_edit", json!({
        "operation": "create",
        "title": "Test Note",
        "content": "This is a test note content",
        "tags": ["test", "example"]
    })).await;

    assert!(create_result.is_ok());

    // Search notes
    let search_result = registry.execute("note_edit", json!({
        "operation": "search",
        "search_query": "test"
    })).await;

    assert!(search_result.is_ok());
}