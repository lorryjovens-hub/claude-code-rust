//! Execute Command Tool

use super::{Tool, ToolOutput, ToolError};
use async_trait::async_trait;
use serde_json;

pub struct ExecuteCommandTool;

impl Default for ExecuteCommandTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecuteCommandTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ExecuteCommandTool {
    fn name(&self) -> &str {
        "execute_command"
    }
    
    fn description(&self) -> &str {
        "Execute a shell command"
    }
    
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Command to execute"
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds (optional)"
                }
            },
            "required": ["command"]
        })
    }
    
    async fn execute(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let command = input["command"].as_str()
            .ok_or_else(|| ToolError {
                message: "command is required".to_string(),
                code: Some("missing_parameter".to_string()),
            })?;
        
        let timeout = input["timeout"].as_u64().unwrap_or(60);
        
        // Execute command using tokio::process
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(timeout),
            tokio::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
        ).await;
        
        match output {
            Ok(Ok(result)) => {
                let stdout = String::from_utf8_lossy(&result.stdout).to_string();
                let stderr = String::from_utf8_lossy(&result.stderr).to_string();
                
                let content = if result.status.success() {
                    stdout
                } else {
                    format!("Error: {}\n{}", result.status, stderr)
                };
                
                Ok(ToolOutput {
                    output_type: "text".to_string(),
                    content,
                    metadata: std::collections::HashMap::new(),
                })
            }
            Ok(Err(e)) => Err(ToolError {
                message: format!("Failed to execute command: {}", e),
                code: Some("execution_error".to_string()),
            }),
            Err(_) => Err(ToolError {
                message: "Command timed out".to_string(),
                code: Some("timeout".to_string()),
            }),
        }
    }
}