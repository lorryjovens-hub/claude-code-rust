//! Search Tool

use super::{Tool, ToolOutput, ToolError};
use async_trait::async_trait;
use serde_json;
use std::path::Path;

pub struct SearchTool;

impl Default for SearchTool {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SearchTool {
    fn name(&self) -> &str {
        "search"
    }
    
    fn description(&self) -> &str {
        "Search for patterns in files using regex"
    }
    
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path to search in"
                },
                "pattern": {
                    "type": "string",
                    "description": "Regex pattern to search for"
                },
                "file_pattern": {
                    "type": "string",
                    "description": "File pattern to match (optional)"
                }
            },
            "required": ["path", "pattern"]
        })
    }
    
    async fn execute(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let path = input["path"].as_str()
            .ok_or_else(|| ToolError {
                message: "path is required".to_string(),
                code: Some("missing_parameter".to_string()),
            })?;
        
        let pattern = input["pattern"].as_str()
            .ok_or_else(|| ToolError {
                message: "pattern is required".to_string(),
                code: Some("missing_parameter".to_string()),
            })?;
        
        let search_path = Path::new(path);
        
        if !search_path.exists() {
            return Err(ToolError {
                message: format!("Path does not exist: {}", path),
                code: Some("path_not_found".to_string()),
            });
        }
        
        // Simple grep-like search
        let regex = regex::Regex::new(pattern)
            .map_err(|e| ToolError {
                message: format!("Invalid regex pattern: {}", e),
                code: Some("invalid_pattern".to_string()),
            })?;
        
        let mut results = Vec::new();
        
        // Walk the directory
        for entry in walkdir::WalkDir::new(search_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();
            
            if entry_path.is_file() {
                // Try to read and search
                if let Ok(content) = std::fs::read_to_string(entry_path) {
                    for (line_num, line) in content.lines().enumerate() {
                        if regex.is_match(line) {
                            results.push(format!("{}:{}: {}", entry_path.display(), line_num + 1, line));
                        }
                    }
                }
            }
        }
        
        Ok(ToolOutput {
            output_type: "text".to_string(),
            content: results.join("\n"),
            metadata: std::collections::HashMap::new(),
        })
    }
}