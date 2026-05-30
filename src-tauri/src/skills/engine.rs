use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;

use crate::tools::{get_tool_definitions, execute_tool_async, ToolDefinition};
use crate::mcp::{McpServerManager, McpTool};

#[derive(Clone, Serialize, Deserialize)]
pub struct SkillExecutionContext {
    pub conversation_id: String,
    pub messages: Vec<JsonValue>,
    pub available_tools: Vec<ToolDefinition>,
    pub available_mcp_tools: Vec<McpTool>,
    pub current_input: String,
    pub workspace_path: Option<String>,
    pub variables: HashMap<String, String>,
    #[serde(skip_serializing, skip_deserializing)]
    pub mcp_server_manager: Option<std::sync::Arc<crate::mcp::McpServerManager>>,
}

impl Default for SkillExecutionContext {
    fn default() -> Self {
        Self {
            conversation_id: "".to_string(),
            messages: Vec::new(),
            available_tools: get_tool_definitions(),
            available_mcp_tools: Vec::new(),
            current_input: "".to_string(),
            workspace_path: None,
            variables: HashMap::new(),
            mcp_server_manager: None,
        }
    }
}

impl SkillExecutionContext {
    pub fn with_variable(mut self, key: &str, value: &str) -> Self {
        self.variables.insert(key.to_string(), value.to_string());
        self
    }
    
    pub fn with_mcp_manager(mut self, manager: Arc<McpServerManager>) -> Self {
        self.mcp_server_manager = Some(manager);
        self
    }
    
