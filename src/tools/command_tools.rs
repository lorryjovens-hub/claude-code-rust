//! 命令执行工具
//! 
//! 实现 Shell 命令执行功能

use crate::error::Result;
use async_trait::async_trait;
use super::base::{Tool, ToolBuilder};
use super::types::{
    ToolMetadata, ToolUseContext, ToolResult, ToolInputSchema,
    ToolCategory, ToolPermissionLevel,
};

/// Bash 命令执行工具
pub struct BashTool;

#[async_trait]
impl Tool for BashTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("Bash", "Execute a bash command")
            .category(ToolCategory::CommandExecution)
            .permission_level(ToolPermissionLevel::Dangerous)
            .aliases(vec!["bash".to_string(), "sh".to_string()])
            .destructive()
            .input_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::Map::from_iter([
                    ("command".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "The command to execute"
                    })),
                    ("timeout".to_string(), serde_json::json!({
                        "type": "integer",
                        "description": "Timeout in seconds (default: 120)"
                    })),
                ])),
                required: Some(vec!["command".to_string()]),
            })
            .build_metadata()
    }
    
    async fn execute(
        &self,
        input: serde_json::Value,
        context: ToolUseContext,
    ) -> Result<ToolResult> {
        let command = input["command"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("command is required".to_string()))?;
        
        let timeout = input["timeout"].as_u64().unwrap_or(120);
        
        // 执行命令
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(timeout),
            tokio::process::Command::new("bash")
                .arg("-c")
                .arg(command)
                .current_dir(&context.cwd)
                .output(),
        ).await
            .map_err(|_| crate::error::ClaudeError::Tool("Command timed out".to_string()))?
            .map_err(|e| crate::error::ClaudeError::Tool(format!("Failed to execute command: {}", e)))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        
        Ok(ToolResult::success(serde_json::json!({
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": exit_code,
            "success": output.status.success(),
        })))
    }
    
    fn get_activity_description(&self, input: &serde_json::Value) -> Option<String> {
        input["command"].as_str().map(|c| format!("Running: {}", c))
    }
}

/// PowerShell 命令执行工具
pub struct PowerShellTool;

#[async_trait]
impl Tool for PowerShellTool {
    fn metadata(&self) -> ToolMetadata {
        ToolBuilder::new("PowerShell", "Execute a PowerShell command")
            .category(ToolCategory::CommandExecution)
            .permission_level(ToolPermissionLevel::Dangerous)
            .aliases(vec!["pwsh".to_string(), "ps".to_string()])
            .destructive()
            .input_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(serde_json::Map::from_iter([
                    ("command".to_string(), serde_json::json!({
                        "type": "string",
                        "description": "The PowerShell command to execute"
                    })),
                    ("timeout".to_string(), serde_json::json!({
                        "type": "integer",
                        "description": "Timeout in seconds (default: 120)"
                    })),
                ])),
                required: Some(vec!["command".to_string()]),
            })
            .build_metadata()
    }
    
    async fn execute(
        &self,
        input: serde_json::Value,
        context: ToolUseContext,
    ) -> Result<ToolResult> {
        let command = input["command"].as_str()
            .ok_or_else(|| crate::error::ClaudeError::Tool("command is required".to_string()))?;
        
        let timeout = input["timeout"].as_u64().unwrap_or(120);
        
        // 执行 PowerShell 命令
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(timeout),
            tokio::process::Command::new("powershell")
                .arg("-Command")
                .arg(command)
                .current_dir(&context.cwd)
                .output(),
        ).await
            .map_err(|_| crate::error::ClaudeError::Tool("Command timed out".to_string()))?
            .map_err(|e| crate::error::ClaudeError::Tool(format!("Failed to execute command: {}", e)))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        
        Ok(ToolResult::success(serde_json::json!({
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": exit_code,
            "success": output.status.success(),
        })))
    }
    
    fn get_activity_description(&self, input: &serde_json::Value) -> Option<String> {
        input["command"].as_str().map(|c| format!("Running PowerShell: {}", c))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bash_metadata() {
        let tool = BashTool;
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "Bash");
        assert_eq!(metadata.category, ToolCategory::CommandExecution);
        assert_eq!(metadata.permission_level, ToolPermissionLevel::Dangerous);
        assert!(metadata.is_destructive);
    }
    
    #[test]
    fn test_powershell_metadata() {
        let tool = PowerShellTool;
        let metadata = tool.metadata();
        
        assert_eq!(metadata.name, "PowerShell");
        assert_eq!(metadata.category, ToolCategory::CommandExecution);
        assert_eq!(metadata.permission_level, ToolPermissionLevel::Dangerous);
        assert!(metadata.is_destructive);
    }
}
