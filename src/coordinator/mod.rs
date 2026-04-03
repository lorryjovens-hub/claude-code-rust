//! 协调器模式模块
//! 
//! 这个模块实现了协调器模式，对应 TypeScript 的 coordinator/coordinatorMode.ts

use crate::error::Result;
use std::collections::HashSet;

/// 检查是否启用了协调器模式
pub fn is_coordinator_mode() -> bool {
    // 检查特性标志
    if feature_enabled("COORDINATOR_MODE") {
        return is_env_truthy("CLAUDE_CODE_COORDINATOR_MODE");
    }
    false
}

/// 内部工作器工具集
pub const INTERNAL_WORKER_TOOLS: &[&str] = &[
    "TeamCreate",
    "TeamDelete",
    "SendMessage",
    "SyntheticOutput",
];

/// 检查环境变量是否为真
pub fn is_env_truthy(value: &str) -> bool {
    match value {
        "1" | "true" | "yes" | "on" => true,
        _ => false,
    }
}

/// 检查特性标志是否启用（简化版本）
fn feature_enabled(name: &str) -> bool {
    // TODO: 与 GrowthBook 集成
    // 这里应该调用 checkStatsigFeatureGate_CACHED_MAY_BE_STALE
    // 目前简化实现，直接检查环境变量
    std::env::var(format!("CLAUDE_CODE_FEATURE_{}", name.to_uppercase()))
        .map(|v| is_env_truthy(&v))
        .unwrap_or(false)
}

/// 简单的工具名称集合
pub struct SimpleToolSet {
    tools: HashSet<String>,
}

impl SimpleToolSet {
    /// 创建新的工具集
    pub fn new() -> Self {
        Self {
            tools: HashSet::new(),
        }
    }
    
    /// 添加工具
    pub fn insert(&mut self, tool: &str) {
        self.tools.insert(tool.to_string());
    }
    
    /// 检查是否包含工具
    pub fn contains(&self, tool: &str) -> bool {
        self.tools.contains(tool)
    }
    
    /// 获取排序后的工具列表
    pub fn sorted_list(&self) -> Vec<String> {
        let mut tools: Vec<String> = self.tools.iter().cloned().collect();
        tools.sort();
        tools
    }
    
    /// 转换为逗号分隔的字符串
    pub fn to_comma_string(&self) -> String {
        self.sorted_list().join(", ")
    }
}

impl Default for SimpleToolSet {
    fn default() -> Self {
        Self::new()
    }
}

/// 协调器用户上下文
#[derive(Debug, Clone)]
pub struct CoordinatorUserContext {
    /// 工作器工具上下文
    pub worker_tools_context: String,
}

impl CoordinatorUserContext {
    /// 创建协调器用户上下文
    pub fn new(mcp_clients: &[McpClientInfo], scratchpad_dir: Option<&str>) -> Option<Self> {
        if !is_coordinator_mode() {
            return None;
        }
        
        let worker_tools = get_worker_tools();
        let mut content = format!("Workers spawned via the Agent tool have access to these tools: {}", worker_tools);
        
        // 添加 MCP 工具
        if !mcp_clients.is_empty() {
            let server_names: Vec<String> = mcp_clients.iter().map(|c| c.name.clone()).collect();
            content.push_str(&format!("\n\nWorkers also have access to MCP tools from connected MCP servers: {}", server_names.join(", ")));
        }
        
        // 添加 scratchpad 目录
        if let Some(dir) = scratchpad_dir {
            if is_scratchpad_gate_enabled() {
                content.push_str(&format!("\n\nScratchpad directory: {}\nWorkers can read and write here without permission prompts. Use this for durable cross-worker knowledge — structure files however fits the work.", dir));
            }
        }
        
        Some(Self {
            worker_tools_context: content,
        })
    }
}

/// MCP 客户端信息
#[derive(Debug, Clone)]
pub struct McpClientInfo {
    /// 客户端名称
    pub name: String,
}

/// 获取工作器可用的工具
pub fn get_worker_tools() -> String {
    let mut tools = SimpleToolSet::new();
    
    if is_env_truthy("CLAUDE_CODE_SIMPLE") {
        // 简单模式：只允许基本工具
        tools.insert("Bash");
        tools.insert("Read");
        tools.insert("Edit");
    } else {
        // 标准模式：允许所有同步代理工具（排除内部工具）
        let allowed_tools = get_async_agent_allowed_tools();
        for tool in allowed_tools {
            if !INTERNAL_WORKER_TOOLS.contains(&tool.as_str()) {
                tools.insert(&tool);
            }
        }
    }
    
    tools.to_comma_string()
}

/// 获取异步代理允许的工具列表
fn get_async_agent_allowed_tools() -> Vec<String> {
    // 标准工具列表
    vec![
        "Bash".to_string(),
        "Read".to_string(),
        "Edit".to_string(),
        "Write".to_string(),
        "Glob".to_string(),
        "Grep".to_string(),
        "WebFetch".to_string(),
        "WebSearch".to_string(),
        "Agent".to_string(),
        "TaskCreate".to_string(),
        "TaskOutput".to_string(),
    ]
}

