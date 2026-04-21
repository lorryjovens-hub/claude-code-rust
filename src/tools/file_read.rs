//! File Read Tool

use super::{Tool, ToolOutput, ToolError};
use async_trait::async_trait;
use serde_json;
use std::path::Path;

pub struct FileReadTool;

impl Default for FileReadTool {
    fn default() -> Self {
        Self::new()
    }
}

impl FileReadTool {
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
    
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file to read"
                }
            },
            "required": ["file_path"]
        })
    }
    
    async fn execute(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let file_path = input["file_path"].as_str()
            .ok_or_else(|| ToolError {
                message: "file_path is required".to_string(),
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
        
        Ok(ToolOutput {
            output_type: "text".to_string(),
            content,
            metadata: std::collections::HashMap::new(),
        })
    }
}