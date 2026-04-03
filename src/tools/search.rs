//! Search tools (Glob and Grep)

use crate::error::Result;
use crate::tools::{Tool, ToolInputSchema, ToolResult, ToolUseContext};
use async_trait::async_trait;
use std::collections::HashMap;
use walkdir::WalkDir;

/// Glob tool
#[derive(Debug, Clone, Copy)]
pub struct GlobTool;

impl GlobTool {
    /// Create a new glob tool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }
    
    fn description(&self) -> &str {
        "Search for files matching a glob pattern"
    }
    
    fn input_schema(&self) -> ToolInputSchema {
        let mut properties = HashMap::new();
        properties.insert(
            "pattern".to_string(),
            serde_json::json!({
                "type": "string",
                "description": "Glob pattern to match",
            }),
        );
        properties.insert(
            "path".to_string(),
            serde_json::json!({
                "type": "string",
                "description": "Directory to search in (default: current directory)",
            }),
        );
        
        ToolInputSchema {
            r#type: "object".to_string(),
            properties,
            required: vec!["pattern".to_string()],
        }
    }
    
    async fn execute(&self, input: serde_json::Value, context: ToolUseContext) -> Result<ToolResult> {
        let pattern = input["pattern"]
            .as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("Pattern is required".to_string()))?;
        let path = input["path"].as_str().unwrap_or(".");
        
        let search_path = context.cwd.join(path);
        
        let mut results = Vec::new();
        
        for entry in WalkDir::new(&search_path) {
            let entry = entry?;
            let relative_path = entry.path().strip_prefix(&search_path)
                .unwrap_or_else(|_| entry.path())
                .to_string_lossy()
                .to_string();
            
            if glob_match::glob_match(pattern, &relative_path) {
                results.push(relative_path);
            }
        }
        
        Ok(ToolResult::success(results.join("\n")))
    }
}

/// Grep tool
#[derive(Debug, Clone, Copy)]
pub struct GrepTool;

impl GrepTool {
    /// Create a new grep tool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }
    
    fn description(&self) -> &str {
        "Search for text in files"
    }
    
    fn input_schema(&self) -> ToolInputSchema {
        let mut properties = HashMap::new();
        properties.insert(
            "pattern".to_string(),
            serde_json::json!({
                "type": "string",
                "description": "Regular expression pattern to search for",
            }),
        );
        properties.insert(
            "path".to_string(),
            serde_json::json!({
                "type": "string",
                "description": "Directory or file to search in (default: current directory)",
            }),
        );
        properties.insert(
            "case_insensitive".to_string(),
            serde_json::json!({
                "type": "boolean",
                "description": "Case-insensitive search",
                "default": false,
            }),
        );
        
        ToolInputSchema {
            r#type: "object".to_string(),
            properties,
            required: vec!["pattern".to_string()],
        }
    }
    
    async fn execute(&self, input: serde_json::Value, context: ToolUseContext) -> Result<ToolResult> {
        let pattern = input["pattern"]
            .as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("Pattern is required".to_string()))?;
        let path = input["path"].as_str().unwrap_or(".");
        let case_insensitive = input["case_insensitive"].as_bool().unwrap_or(false);
        
        let search_path = context.cwd.join(path);
        let _regex_flags = if case_insensitive { "i" } else { "" };
        let regex = regex::RegexBuilder::new(pattern)
            .case_insensitive(case_insensitive)
            .build()?;
        
        let mut results = Vec::new();
        
        for entry in WalkDir::new(&search_path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let content = tokio::fs::read_to_string(entry.path()).await;
                if let Ok(content) = content {
                    for (line_num, line) in content.lines().enumerate() {
                        if regex.is_match(line) {
                            let relative_path = entry.path().strip_prefix(&search_path)
                                .unwrap_or_else(|_| entry.path())
                                .to_string_lossy()
                                .to_string();
                            results.push(format!("{}:{}: {}", relative_path, line_num + 1, line));
                        }
                    }
                }
            }
        }
        
        Ok(ToolResult::success(results.join("\n")))
    }
}
