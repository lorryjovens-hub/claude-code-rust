//! List Files Tool

use super::{Tool, ToolOutput, ToolError};
use async_trait::async_trait;
use serde_json;
use std::path::Path;

pub struct ListFilesTool;

impl Default for ListFilesTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ListFilesTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ListFilesTool {
    fn name(&self) -> &str {
        "list_files"
    }
    
    fn description(&self) -> &str {
        "List files in a directory"
    }
    
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path to list"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "List recursively (optional)"
                }
            },
            "required": ["path"]
        })
    }
    
    async fn execute(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ToolError {
                message: "path is required".to_string(),
                code: Some("missing_parameter".to_string()),
            })?;
        
        let recursive = input["recursive"].as_bool().unwrap_or(false);
        
        let list_path = Path::new(path);
        
        if !list_path.exists() {
            return Err(ToolError {
                message: format!("Path does not exist: {}", path),
                code: Some("path_not_found".to_string()),
            });
        }
        
        if !list_path.is_dir() {
            return Err(ToolError {
                message: format!("Path is not a directory: {}", path),
                code: Some("not_directory".to_string()),
            });
        }
        
        let mut results = Vec::new();
        
        if recursive {
            for entry in walkdir::WalkDir::new(list_path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let entry_path = entry.path();
                let file_type = if entry_path.is_dir() { "DIR" } else { "FILE" };
                results.push(format!("{} [{}]", entry_path.display(), file_type));
            }
        } else {
            for entry in std::fs::read_dir(list_path)
                .map_err(|e| ToolError {
                    message: format!("Failed to read directory: {}", e),
                    code: Some("read_error".to_string()),
                })?
            {
                let entry = entry.map_err(|e| ToolError {
                    message: format!("Failed to read entry: {}", e),
                    code: Some("read_error".to_string()),
                })?;
                
                let entry_path = entry.path();
                let file_type = if entry_path.is_dir() { "DIR" } else { "FILE" };
                results.push(format!("{} [{}]", entry_path.display(), file_type));
            }
        }
        
        Ok(ToolOutput {
            output_type: "text".to_string(),
            content: results.join("\n"),
            metadata: std::collections::HashMap::new(),
        })
    }
}