/// 检查 scratchpad gate 是否启用
fn is_scratchpad_gate_enabled() -> bool {
    // TODO: 与 GrowthBook 集成
    // 调用 checkStatsigFeatureGate_CACHED_MAY_BE_STALE('tengu_scratch')
    std::env::var("CLAUDE_CODE_SCRATCHPAD")
        .map(|v| is_env_truthy(&v))
        .unwrap_or(false)
}

/// 获取协调器系统提示
pub fn get_coordinator_system_prompt() -> String {
    let worker_capabilities = if is_env_truthy("CLAUDE_CODE_SIMPLE") {
        "Workers have access to Bash, Read, and Edit tools, plus MCP tools from configured MCP servers."
    } else {
        "Workers have access to standard tools, MCP tools from configured MCP servers, and project skills via the Skill tool. Delegate skill invocations (e.g. /commit, /verify) to workers."
    };
    
    format!(r#"You are Claude Code, an AI assistant that orchestrates software engineering tasks across multiple workers.

## 1. Your Role

You are a **coordinator**. Your job is to:
- Help the user achieve their goal
- Direct workers to research, implement and verify code changes
- Synthesize results and communicate with the user
- Answer questions directly when possible — don't delegate work that you can handle without tools

Every message you send is to the user. Worker results and system notifications are internal signals, not conversation partners — never thank or acknowledge them. Summarize new information for the user as it arrives.

## 2. Your Tools

- **Agent** - Spawn a new worker
- **SendMessage** - Continue an existing worker
- **TaskStop** - Stop a running worker

## 3. Workers

{worker_capabilities}

## 4. Task Workflow

Most tasks can be broken down into:
- Research (Workers parallel)
- Synthesis (Coordinator)
- Implementation (Workers)
- Verification (Workers)

When calling Agent, use subagent_type `worker`. Workers execute tasks autonomously."#)
}

/// 会话模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionMode {
    /// 协调器模式
    Coordinator,
    /// 正常模式
    Normal,
}

impl SessionMode {
    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "coordinator" => Some(Self::Coordinator),
            "normal" => Some(Self::Normal),
            _ => None,
        }
    }
    
    /// 转换为字符串
    pub fn as_str(&self) -> &str {
        match self {
            Self::Coordinator => "coordinator",
            Self::Normal => "normal",
        }
    }
}

/// 匹配会话模式
/// 如果不匹配，翻转环境变量使 isCoordinatorMode() 返回与恢复的会话相同的值
pub fn match_session_mode(session_mode: Option<SessionMode>) -> Option<String> {
    let stored_mode = session_mode?;
    
    let current_is_coordinator = is_coordinator_mode();
    let session_is_coordinator = stored_mode == SessionMode::Coordinator;
    
    if current_is_coordinator == session_is_coordinator {
        return None;
    }
    
    // 翻转环境变量
    if session_is_coordinator {
        std::env::set_var("CLAUDE_CODE_COORDINATOR_MODE", "1");
    } else {
        std::env::remove_var("CLAUDE_CODE_COORDINATOR_MODE");
    }
    
    // 记录事件
    tracing::info!("Coordinator mode switched to: {}", stored_mode.as_str());
    
    Some(if session_is_coordinator {
        "Entered coordinator mode to match resumed session.".to_string()
    } else {
        "Exited coordinator mode to match resumed session.".to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_coordinator_mode_default() {
        // 默认应该返回 false
        let result = is_coordinator_mode();
        assert!(!result);
    }
    
    #[test]
    fn test_is_env_truthy() {
        assert!(is_env_truthy("1"));
        assert!(is_env_truthy("true"));
        assert!(is_env_truthy("yes"));
        assert!(is_env_truthy("on"));
        assert!(!is_env_truthy("0"));
        assert!(!is_env_truthy("false"));
        assert!(!is_env_truthy(""));
    }
    
    #[test]
    fn test_simple_tool_set() {
        let mut tools = SimpleToolSet::new();
        tools.insert("Bash");
        tools.insert("Read");
        tools.insert("Edit");
        
        assert!(tools.contains("Bash"));
        assert!(tools.contains("Read"));
        assert!(!tools.contains("Write"));
        
        let list = tools.sorted_list();
        assert_eq!(list, vec!["Bash", "Edit", "Read"]);
    }
    
    #[test]
    fn test_session_mode() {
        assert_eq!(SessionMode::from_str("coordinator"), Some(SessionMode::Coordinator));
        assert_eq!(SessionMode::from_str("normal"), Some(SessionMode::Normal));
        assert_eq!(SessionMode::from_str("unknown"), None);
        
        assert_eq!(SessionMode::Coordinator.as_str(), "coordinator");
        assert_eq!(SessionMode::Normal.as_str(), "normal");
    }
    
    #[test]
    fn test_coordinator_user_context_disabled() {
        // 当协调器模式未启用时，返回 None
        let result = CoordinatorUserContext::new(&[], None);
        assert!(result.is_none());
    }
}
