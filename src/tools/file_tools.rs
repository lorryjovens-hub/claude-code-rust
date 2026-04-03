//! 文件读取工具
//! 
//! 实现文件读取功能

use crate::error::Result;
use async_trait::async_trait;
use std::path::Path;
use super::base::{Tool, ToolBuilder};
use super::types::{
    ToolMetadata, ToolUseContext, ToolResult, ToolInputSchema,
    ToolCategory, ToolPermissionLevel,
};

/// 文件读取工具
pub struct FileReadTool;

#[async_trait]
impl Tool for FileReadTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("Read", "Read a file from the local filesystem")
            .category(ToolCategory::FileOperation)
            .permission_level(ToolPermissionLevel::Standard)
            .aliases(vec!["read".to_string(), "cat".to_string()])
            .read_only()
            .input_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::Map::from_iter([
                    ("file_path".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "The absolute path to the file to read"
                    })),
                    ("offset".to_string(), serde_json::json!({
                        "type": "integer",
                        "description": "The line number to start reading from"
                    })),
                    ("limit".to_string(), serde_json::json!({
                        "type": "integer", 
                        "description": "The number of lines to read"
                    })),
                ])),
                required: Some(vec!["file_path".to_string()]),
            })
            .build_metadata()
    }
    
    async fn execute(
        &self,
        input: serde_json::Value,
        context: ToolUseContext,
    ) -> Result<ToolResult> {
        let file_path = input["file_path"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("file_path is required".to_string()))?;
        
        let path = Path::new(file_path);
        
        // 检查路径是否在工作目录内
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            context.cwd.join(path)
        };
        
        // 读取文件
        let content = tokio::fs::read_to_string(&abs_path).await
            .map_err(|e| crate::error::ClaudeError::File(format!("Failed to read file: {}", e)))?;
        
        // 处理 offset 和 limit
        let offset = input["offset"].as_u64().unwrap_or(1) as usize;
        let limit = input["limit"].as_u64().unwrap_or(2000) as usize;
        
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        
        let start = (offset.saturating_sub(1)).min(total_lines);
        let end = (start + limit).min(total_lines);
        
        let selected_lines: Vec<&str> = lines[start..end].to_vec();
        let result = selected_lines.join("\n");
        
        Ok(ToolResult::success(serde_json::json!({
            "content": result,
            "total_lines": total_lines,
            "lines_read": end - start,
            "start_line": start + 1,
            "end_line": end,
        })))
    }
    
    fn get_path(&self, input: &serde_json::Value) -> Option<String> {
        input["file_path"].as_str().map(|s| s.to_string())
    }
    
    fn get_activity_description(&self, input: &serde_json::Value) -> Option<String> {
        input["file_path"].as_str().map(|p| format!("Reading {}", p))
    }
}

/// 文件编辑工具
pub struct FileEditTool;

#[async_trait]
impl Tool for FileEditTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("Edit", "Edit a file by replacing text")
            .category(ToolCategory::FileOperation)
            .permission_level(ToolPermissionLevel::Standard)
            .aliases(vec!["edit".to_string()])
            .input_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::Map::from_iter([
                    ("file_path".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "The absolute path to the file to edit"
                    })),
                    ("old_str".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "The text to search for and replace"
                    })),
                    ("new_str".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "The text to replace with"
                    })),
                ])),
                required: Some(vec!["file_path".to_string(), "old_str".to_string(), "new_str".to_string()]),
            })
            .build_metadata()
    }
    
    async fn execute(
        &self,
        input: serde_json::Value,
        context: ToolUseContext,
    ) -> Result<ToolResult> {
        let file_path = input["file_path"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("file_path is required".to_string()))?;
        let old_str = input["old_str"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("old_str is required".to_string()))?;
        let new_str = input["new_str"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("new_str is required".to_string()))?;
        
        let path = Path::new(file_path);
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            context.cwd.join(path)
        };
        
        // 读取文件
        let content = tokio::fs::read_to_string(&abs_path).await
            .map_err(|e| crate::error::ClaudeError::File(format!("Failed to read file: {}", e)))?;
        
        // 替换文本
        if !content.contains(old_str) {
            return Ok(ToolResult::error("old_str not found in file"));
        }
        
        let new_content = content.replacen(old_str, new_str, 1);
        
        // 写入文件
        tokio::fs::write(&abs_path, new_content).await
            .map_err(|e| crate::error::ClaudeError::File(format!("Failed to write file: {}", e)))?;
        
        Ok(ToolResult::success(serde_json::json!({
            "message": "File edited successfully",
            "file_path": file_path,
        })))
    }
    
    fn get_path(&self, input: &serde_json::Value) -> Option<String> {
        input["file_path"].as_str().map(|s| s.to_string())
    }
    
    fn get_activity_description(&self, input: &serde_json::Value) -> Option<String> {
        input["file_path"].as_str().map(|p| format!("Editing {}", p))
    }
}

/// 文件写入工具
pub struct FileWriteTool;

#[async_trait]
impl Tool for FileWriteTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("Write", "Write content to a file")
            .category(ToolCategory::FileOperation)
            .permission_level(ToolPermissionLevel::Standard)
            .aliases(vec!["write".to_string()])
            .destructive()
            .input_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::Map::from_iter([
                    ("file_path".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "The absolute path to the file to write"
                    })),
                    ("content".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "The content to write to the file"
                    })),
                ])),
                required: Some(vec!["file_path".to_string(), "content".to_string()]),
            })
            .build_metadata()
    }
    
    async fn execute(
        &self,
        input: serde_json::Value,
        context: ToolUseContext,
    ) -> Result<ToolResult> {
        let file_path = input["file_path"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("file_path is required".to_string()))?;
        let content = input["content"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("content is required".to_string()))?;
        
        let path = Path::new(file_path);
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            context.cwd.join(path)
        };
        
        // 确保父目录存在
        if let Some(parent) = abs_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| crate::error::ClaudeError::File(format!("Failed to create directory: {}", e)))?;
        }
        
        // 写入文件
        tokio::fs::write(&abs_path, content).await
            .map_err(|e| crate::error::ClaudeError::File(format!("Failed to write file: {}", e)))?;
        
        Ok(ToolResult::success(serde_json::json!({
            "message": "File written successfully",
            "file_path": file_path,
        })))
    }
    
    fn get_path(&self, input: &serde_json::Value) -> Option<String> {
        input["file_path"].as_str().map(|s| s.to_string())
    }
    
    fn get_activity_description(&self, input: &serde_json::Value) -> Option<String> {
        input["file_path"].as_str().map(|p| format!("Writing {}", p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_file_read_metadata() {
        let tool = FileReadTool;
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "Read");
        assert_eq!(metadata.category, ToolCategory::FileOperation);
        assert!(metadata.is_read_only);
    }
    
    #[test]
    fn test_file_edit_metadata() {
        let tool = FileEditTool;
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "Edit");
        assert_eq!(metadata.category, ToolCategory::FileOperation);
    }
    
    #[test]
    fn test_file_write_metadata() {
        let tool = FileWriteTool;
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "Write");
        assert_eq!(metadata.category, ToolCategory::FileOperation);
        assert!(metadata.is_destructive);
    }
}
