//! 代理类型系统
//! 
//! 这个模块定义了代理系统的核心类型

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 代理 ID
pub type AgentId = String;

/// 代理类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    /// 通用代理 - 处理通用任务，全工具集访问权限
    GeneralPurpose,
    
    /// 探索代理 - 代码探索分析，只读工具集限制
    Explore,
    
    /// 规划代理 - 规划设计任务，分析工具集访问
    Plan,
    
    /// 验证代理 - 验证测试任务，测试工具集访问
    Verification,
    
    /// Bash 代理 - 命令执行任务，Bash工具+文件工具集
    Bash,
    
    /// 自定义代理
    Custom(String),
}

impl AgentType {
    /// 获取代理类型的工具集
    pub fn get_tool_set(&self) -> Vec<String> {
        match self {
            AgentType::GeneralPurpose => vec![
                "Bash", "Read", "Edit", "Write", "Glob", "Grep",
                "WebFetch", "WebSearch", "Agent", "TaskCreate", "TaskOutput",
            ].iter().map(|s| s.to_string()).collect(),
            
            AgentType::Explore => vec![
                "Read", "Glob", "Grep", "WebFetch",
            ].iter().map(|s| s.to_string()).collect(),
            
            AgentType::Plan => vec![
                "Read", "Glob", "Grep", "WebFetch", "WebSearch",
            ].iter().map(|s| s.to_string()).collect(),
            
            AgentType::Verification => vec![
                "Bash", "Read", "Glob", "Grep",
            ].iter().map(|s| s.to_string()).collect(),
            
            AgentType::Bash => vec![
                "Bash", "Read", "Edit", "Write",
            ].iter().map(|s| s.to_string()).collect(),
            
            AgentType::Custom(_) => vec![
                "Bash", "Read", "Edit", "Write", "Glob", "Grep",
            ].iter().map(|s| s.to_string()).collect(),
        }
    }
    
    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "general-purpose" => Some(Self::GeneralPurpose),
            "explore" => Some(Self::Explore),
            "plan" => Some(Self::Plan),
            "verification" => Some(Self::Verification),
            "bash" => Some(Self::Bash),
            _ => Some(Self::Custom(s.to_string())),
        }
    }
    
    /// 转换为字符串
    pub fn as_str(&self) -> &str {
        match self {
            Self::GeneralPurpose => "general-purpose",
            Self::Explore => "explore",
            Self::Plan => "plan",
            Self::Verification => "verification",
            Self::Bash => "bash",
            Self::Custom(s) => s,
        }
    }
}

/// 代理定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    /// 代理名称
    pub name: String,
    
    /// 代理类型
    #[serde(rename = "type")]
    pub agent_type: AgentType,
    
    /// 代理描述
    pub description: String,
    
    /// 系统提示
    pub system_prompt: String,
    
    /// 用户上下文
    #[serde(default)]
    pub user_context: HashMap<String, String>,
    
    /// 系统上下文
    #[serde(default)]
    pub system_context: HashMap<String, String>,
    
    /// 工具集
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    
    /// 最大输出 token 数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    
    /// 最大轮次
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_turns: Option<u32>,
    
    /// 是否启用
    pub enabled: bool,
}

impl AgentDefinition {
    /// 创建新的代理定义
    pub fn new(name: String, agent_type: AgentType, description: String) -> Self {
        let tools = agent_type.get_tool_set();
        
        Self {
            name,
            agent_type,
            description,
            system_prompt: String::new(),
            user_context: HashMap::new(),
            system_context: HashMap::new(),
            tools: Some(tools),
            max_output_tokens: None,
            max_turns: None,
            enabled: true,
        }
    }
    
    /// 设置系统提示
    pub fn with_system_prompt(mut self, prompt: String) -> Self {
        self.system_prompt = prompt;
        self
    }
    
    /// 设置工具集
    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.tools = Some(tools);
        self
    }
    
    /// 设置最大输出 token 数
    pub fn with_max_output_tokens(mut self, max: u32) -> Self {
        self.max_output_tokens = Some(max);
        self
    }
    
    /// 设置最大轮次
    pub fn with_max_turns(mut self, max: u32) -> Self {
        self.max_turns = Some(max);
        self
    }
}

/// 代理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// 代理定义
    #[serde(flatten)]
    pub definition: AgentDefinition,
    
    /// 颜色标识
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    
    /// 父代理 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<AgentId>,
}

impl AgentConfig {
    /// 创建新的代理配置
    pub fn new(definition: AgentDefinition) -> Self {
        Self {
            definition,
            color: None,
            parent_id: None,
        }
    }
}

/// 代理执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    /// 代理 ID
    pub agent_id: AgentId,
    
    /// 输出消息
    pub messages: Vec<String>,
    
    /// 使用的 token 数
    pub usage: TokenUsage,
    
    /// 执行状态
    pub status: AgentStatus,
    
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Token 使用统计
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TokenUsage {
    /// 输入 token 数
    pub input_tokens: u64,
    
    /// 输出 token 数
    pub output_tokens: u64,
    
    /// 缓存读取 token 数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_tokens: Option<u64>,
    
    /// 缓存写入 token 数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_write_tokens: Option<u64>,
}

impl Default for TokenUsage {
    fn default() -> Self {
        Self {
            input_tokens: 0,
            output_tokens: 0,
            cache_read_tokens: None,
            cache_write_tokens: None,
        }
    }
}

impl TokenUsage {
    /// 创建新的 token 使用统计
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 添加使用量
    pub fn add(&mut self, other: &TokenUsage) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        
        if let Some(cache_read) = other.cache_read_tokens {
            self.cache_read_tokens = Some(self.cache_read_tokens.unwrap_or(0) + cache_read);
        }
        
        if let Some(cache_write) = other.cache_write_tokens {
            self.cache_write_tokens = Some(self.cache_write_tokens.unwrap_or(0) + cache_write);
        }
    }
    
    /// 获取总 token 数
    pub fn total(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }
}

/// 代理执行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    /// 空闲
    Idle,
    
    /// 运行中
    Running,
    
    /// 已完成
    Completed,
    
    /// 已取消
    Cancelled,
    
    /// 错误
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_type_tool_set() {
        let general = AgentType::GeneralPurpose;
        assert!(general.get_tool_set().contains(&"Bash".to_string()));
        assert!(general.get_tool_set().contains(&"Read".to_string()));
        
        let explore = AgentType::Explore;
        assert!(explore.get_tool_set().contains(&"Read".to_string()));
        assert!(!explore.get_tool_set().contains(&"Bash".to_string()));
    }
    
    #[test]
    fn test_agent_type_conversion() {
        assert_eq!(AgentType::from_str("general-purpose"), Some(AgentType::GeneralPurpose));
        assert_eq!(AgentType::from_str("explore"), Some(AgentType::Explore));
        
        assert_eq!(AgentType::GeneralPurpose.as_str(), "general-purpose");
        assert_eq!(AgentType::Explore.as_str(), "explore");
    }
    
    #[test]
    fn test_agent_definition() {
        let agent = AgentDefinition::new(
            "test-agent".to_string(),
            AgentType::GeneralPurpose,
            "Test agent".to_string(),
        );
        
        assert_eq!(agent.name, "test-agent");
        assert_eq!(agent.agent_type, AgentType::GeneralPurpose);
        assert!(agent.enabled);
    }
    
    #[test]
    fn test_token_usage() {
        let mut usage = TokenUsage::new();
        usage.add(&TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
            cache_read_tokens: Some(20),
            cache_write_tokens: Some(10),
        });
        
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.cache_read_tokens, Some(20));
        assert_eq!(usage.total(), 150);
    }
}
