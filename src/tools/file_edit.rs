//! File Edit Tool

use super::{Tool, ToolOutput, ToolError};
use async_trait::async_trait;
use serde_json;
use std::path::Path;

pub struct FileEditTool;

impl Default for FileEditTool {
    fn default() -> Self {
        Self::new()
    }
}

impl FileEditTool {
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
        "Edit a file by replacing specific content"
    }
    
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file to edit"
                },
                "old_content": {
                    "type": "string",
                    "description": "Content to find and replace"
                },
                "new_content": {
                    "type": "string",
                    "description": "New content to replace with"
                }
            },
            "required": ["file_path", "old_content", "new_content"]
        })
    }
    
    async fn execute(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let file_path = input["file_path"].as_str()
            .ok_or_else(|| ToolError {
                message: "file_path is required".to_string(),
                code: Some("missing_parameter".to_string()),
            })?;
        
        let old_content = input["old_content"].as_str()
            .ok_or_else(|| ToolError {
                message: "old_content is required".to_string(),
                code: Some("missing_parameter".to_string()),
            })?;
        
        let new_content = input["new_content"].as_str()
            .ok_or_else(|| ToolError {
                message: "new_content is required".to_string(),
                code: Some("missing_parameter".to_string()),
            })?;
        
        let path = Path::new(file_path);
        
        if !path.exists() {
            return Err(ToolError {
                message: format!("File does not exist: {}", file_path),
                code: Some("file_not_found".to_string()),
            });
        }
        
        let content = std::fs::read_to_string(path)
            .map_err(|e| ToolError {
                message: format!("Failed to read file: {}", e),
                code: Some("read_error".to_string()),
            })?;
        
        if !content.contains(old_content) {
            return Err(ToolError {
                message: "old_content not found in file".to_string(),
                code: Some("content_not_found".to_string()),
            });
        }
        
        let new_file_content = content.replace(old_content, new_content);
        
        std::fs::write(path, new_file_content)
            .map_err(|e| ToolError {
                message: format!("Failed to write file: {}", e),
                code: Some("write_error".to_string()),
            })?;
        
        Ok(ToolOutput {
            output_type: "text".to_string(),
            content: format!("Successfully edited {}", file_path),
            metadata: std::collections::HashMap::new(),
        })
    }
}