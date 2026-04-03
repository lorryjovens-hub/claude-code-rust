//! 代码搜索工具
//! 
//! 实现文件搜索和内容搜索功能

use crate::error::Result;
use async_trait::async_trait;
use super::base::{Tool, ToolBuilder};
use super::types::{
    ToolMetadata, ToolUseContext, ToolResult, ToolInputSchema,
    ToolCategory, ToolPermissionLevel,
};

/// Glob 文件模式匹配工具
pub struct GlobTool;

#[async_trait]
impl Tool for GlobTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("Glob", "Find files matching a glob pattern")
            .category(ToolCategory::CodeSearch)
            .permission_level(ToolPermissionLevel::ReadOnly)
            .aliases(vec!["glob".to_string()])
            .read_only()
            .input_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::Map::from_iter([
                    ("pattern".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "The glob pattern to match files against"
                    })),
                    ("path".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "The directory to search in (defaults to cwd)"
                    })),
                ])),
                required: Some(vec!["pattern".to_string()]),
            })
            .build_metadata()
    }
    
    async fn execute(
        &self,
        input: serde_json::Value,
        context: ToolUseContext,
    ) -> Result<ToolResult> {
        let pattern = input["pattern"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("pattern is required".to_string()))?;
        
        let search_path = input["path"].as_str()
            .map(|p| std::path::PathBuf::from(p))
            .unwrap_or_else(|| context.cwd.clone());
        
        // 使用 glob 模式匹配
        let mut matches = Vec::new();
        
        // 简单的 glob 实现（实际应该使用 glob crate）
        if let Ok(entries) = std::fs::read_dir(&search_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name() {
                    if let Some(file_name_str) = file_name.to_str() {
                        if glob_match(pattern, file_name_str) {
                            matches.push(path.display().to_string());
                        }
                    }
                }
            }
        }
        
        Ok(ToolResult::success(serde_json::json!({
            "matches": matches,
            "count": matches.len(),
        })))
    }
    
    fn get_activity_description(&self, input: &serde_json::Value) -> Option<String> {
        input["pattern"].as_str().map(|p| format!("Searching for {}", p))
    }
}

/// Grep 内容搜索工具
pub struct GrepTool;

#[async_trait]
impl Tool for GrepTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("Grep", "Search for patterns in file contents")
            .category(ToolCategory::CodeSearch)
            .permission_level(ToolPermissionLevel::ReadOnly)
            .aliases(vec!["grep".to_string()])
            .read_only()
            .input_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::Map::from_iter([
                    ("pattern".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "The pattern to search for"
                    })),
                    ("path".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "The file or directory to search in"
                    })),
                    ("output_mode".to_string(), serde_json::json!({
                        "type": "string",
                        "enum": ["content", "files_with_matches", "count"],
                        "description": "The output mode"
                    })),
                ])),
                required: Some(vec!["pattern".to_string()]),
            })
            .build_metadata()
    }
    
    async fn execute(
        &self,
        input: serde_json::Value,
        context: ToolUseContext,
    ) -> Result<ToolResult> {
        let pattern = input["pattern"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("pattern is required".to_string()))?;
        
        let search_path = input["path"].as_str()
            .map(|p| std::path::PathBuf::from(p))
            .unwrap_or_else(|| context.cwd.clone());
        
        let output_mode = input["output_mode"].as_str().unwrap_or("content");
        
        let mut matches = Vec::new();
        let mut file_count = 0;
        
        // 简单的搜索实现
        if search_path.is_file() {
            if let Ok(content) = tokio::fs::read_to_string(&search_path).await {
                for (line_num, line) in content.lines().enumerate() {
                    if line.contains(pattern) {
                        match output_mode {
                            "content" => {
                                matches.push(serde_json::json!({
                                    "file": search_path.display().to_string(),
                                    "line": line_num + 1,
                                    "content": line,
                                }));
                            }
                            "files_with_matches" => {
                                matches.push(serde_json::json!({
                                    "file": search_path.display().to_string(),
                                }));
                                break;
                            }
                            "count" => {
                                file_count += 1;
                            }
                            _ => {}
                        }
                    }
                }
            }
        } else if search_path.is_dir() {
            // 搜索目录中的所有文件
            if let Ok(entries) = std::fs::read_dir(&search_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Ok(content) = tokio::fs::read_to_string(&path).await {
                            let mut found_in_file = false;
                            for (line_num, line) in content.lines().enumerate() {
                                if line.contains(pattern) {
                                    match output_mode {
                                        "content" => {
                                            matches.push(serde_json::json!({
                                                "file": path.display().to_string(),
                                                "line": line_num + 1,
                                                "content": line,
                                            }));
                                        }
                                        "files_with_matches" => {
                                            if !found_in_file {
                                                matches.push(serde_json::json!({
                                                    "file": path.display().to_string(),
                                                }));
                                                found_in_file = true;
                                            }
                                        }
                                        "count" => {
                                            file_count += 1;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(ToolResult::success(serde_json::json!({
            "matches": matches,
            "count": if output_mode == "count" { file_count } else { matches.len() },
        })))
    }
    
    fn get_activity_description(&self, input: &serde_json::Value) -> Option<String> {
        input["pattern"].as_str().map(|p| format!("Searching for pattern: {}", p))
    }
}

/// 简单的 glob 模式匹配
fn glob_match(pattern: &str, text: &str) -> bool {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();
    
    glob_match_helper(&pattern_chars, &text_chars)
}

fn glob_match_helper(pattern: &[char], text: &[char]) -> bool {
    match (pattern.first(), text.first()) {
        (None, None) => true,
        (None, Some(_)) => false,
        (Some('*'), _) => {
            // 匹配零个或多个字符
            glob_match_helper(&pattern[1..], text) || 
            (!text.is_empty() && glob_match_helper(pattern, &text[1..]))
        }
        (Some('?'), Some(_)) => {
            // 匹配任意单个字符
            glob_match_helper(&pattern[1..], &text[1..])
        }
        (Some(p), Some(t)) if *p == *t => {
            glob_match_helper(&pattern[1..], &text[1..])
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_glob_metadata() {
        let tool = GlobTool;
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "Glob");
        assert_eq!(metadata.category, ToolCategory::CodeSearch);
        assert!(metadata.is_read_only);
    }
    
    #[test]
    fn test_grep_metadata() {
        let tool = GrepTool;
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "Grep");
        assert_eq!(metadata.category, ToolCategory::CodeSearch);
        assert!(metadata.is_read_only);
    }
    
    #[test]
    fn test_glob_match() {
        assert!(glob_match("*.rs", "main.rs"));
        assert!(glob_match("*.rs", "lib.rs"));
        assert!(!glob_match("*.rs", "main.txt"));
        assert!(glob_match("test*", "test_file.rs"));
        assert!(glob_match("*test", "my_test"));
    }
}
