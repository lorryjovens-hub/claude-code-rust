//! 分叉代理机制
//! 
//! 这个模块实现了分叉代理功能，对应 TypeScript 的 utils/forkedAgent.ts

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 缓存安全参数
/// 
/// 这些参数必须在父代理和子代理之间保持一致，以确保提示缓存命中
#[derive(Debug, Clone)]
pub struct CacheSafeParams {
    /// 系统提示 - 必须与父代理匹配才能命中缓存
    pub system_prompt: String,
    
    /// 用户上下文 - 添加到消息前面，影响缓存
    pub user_context: HashMap<String, String>,
    
    /// 系统上下文 - 添加到系统提示后面，影响缓存
    pub system_context: HashMap<String, String>,
    
    /// 父代理上下文消息 - 用于提示缓存共享
    pub fork_context_messages: Vec<String>,
}

impl CacheSafeParams {
    /// 创建新的缓存安全参数
    pub fn new(system_prompt: String) -> Self {
        Self {
            system_prompt,
            user_context: HashMap::new(),
            system_context: HashMap::new(),
            fork_context_messages: Vec::new(),
        }
    }
    
    /// 添加用户上下文
    pub fn with_user_context(mut self, key: String, value: String) -> Self {
        self.user_context.insert(key, value);
        self
    }
    
    /// 添加系统上下文
    pub fn with_system_context(mut self, key: String, value: String) -> Self {
        self.system_context.insert(key, value);
        self
    }
    
    /// 设置上下文消息
    pub fn with_context_messages(mut self, messages: Vec<String>) -> Self {
        self.fork_context_messages = messages;
        self
    }
}

/// 分叉代理参数
pub struct ForkedAgentParams {
    /// 启动分叉查询循环的消息
    pub prompt_messages: Vec<String>,
    
    /// 必须与父查询匹配的缓存安全参数
    pub cache_safe_params: CacheSafeParams,
    
    /// 分叉代理的权限检查函数名称
    pub can_use_tool: String,
    
    /// 跟踪用的源标识符
    pub query_source: String,
    
    /// 分析标签（例如 'session_memory', 'supervisor'）
    pub fork_label: String,
    
    /// 可选的子代理上下文覆盖
    pub overrides: Option<SubagentContextOverrides>,
    
    /// 可选的输出 token 上限
    /// 注意：设置此参数会改变 max_tokens 和 budget_tokens
    pub max_output_tokens: Option<u32>,
    
    /// 可选的轮次上限（API 往返次数）
    pub max_turns: Option<u32>,
    
    /// 可选的消息回调（用于流式 UI）
    pub on_message: Option<Arc<dyn Fn(String) + Send + Sync>>,
    
    /// 跳过侧链转录记录
    pub skip_transcript: bool,
    
    /// 跳过最后一条消息的提示缓存写入
    pub skip_cache_write: bool,
}

impl Clone for ForkedAgentParams {
    fn clone(&self) -> Self {
        Self {
            prompt_messages: self.prompt_messages.clone(),
            cache_safe_params: self.cache_safe_params.clone(),
            can_use_tool: self.can_use_tool.clone(),
            query_source: self.query_source.clone(),
            fork_label: self.fork_label.clone(),
            overrides: self.overrides.clone(),
            max_output_tokens: self.max_output_tokens,
            max_turns: self.max_turns,
            on_message: self.on_message.clone(),
            skip_transcript: self.skip_transcript,
            skip_cache_write: self.skip_cache_write,
        }
    }
}

impl ForkedAgentParams {
    /// 创建新的分叉代理参数
    pub fn new(
        prompt_messages: Vec<String>,
        cache_safe_params: CacheSafeParams,
        fork_label: String,
    ) -> Self {
        Self {
            prompt_messages,
            cache_safe_params,
            can_use_tool: String::new(),
            query_source: "fork".to_string(),
            fork_label,
            overrides: None,
            max_output_tokens: None,
            max_turns: None,
            on_message: None,
            skip_transcript: false,
            skip_cache_write: false,
        }
    }
}

/// 子代理上下文覆盖
#[derive(Debug, Clone, Default)]
pub struct SubagentContextOverrides {
    /// 文件状态缓存
    pub file_state_cache: Option<String>,
    
    /// 权限上下文
    pub permission_context: Option<String>,
}

