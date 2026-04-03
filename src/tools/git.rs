//! Git tool

use crate::error::Result;
use crate::tools::{Tool, ToolInputSchema, ToolResult, ToolUseContext};
use async_trait::async_trait;
use std::collections::HashMap;

/// Git tool
#[derive(Debug, Clone, Copy)]
pub struct GitTool;

impl GitTool {
    /// Create a new git tool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GitTool {
    fn name(&self) -> &str {
        "git"
    }
    
    fn description(&self) -> &str {
        "Execute git commands"
    }
    
    fn input_schema(&self) -> ToolInputSchema {
        let mut properties = HashMap::new();
        properties.insert(
            "command".to_string(),
            serde_json::json!({
                "type": "string",
                "description": "Git command to execute (without the 'git' prefix)",
            }),
        );
        
        ToolInputSchema {
            r#type: "object".to_string(),
            properties,
            required: vec!["command".to_string()],
        }
    }
    
    async fn execute(&self, input: serde_json::Value, context: ToolUseContext) -> Result<ToolResult> {
        let command = input["command"]
            .as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("Command is required".to_string()))?;
        
        let mut args: Vec<&str> = command.split_whitespace().collect();
        let git_cmd = args.remove(0);
        
        let output = tokio::process::Command::new("git")
            .arg(git_cmd)
            .args(&args)
            .current_dir(&context.cwd)
            .output()
            .await?;
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        let result = if output.status.success() {
            ToolResult::success(stdout)
        } else {
            ToolResult::error(format!("{}\n{}", stdout, stderr))
        };
        
        Ok(result)
    }
}
