//! File-related tools

use crate::error::Result;
use crate::tools::{Tool, ToolInputSchema, ToolResult, ToolUseContext};
use async_trait::async_trait;
use std::collections::HashMap;

/// File read tool
#[derive(Debug, Clone, Copy)]
pub struct FileReadTool;

impl FileReadTool {
    /// Create a new file read tool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for FileReadTool {
    fn name(&self) -> &str {
        "file_read"
    }
    
    fn description(&self) -> &str {
        "Read the contents of a file"
    }
    
    fn input_schema(&self) -> ToolInputSchema {
        let mut properties = HashMap::new();
        properties.insert(
            "path".to_string(),
            serde_json::json!({
                "type": "string",
                "description": "Path to the file to read",
            }),
        );
        
        ToolInputSchema {
            r#type: "object".to_string(),
            properties,
            required: vec!["path".to_string()],
        }
    }
    
    async fn execute(&self, input: serde_json::Value, context: ToolUseContext) -> Result<ToolResult> {
        let path = input["path"]
            .as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("Path is required".to_string()))?;
        
        let file_path = context.cwd.join(path);
        
        if !file_path.exists() {
            return Ok(ToolResult::error(format!("File not found: {}", path)));
        }
        
        let content = tokio::fs::read_to_string(&file_path).await?;
        
        Ok(ToolResult::success(content))
    }
}

/// File edit tool
#[derive(Debug, Clone, Copy)]
pub struct FileEditTool;

impl FileEditTool {
    /// Create a new file edit tool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for FileEditTool {
    fn name(&self) -> &str {
        "file_edit"
    }
    
    fn description(&self) -> &str {
        "Edit the contents of a file"
    }
    
    fn input_schema(&self) -> ToolInputSchema {
        let mut properties = HashMap::new();
        properties.insert(
            "path".to_string(),
            serde_json::json!({
                "type": "string",
                "description": "Path to the file to edit",
            }),
        );
        properties.insert(
            "old_str".to_string(),
            serde_json::json!({
                "type": "string",
                "description": "Old string to replace",
            }),
        );
        properties.insert(
            "new_str".to_string(),
            serde_json::json!({
                "type": "string",
                "description": "New string to replace with",
            }),
        );
        
        ToolInputSchema {
            r#type: "object".to_string(),
            properties,
            required: vec!["path".to_string(), "old_str".to_string(), "new_str".to_string()],
        }
    }
    
    async fn execute(&self, input: serde_json::Value, context: ToolUseContext) -> Result<ToolResult> {
        let path = input["path"]
            .as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("Path is required".to_string()))?;
        let old_str = input["old_str"]
            .as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("Old string is required".to_string()))?;
        let new_str = input["new_str"]
            .as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("New string is required".to_string()))?;
        
        let file_path = context.cwd.join(path);
        
        if !file_path.exists() {
            return Ok(ToolResult::error(format!("File not found: {}", path)));
        }
        
        let content = tokio::fs::read_to_string(&file_path).await?;
        
        if !content.contains(old_str) {
            return Ok(ToolResult::error("Old string not found in file".to_string()));
        }
        
        let new_content = content.replace(old_str, new_str);
        
        tokio::fs::write(&file_path, new_content).await?;
        
        Ok(ToolResult::success("File edited successfully".to_string()))
    }
}

/// File write tool
#[derive(Debug, Clone, Copy)]
pub struct FileWriteTool;

impl FileWriteTool {
    /// Create a new file write tool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for FileWriteTool {
    fn name(&self) -> &str {
        "file_write"
    }
    
    fn description(&self) -> &str {
        "Write content to a file"
    }
    
    fn input_schema(&self) -> ToolInputSchema {
        let mut properties = HashMap::new();
        properties.insert(
            "path".to_string(),
            serde_json::json!({
                "type": "string",
                "description": "Path to the file to write",
            }),
        );
        properties.insert(
            "content".to_string(),
            serde_json::json!({
                "type": "string",
                "description": "Content to write to the file",
            }),
        );
        
        ToolInputSchema {
            r#type: "object".to_string(),
            properties,
            required: vec!["path".to_string(), "content".to_string()],
        }
    }
    
    async fn execute(&self, input: serde_json::Value, context: ToolUseContext) -> Result<ToolResult> {
        let path = input["path"]
            .as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("Path is required".to_string()))?;
        let content = input["content"]
            .as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("Content is required".to_string()))?;
        
        let file_path = context.cwd.join(path);
        
        // Create parent directories if needed
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        tokio::fs::write(&file_path, content).await?;
        
        Ok(ToolResult::success("File written successfully".to_string()))
    }
}
