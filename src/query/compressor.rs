//! 上下文压缩器
//!
//! 实现上下文压缩功能，包括多种压缩策略。

use super::message::{Message, MessageContent};
use super::result::QueryError;

/// 压缩器配置
#[derive(Debug, Clone)]
pub struct CompressorConfig {
    /// 最大上下文长度（token数）
    pub max_context_tokens: usize,
    /// 保留最新消息数
    pub keep_recent_messages: usize,
    /// 压缩阈值（百分比）
    pub compression_threshold: f32,
    /// 启用智能压缩
    pub enable_smart_compression: bool,
}

impl Default for CompressorConfig {
    fn default() -> Self {
        Self {
            max_context_tokens: 128000, // Claude 上下文窗口
            keep_recent_messages: 10,
            compression_threshold: 0.8, // 80%时开始压缩
            enable_smart_compression: true,
        }
    }
}

/// 上下文压缩器
pub struct ContextCompressor {
    config: CompressorConfig,
}

impl ContextCompressor {
    /// 创建新压缩器
    pub fn new(max_context_tokens: usize) -> Self {
        Self {
            config: CompressorConfig {
                max_context_tokens,
                ..Default::default()
            },
        }
    }

    /// 创建带配置的压缩器
    pub fn with_config(config: CompressorConfig) -> Self {
        Self { config }
    }

    /// 压缩消息列表
    pub async fn compress(&self, messages: &[Message]) -> Result<Vec<Message>, QueryError> {
        if messages.len() <= 1 {
            return Ok(messages.to_vec());
        }

        // 估计当前 token 使用量
        let estimated_tokens = self.estimate_tokens(messages);

        // 检查是否需要压缩
        if estimated_tokens <= self.config.max_context_tokens {
            return Ok(messages.to_vec());
        }

        // 计算需要压缩的比例
        let excess_ratio = estimated_tokens as f32 / self.config.max_context_tokens as f32;
        let compression_ratio = 1.0 / excess_ratio;

        debug!("Compressing context: {} tokens, ratio: {:.2}", estimated_tokens, compression_ratio);

        // 应用压缩策略
        let compressed = self.apply_compression_strategy(messages, compression_ratio).await?;

        let compressed_tokens = self.estimate_tokens(&compressed);
        debug!("After compression: {} tokens (reduction: {:.1}%)",
            compressed_tokens,
            (1.0 - compressed_tokens as f32 / estimated_tokens as f32) * 100.0);

        Ok(compressed)
    }

    /// 应用压缩策略
    async fn apply_compression_strategy(
        &self,
        messages: &[Message],
        compression_ratio: f32,
    ) -> Result<Vec<Message>, QueryError> {
        if !self.config.enable_smart_compression {
            // 简单策略：保留最新消息
            return self.simple_compression(messages, compression_ratio);
        }

        // 智能压缩：优先保留重要消息
        self.smart_compression(messages, compression_ratio).await
    }

    /// 简单压缩：保留最新消息
    fn simple_compression(
        &self,
        messages: &[Message],
        compression_ratio: f32,
    ) -> Result<Vec<Message>, QueryError> {
        let keep_count = (messages.len() as f32 * compression_ratio) as usize;
        let keep_count = keep_count.max(self.config.keep_recent_messages).min(messages.len());

        // 保留最新的消息
        let compressed = messages[messages.len() - keep_count..].to_vec();

        Ok(compressed)
    }

    /// 智能压缩：分析消息重要性
    async fn smart_compression(
        &self,
        messages: &[Message],
        compression_ratio: f32,
    ) -> Result<Vec<Message>, QueryError> {
        // TODO: 实现智能压缩算法
        // 1. 分析消息重要性（系统消息、用户查询、工具结果等）
        // 2. 计算消息权重
        // 3. 根据权重和压缩比率选择保留的消息

        // 暂时使用简单压缩
        self.simple_compression(messages, compression_ratio)
    }

    /// 估计消息的 token 数
    fn estimate_tokens(&self, messages: &[Message]) -> usize {
        // 简单估计：4个字符约等于1个token
        let total_chars: usize = messages
            .iter()
            .map(|msg| {
                match &msg.content {
                    MessageContent::Text(text) => text.len(),
                    MessageContent::ToolCalls(_) => 100, // 估计值
                    MessageContent::ToolResult(result) => result.content.len(),
                }
            })
            .sum();

        total_chars / 4
    }

    /// 计算消息重要性分数
    fn message_importance(&self, message: &Message) -> f32 {
        match message.role {
            super::message::MessageRole::System => 1.0, // 系统消息最重要
            super::message::MessageRole::User => 0.8,   // 用户消息重要
            super::message::MessageRole::Assistant => 0.6, // 助理响应
            super::message::MessageRole::Tool => 0.4,   // 工具结果
        }
    }

    /// 检查是否应该保留消息
    fn should_keep_message(&self, message: &Message, index: usize, total: usize) -> bool {
        // 总是保留第一条系统消息
        if index == 0 && matches!(message.role, super::message::MessageRole::System) {
            return true;
        }

        // 保留最新的消息
        if total - index <= self.config.keep_recent_messages {
            return true;
        }

        // 根据重要性决定
        let importance = self.message_importance(message);
        importance > 0.5
    }
}