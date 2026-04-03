//! Bash command tool

use crate::error::Result;
use crate::tools::{Tool, ToolInputSchema, ToolResult, ToolUseContext};
use async_trait::async_trait;
use std::collections::HashMap;

/// Bash tool
#[derive(Debug, Clone, Copy)]
pub struct BashTool;

impl BashTool {
    /// Create a new bash tool
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }
    
    fn description(&self) -> &str {
        "Execute a bash command"
    }
    
    fn input_schema(&self) -> ToolInputSchema {
        let mut properties = HashMap::new();
        properties.insert(
            "command".to_string(),
            serde_json::json!({
                "type": "string",
                "description": "Bash command to execute",
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
        
        let output = if cfg!(target_os = "windows") {
            tokio::process::Command::new("cmd")
                .args(["/C", command])
                .current_dir(&context.cwd)
                .output()
                .await?
        } else {
            tokio::process::Command::new("bash")
                .args(["-c", command])
                .current_dir(&context.cwd)
                .output()
                .await?
        };
        
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