/// 分叉代理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkedAgentResult {
    /// 查询循环期间产生的所有消息
    pub messages: Vec<String>,
    
    /// 循环中所有 API 调用的累计使用量
    pub total_usage: TokenUsage,
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
    pub cache_read_input_tokens: Option<u64>,
    
    /// 缓存创建 token 数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u64>,
}

impl Default for TokenUsage {
    fn default() -> Self {
        Self {
            input_tokens: 0,
            output_tokens: 0,
            cache_read_input_tokens: None,
            cache_creation_input_tokens: None,
        }
    }
}

impl TokenUsage {
    /// 创建新的 token 使用统计
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 累加使用量
    pub fn accumulate(&mut self, other: &TokenUsage) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        
        if let Some(cache_read) = other.cache_read_input_tokens {
            self.cache_read_input_tokens = Some(
                self.cache_read_input_tokens.unwrap_or(0) + cache_read
            );
        }
        
        if let Some(cache_creation) = other.cache_creation_input_tokens {
            self.cache_creation_input_tokens = Some(
                self.cache_creation_input_tokens.unwrap_or(0) + cache_creation
            );
        }
    }
}

/// 分叉代理
pub struct ForkedAgent {
    /// 参数
    params: ForkedAgentParams,
    
    /// 是否已运行
    has_run: Arc<RwLock<bool>>,
}

impl ForkedAgent {
    /// 创建新的分叉代理
    pub fn new(params: ForkedAgentParams) -> Self {
        Self {
            params,
            has_run: Arc::new(RwLock::new(false)),
        }
    }
    
    /// 运行分叉代理
    pub async fn run(&self) -> Result<ForkedAgentResult> {
        if *self.has_run.read().await {
            return Err(crate::error::ClaudeError::Agent("Agent already run".to_string()));
        }
        
        *self.has_run.write().await = true;
        
        // TODO: 实现实际的分叉代理逻辑
        // 1. 准备上下文
        // 2. 调用 API
        // 3. 处理响应
        // 4. 累计使用量
        
        // 模拟执行
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        Ok(ForkedAgentResult {
            messages: self.params.prompt_messages.clone(),
            total_usage: TokenUsage::new(),
        })
    }
}

/// 全局缓存安全参数存储
static LAST_CACHE_SAFE_PARAMS: once_cell::sync::Lazy<Arc<RwLock<Option<CacheSafeParams>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(None)));

/// 保存缓存安全参数
pub async fn save_cache_safe_params(params: Option<CacheSafeParams>) {
    *LAST_CACHE_SAFE_PARAMS.write().await = params;
}

/// 获取最后的缓存安全参数
pub async fn get_last_cache_safe_params() -> Option<CacheSafeParams> {
    LAST_CACHE_SAFE_PARAMS.read().await.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_safe_params() {
        let params = CacheSafeParams::new("test prompt".to_string())
            .with_user_context("key".to_string(), "value".to_string())
            .with_system_context("sys_key".to_string(), "sys_value".to_string());
        
        assert_eq!(params.system_prompt, "test prompt");
        assert_eq!(params.user_context.get("key"), Some(&"value".to_string()));
    }
    
    #[test]
    fn test_forked_agent_params() {
        let cache_params = CacheSafeParams::new("test prompt".to_string());
        let params = ForkedAgentParams::new(
            vec!["message1".to_string()],
            cache_params,
            "test_label".to_string(),
        );
        
        assert_eq!(params.fork_label, "test_label");
        assert_eq!(params.prompt_messages.len(), 1);
    }
    
    #[tokio::test]
    async fn test_forked_agent() {
        let cache_params = CacheSafeParams::new("test prompt".to_string());
        let params = ForkedAgentParams::new(
            vec!["message1".to_string()],
            cache_params,
            "test_label".to_string(),
        );
        
        let agent = ForkedAgent::new(params);
        let result = agent.run().await.unwrap();
        
        assert_eq!(result.messages.len(), 1);
    }
    
    #[tokio::test]
    async fn test_global_cache_params() {
        let params = CacheSafeParams::new("test prompt".to_string());
        
        save_cache_safe_params(Some(params.clone())).await;
        
        let retrieved = get_last_cache_safe_params().await;
        assert!(retrieved.is_some());
        
        save_cache_safe_params(None).await;
        let retrieved = get_last_cache_safe_params().await;
        assert!(retrieved.is_none());
    }
}
