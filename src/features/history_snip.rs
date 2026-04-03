//! HISTORY_SNIP 历史片段
//! 
//! 这个模块实现了上下文历史片段功能，用于高效管理对话历史。

use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// 片段类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnippetType {
    /// 用户消息
    UserMessage,
    
    /// AI 回复
    AiResponse,
    
    /// 工具调用
    ToolCall,
    
    /// 工具结果
    ToolResult,
    
    /// 系统提示
    SystemPrompt,
    
    /// 压缩摘要
    CompressedSummary,
}

/// 历史片段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySnippet {
    /// 片段 ID
    pub id: String,
    
    /// 片段类型
    pub snippet_type: SnippetType,
    
    /// 片段内容
    pub content: String,
    
    /// 创建时间
    pub created_at: String,
    
    /// 优先级（用于保留决策）
    pub priority: u8,
    
    /// Token 数量
    pub token_count: usize,
    
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 压缩策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionStrategy {
    /// 保留最近的 N 条
    KeepRecent(usize),
    
    /// 基于优先级
    PriorityBased,
    
    /// 智能压缩（AI 摘要）
    Smart,
    
    /// 混合策略
    Hybrid,
}

impl Default for CompressionStrategy {
    fn default() -> Self {
        CompressionStrategy::KeepRecent(50)
    }
}

/// 历史片段管理器
#[derive(Debug)]
pub struct HistorySnipManager {
    /// 应用状态
    state: AppState,
    
    /// 片段队列
    snippets: VecDeque<HistorySnippet>,
    
    /// 压缩策略
    compression_strategy: CompressionStrategy,
    
    /// 最大 Token 数
    max_tokens: usize,
    
    /// 当前 Token 计数
    current_tokens: usize,
    
    /// 已压缩的摘要
    compressed_summaries: Vec<HistorySnippet>,
}

impl HistorySnipManager {
    /// 创建新的历史片段管理器
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            snippets: VecDeque::new(),
            compression_strategy: CompressionStrategy::default(),
            max_tokens: 100000,
            current_tokens: 0,
            compressed_summaries: Vec::new(),
        }
    }
    
    /// 添加片段
    pub fn add_snippet(&mut self, snippet: HistorySnippet) {
        self.current_tokens += snippet.token_count;
        self.snippets.push_back(snippet);
        
        self.check_and_compress();
    }
    
    /// 检查并执行压缩
    fn check_and_compress(&mut self) {
        while self.current_tokens > self.max_tokens {
            self.compress_once();
        }
    }
    
    /// 执行一次压缩
    fn compress_once(&mut self) {
        match self.compression_strategy {
            CompressionStrategy::KeepRecent(n) => {
                while self.snippets.len() > n {
                    if let Some(snippet) = self.snippets.pop_front() {
                        self.current_tokens -= snippet.token_count;
                    }
                }
            }
            CompressionStrategy::PriorityBased => {
                if let Some(index) = self.snippets.iter().position(|s| s.priority <= 3) {
                    if let Some(snippet) = self.snippets.remove(index) {
                        self.current_tokens -= snippet.token_count;
                    }
                }
            }
            CompressionStrategy::Smart | CompressionStrategy::Hybrid => {
                if self.snippets.len() >= 10 {
                    self.create_compressed_summary();
                }
            }
        }
    }
    
    /// 创建压缩摘要
    fn create_compressed_summary(&mut self) {
        let to_compress: Vec<_> = self.snippets.drain(0..10).collect();
        
        let summary_tokens: usize = to_compress.iter().map(|s| s.token_count).sum();
        
        let summary = HistorySnippet {
            id: generate_snippet_id(),
            snippet_type: SnippetType::CompressedSummary,
            content: "Previous conversation compressed...".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            priority: 10,
            token_count: summary_tokens / 10,
            metadata: HashMap::new(),
        };
        
        self.current_tokens -= summary_tokens;
        self.current_tokens += summary.token_count;
        
        self.compressed_summaries.push(summary);
    }
    
    /// 获取所有片段
    pub fn get_snippets(&self) -> Vec<&HistorySnippet> {
        self.snippets.iter().collect()
    }
    
    /// 获取最近的 N 条片段
    pub fn get_recent_snippets(&self, count: usize) -> Vec<&HistorySnippet> {
        let start = self.snippets.len().saturating_sub(count);
        self.snippets.range(start..).collect()
    }
    
    /// 按类型过滤片段
    pub fn get_snippets_by_type(&self, snippet_type: SnippetType) -> Vec<&HistorySnippet> {
        self.snippets.iter().filter(|s| s.snippet_type == snippet_type).collect()
    }
    
    /// 设置压缩策略
    pub fn set_compression_strategy(&mut self, strategy: CompressionStrategy) {
        self.compression_strategy = strategy;
    }
    
    /// 设置最大 Token 数
    pub fn set_max_tokens(&mut self, max_tokens: usize) {
        self.max_tokens = max_tokens;
        self.check_and_compress();
    }
    
    /// 获取当前 Token 数
    pub fn current_token_count(&self) -> usize {
        self.current_tokens
    }
    
    /// 清空所有片段
    pub fn clear(&mut self) {
        self.snippets.clear();
        self.compressed_summaries.clear();
        self.current_tokens = 0;
    }
    
    /// 创建用户消息片段
    pub fn create_user_message(content: String, token_count: usize) -> HistorySnippet {
        HistorySnippet {
            id: generate_snippet_id(),
            snippet_type: SnippetType::UserMessage,
            content,
            created_at: chrono::Utc::now().to_rfc3339(),
            priority: 8,
            token_count,
            metadata: HashMap::new(),
        }
    }
    
    /// 创建 AI 回复片段
    pub fn create_ai_response(content: String, token_count: usize) -> HistorySnippet {
        HistorySnippet {
            id: generate_snippet_id(),
            snippet_type: SnippetType::AiResponse,
            content,
            created_at: chrono::Utc::now().to_rfc3339(),
            priority: 9,
            token_count,
            metadata: HashMap::new(),
        }
    }
}

/// 生成片段 ID
fn generate_snippet_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("snippet_{}", rng.gen::<u64>())
}