    pub fn resolve_variables(&self, text: &str) -> String {
        let mut result = text.to_string();
        for (key, value) in &self.variables {
            let placeholder = format!("${{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }
}

pub struct SkillExecutionEngine;

impl SkillExecutionEngine {
    pub async fn execute(
        skill_content: &str,
        context: &SkillExecutionContext,
    ) -> Result<SkillExecutionResult> {
        let frontmatter = Self::parse_frontmatter(skill_content);
        let instructions = Self::extract_instructions(skill_content);
        
        let mut results: Vec<ToolExecutionResult> = Vec::new();
        let mut context_vars = context.variables.clone();
        
        for instruction in instructions {
            let resolved_instruction = context.resolve_variables(&instruction);
            
            match Self::parse_instruction(&resolved_instruction) {
                Some(Instruction::ToolCall(tool_name, args)) => {
                    let tool_result = Self::execute_tool_call(&tool_name, &args, context, &mut context_vars).await?;
                    results.push(tool_result.clone());
                    
                    if let Ok(output_str) = serde_json::to_string(&tool_result.output) {
                        context_vars.insert(format!("tool_{}_output", tool_name), output_str);
                    }
                }
                Some(Instruction::Text(text)) => {
                    let resolved_text = context.resolve_variables(&text);
                    results.push(ToolExecutionResult {
                        tool_name: "text".to_string(),
                        input: JsonValue::Null,
                        output: JsonValue::String(resolved_text),
                        error: None,
                    });
                }
                None => {}
            }
        }
        
        let summary = Self::generate_summary(&frontmatter, &results, context);
        
        Ok(SkillExecutionResult {
            results,
            summary,
            frontmatter,
        })
    }

    fn parse_frontmatter(content: &str) -> FrontmatterData {
        let content_trimmed = content.trim_start();
        if !content_trimmed.starts_with("---") {
            return FrontmatterData::default();
        }

        if let Some(end_pos) = content_trimmed[3..].find("---") {
            let yaml_block = &content_trimmed[3..end_pos + 3];
            return Self::parse_yaml_frontmatter(yaml_block);
        }

        FrontmatterData::default()
    }

    fn parse_yaml_frontmatter(yaml_block: &str) -> FrontmatterData {
        let mut frontmatter = FrontmatterData::default();
        
        for line in yaml_block.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim().trim_matches('"').trim_matches('\'').trim();
                
                match key {
                    "name" => frontmatter.name = Some(value.to_string()),
                    "description" => frontmatter.description = Some(value.to_string()),
                    "when" | "whenToUse" => frontmatter.when_to_use = Some(value.to_string()),
                    "model" => frontmatter.model = Some(value.to_string()),
                    "userInvocable" | "user_invocable" => {
                        frontmatter.user_invokable = Some(value == "true" || value == "yes");
                    }
                    "allowedTools" | "allowed_tools" => {
                        let tools: Vec<String> = value
                            .trim_start_matches('[')
                            .trim_end_matches(']')
                            .split(',')
                            .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        if !tools.is_empty() {
                            frontmatter.allowed_tools = Some(tools);
                        }
                    }
                    _ => {}
                }
            }
        }
        
        frontmatter
    }

    fn extract_instructions(content: &str) -> Vec<String> {
        let content_trimmed = content.trim_start();
        
        let start_pos = if content_trimmed.starts_with("---") {
            content_trimmed[3..].find("---").map(|p| p + 6).unwrap_or(0)
        } else {
            0
        };

        let body = &content[start_pos..];
        
        let mut instructions = Vec::new();
        let mut current_instruction = String::new();
        
        for line in body.lines() {
            if line.starts_with("```") {
                if !current_instruction.is_empty() {
                    instructions.push(current_instruction.trim().to_string());
                    current_instruction.clear();
                }
                let lang = line[3..].trim();
                if lang.starts_with("tool") {
                    continue;
                }
            } else if line.starts_with("- ") || line.starts_with("* ") {
                if !current_instruction.is_empty() {
                    instructions.push(current_instruction.trim().to_string());
                    current_instruction.clear();
                }
                current_instruction.push_str(&line[2..]);
            } else if line.starts_with("## ") || line.starts_with("# ") {
                if !current_instruction.is_empty() {
                    instructions.push(current_instruction.trim().to_string());
                    current_instruction.clear();
                }
            } else {
                current_instruction.push_str(line);
                current_instruction.push('\n');
            }
        }
        
        if !current_instruction.trim().is_empty() {
            instructions.push(current_instruction.trim().to_string());
        }
        
        instructions
    }

    fn parse_instruction(line: &str) -> Option<Instruction> {
        if line.contains("tool_call") || line.contains("调用工具") || line.contains("execute") {
            if let Some((tool_part, args_part)) = line.split_once('(') {
                let tool_name = tool_part.trim().split_whitespace().last()?;
                let args_str = args_part.trim_end_matches(')').trim();
                
                if let Ok(args) = serde_json::from_str(args_str) {
                    return Some(Instruction::ToolCall(tool_name.to_string(), args));
                }
            }
        }
        
        if !line.is_empty() {
            return Some(Instruction::Text(line.to_string()));
        }
        
        None
    }

    async fn execute_tool_call(
        tool_name: &str,
        args: &JsonValue,
        context: &SkillExecutionContext,
        _context_vars: &mut HashMap<String, String>,
    ) -> Result<ToolExecutionResult> {
        let cwd = context.workspace_path.as_deref().unwrap_or(".");
        
        let is_mcp_tool = context.available_mcp_tools.iter()
            .any(|t| t.name == tool_name);
        
        let result = if is_mcp_tool {
            Self::execute_mcp_tool_call(tool_name, args, context).await
        } else {
            execute_tool_async(tool_name, args.clone(), cwd).await
        };
        
        match result {
            Ok(output) => Ok(ToolExecutionResult {
                tool_name: tool_name.to_string(),
                input: args.clone(),
                output,
                error: None,
            }),
            Err(e) => Ok(ToolExecutionResult {
                tool_name: tool_name.to_string(),
                input: args.clone(),
                output: JsonValue::Null,
                error: Some(e.to_string()),
            }),
        }
    }
    
    async fn execute_mcp_tool_call(
        tool_name: &str,
        args: &JsonValue,
        context: &SkillExecutionContext,
    ) -> Result<JsonValue> {
        let manager = context.mcp_server_manager.clone()
            .ok_or_else(|| anyhow!("MCP server manager not available"))?;
        
        let mcp_tool = context.available_mcp_tools.iter()
            .find(|t| t.name == tool_name)
            .ok_or_else(|| anyhow!("MCP tool not found: {}", tool_name))?;
        
        let server_name = &mcp_tool.server_name;
        
        let response = manager.call_tool(server_name, tool_name, args.clone()).await?;
        
        Ok(response)
    }

    fn generate_summary(
        frontmatter: &FrontmatterData,
        results: &[ToolExecutionResult],
        _context: &SkillExecutionContext,
    ) -> String {
        let mut summary = String::new();
        
        if let Some(name) = &frontmatter.name {
            summary.push_str(&format!("## {} 执行结果\n\n", name));
        }
        
        for (_i, result) in results.iter().enumerate() {
            if result.tool_name == "text" {
                if let Some(text) = result.output.as_str() {
                    summary.push_str(text);
                    summary.push('\n');
                }
            } else {
                summary.push_str(&format!("### 工具调用: {}\n\n", result.tool_name));
                summary.push_str(&format!("**输入:**\n```json\n{}\n```\n\n", result.input));
                
                if let Some(error) = &result.error {
                    summary.push_str(&format!("**错误:** {}\n\n", error));
                } else {
                    summary.push_str(&format!("**输出:**\n```json\n{}\n```\n\n", result.output));
                }
            }
        }
        
        summary
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillExecutionResult {
    pub results: Vec<ToolExecutionResult>,
    pub summary: String,
    pub frontmatter: FrontmatterData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    pub tool_name: String,
    pub input: JsonValue,
    pub output: JsonValue,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrontmatterData {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "whenToUse")]
    pub when_to_use: Option<String>,
    #[serde(rename = "allowedTools")]
    pub allowed_tools: Option<Vec<String>>,
    pub model: Option<String>,
    #[serde(rename = "userInvocable")]
    pub user_invokable: Option<bool>,
}

#[derive(Debug, Clone)]
enum Instruction {
    ToolCall(String, JsonValue),
    Text(String),
}