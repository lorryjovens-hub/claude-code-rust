//! 内置代理
//! 
//! 这个模块实现了内置代理定义

use crate::error::Result;
use super::types::{AgentDefinition, AgentType};

/// 内置代理列表
pub fn get_builtin_agents() -> Vec<AgentDefinition> {
    vec![
        // 通用代理
        AgentDefinition::new(
            "general-purpose".to_string(),
            AgentType::GeneralPurpose,
            "General purpose agent for handling various tasks".to_string(),
        )
        .with_system_prompt(GENERAL_PURPOSE_PROMPT.to_string()),
        
        // 探索代理
        AgentDefinition::new(
            "explore".to_string(),
            AgentType::Explore,
            "Agent for code exploration and analysis".to_string(),
        )
        .with_system_prompt(EXPLORE_PROMPT.to_string()),
        
        // 规划代理
        AgentDefinition::new(
            "plan".to_string(),
            AgentType::Plan,
            "Agent for planning and design tasks".to_string(),
        )
        .with_system_prompt(PLAN_PROMPT.to_string()),
        
        // 验证代理
        AgentDefinition::new(
            "verification".to_string(),
            AgentType::Verification,
            "Agent for verification and testing tasks".to_string(),
        )
        .with_system_prompt(VERIFICATION_PROMPT.to_string()),
        
        // Bash 代理
        AgentDefinition::new(
            "bash".to_string(),
            AgentType::Bash,
            "Agent for command execution tasks".to_string(),
        )
        .with_system_prompt(BASH_PROMPT.to_string()),
    ]
}

/// 初始化内置代理
pub async fn init() -> Result<()> {
    tracing::debug!("Initializing built-in agents");
    
    let agents = get_builtin_agents();
    tracing::info!("Loaded {} built-in agents", agents.len());
    
    Ok(())
}

/// 通用代理系统提示
const GENERAL_PURPOSE_PROMPT: &str = r#"You are a general-purpose AI agent that can handle various tasks.

Your capabilities include:
- Reading and writing files
- Executing shell commands
- Searching and analyzing code
- Web searches and fetches
- Creating and managing tasks

You have access to the full set of tools. Use them wisely to accomplish the user's goals efficiently."#;

/// 探索代理系统提示
const EXPLORE_PROMPT: &str = r#"You are an exploration agent specialized in code analysis and discovery.

Your capabilities include:
- Reading files
- Searching for patterns using glob and grep
- Fetching web content
- Analyzing code structure

You have read-only access to tools. Focus on understanding and documenting code without making changes."#;

/// 规划代理系统提示
const PLAN_PROMPT: &str = r#"You are a planning agent specialized in design and architecture.

Your capabilities include:
- Reading files
- Searching for patterns
- Web searches for best practices
- Analyzing requirements

You have read-only access to tools. Focus on creating detailed plans and designs."#;

/// 验证代理系统提示
const VERIFICATION_PROMPT: &str = r#"You are a verification agent specialized in testing and validation.

Your capabilities include:
- Executing shell commands
- Reading files
- Running tests
- Validating implementations

You have access to testing tools. Focus on verifying that implementations meet requirements."#;

/// Bash 代理系统提示
const BASH_PROMPT: &str = r#"You are a bash agent specialized in command execution.

Your capabilities include:
- Executing shell commands
- Reading and writing files
- Managing file system operations

You have access to bash and file tools. Focus on executing commands efficiently and safely."#;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_builtin_agents() {
        let agents = get_builtin_agents();
        
        assert_eq!(agents.len(), 5);
        
        let names: Vec<&str> = agents.iter().map(|a| a.name.as_str()).collect();
        assert!(names.contains(&"general-purpose"));
        assert!(names.contains(&"explore"));
        assert!(names.contains(&"plan"));
        assert!(names.contains(&"verification"));
        assert!(names.contains(&"bash"));
    }
    
    #[test]
    fn test_agent_tool_sets() {
        let agents = get_builtin_agents();
        
        for agent in agents {
            let tools = agent.agent_type.get_tool_set();
            assert!(!tools.is_empty());
        }
    }
}
