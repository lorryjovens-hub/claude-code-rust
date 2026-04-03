//! 上下文压缩服务
//! 
//! 实现三级压缩策略：
//! - 微压缩：单轮对话压缩
//! - 会话压缩：整个会话摘要
//! - 记忆压缩：长期记忆提取

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod grouping;
pub mod projection;

pub use grouping::group_messages;
pub use projection::project_to_essential;

/// 压缩级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompactLevel {
    /// 微压缩 - 单轮对话压缩
    Micro,
    /// 会话压缩 - 整个会话摘要
    Session,
    /// 记忆压缩 - 长期记忆提取
    Memory,
}

impl Default for CompactLevel {
    fn default() -> Self {
        Self::Micro
    }
}

/// 压缩配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactConfig {
    /// 压缩级别
    pub level: CompactLevel,
    /// 目标压缩率
    pub target_ratio: f32,
    /// 最大 token 数
    pub max_tokens: usize,
    /// 保留最近消息数
    pub keep_recent: usize,
}

impl Default for CompactConfig {
    fn default() -> Self {
        Self {
            level: CompactLevel::default(),
            target_ratio: 0.6,
            max_tokens: 4000,
            keep_recent: 10,
        }
    }
}

/// 压缩结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactResult {
    /// 压缩后的消息
    pub messages: Vec<String>,
    /// 压缩前 token 数
    pub pre_tokens: usize,
    /// 压缩后 token 数
    pub post_tokens: usize,
    /// 压缩率
    pub ratio: f32,
    /// 摘要
    pub summary: Option<String>,
}

/// 微压缩服务
pub struct MicrocompactService {
    config: CompactConfig,
}

impl MicrocompactService {
    pub fn new(config: Option<CompactConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
        }
    }

    pub async fn compact(&self, messages: &[String]) -> crate::error::Result<CompactResult> {
        let pre_tokens = self.estimate_tokens(messages);
        
        let groups = group_messages(messages);
        let mut compacted = Vec::new();

        for group in groups {
            let essential = project_to_essential(&group.messages);
            compacted.push(essential);
        }

        let post_tokens = self.estimate_tokens(&compacted);
        let ratio = post_tokens as f32 / pre_tokens as f32;

        Ok(CompactResult {
            messages: compacted,
            pre_tokens,
            post_tokens,
            ratio,
            summary: None,
        })
    }

    fn estimate_tokens(&self, messages: &[String]) -> usize {
        messages.iter().map(|m| m.len() / 4).sum()
    }
}

/// 会话压缩服务
pub struct SessionCompactService {
    config: CompactConfig,
}

impl SessionCompactService {
    pub fn new(config: Option<CompactConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
        }
    }

    pub async fn compact(&self, messages: &[String]) -> crate::error::Result<CompactResult> {
        let pre_tokens = self.estimate_tokens(messages);

        let summary = self.generate_summary(messages).await?;

        let keep_count = self.config.keep_recent.min(messages.len());
        let recent: Vec<String> = messages.iter().rev().take(keep_count).cloned().collect();
        
        let mut compacted = vec![summary.clone()];
        compacted.extend(recent.into_iter().rev());

        let post_tokens = self.estimate_tokens(&compacted);
        let ratio = post_tokens as f32 / pre_tokens as f32;

        Ok(CompactResult {
            messages: compacted,
            pre_tokens,
            post_tokens,
            ratio,
            summary: Some(summary),
        })
    }

    async fn generate_summary(&self, messages: &[String]) -> crate::error::Result<String> {
        Ok(format!("Session summary: {} messages compressed", messages.len()))
    }

    fn estimate_tokens(&self, messages: &[String]) -> usize {
        messages.iter().map(|m| m.len() / 4).sum()
    }
}

/// 记忆压缩服务
pub struct MemoryCompactService {
    config: CompactConfig,
    memories: Arc<RwLock<HashMap<String, MemoryEntry>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub key: String,
    pub content: String,
    pub importance: f32,
    pub created_at: i64,
}

impl MemoryCompactService {
    pub fn new(config: Option<CompactConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            memories: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn compact(&self, messages: &[String]) -> crate::error::Result<CompactResult> {
        let pre_tokens = self.estimate_tokens(messages);

        let extracted = self.extract_memories(messages).await?;
        
        let mut memories = self.memories.write().await;
        for memory in &extracted {
            memories.insert(memory.key.clone(), memory.clone());
        }

        let summary = self.generate_memory_summary(&extracted);

        let post_tokens = self.estimate_tokens(&[summary.clone()]);
        let ratio = post_tokens as f32 / pre_tokens as f32;

        Ok(CompactResult {
            messages: vec![summary],
            pre_tokens,
            post_tokens,
            ratio,
            summary: Some(format!("Extracted {} memories", extracted.len())),
        })
    }

    async fn extract_memories(&self, messages: &[String]) -> crate::error::Result<Vec<MemoryEntry>> {
        let mut memories = Vec::new();
        
        for (i, msg) in messages.iter().enumerate() {
            if msg.contains("important") || msg.contains("key") || msg.contains("remember") {
                memories.push(MemoryEntry {
                    key: format!("memory_{}", i),
                    content: msg.clone(),
                    importance: 0.8,
                    created_at: chrono::Utc::now().timestamp_millis(),
                });
            }
        }

        Ok(memories)
    }

    fn generate_memory_summary(&self, memories: &[MemoryEntry]) -> String {
        let mut summary = String::new();
        summary.push_str("# Extracted Memories\n\n");
        
        for memory in memories {
            summary.push_str(&format!("## {}\n{}\n\n", memory.key, memory.content));
        }

        summary
    }

    fn estimate_tokens(&self, messages: &[String]) -> usize {
        messages.iter().map(|m| m.len() / 4).sum()
    }

    pub async fn get_memories(&self) -> Vec<MemoryEntry> {
        self.memories.read().await.values().cloned().collect()
    }

    pub async fn clear(&self) {
        self.memories.write().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_microcompact() {
        let service = MicrocompactService::new(None);
        let messages = vec![
            "Hello world".to_string(),
            "This is a test message".to_string(),
            "Another message here".to_string(),
        ];

        let result = service.compact(&messages).await.unwrap();
        assert!(result.ratio <= 1.0);
    }

    #[tokio::test]
    async fn test_session_compact() {
        let service = SessionCompactService::new(None);
        let messages: Vec<String> = (0..20).map(|i| format!("Message {}", i)).collect();

        let result = service.compact(&messages).await.unwrap();
        assert!(result.summary.is_some());
        assert!(result.ratio < 1.0);
    }

    #[tokio::test]
    async fn test_memory_compact() {
        let service = MemoryCompactService::new(None);
        let messages = vec![
            "This is important information".to_string(),
            "Regular message".to_string(),
            "Remember this key point".to_string(),
        ];

        let result = service.compact(&messages).await.unwrap();
        assert!(result.summary.is_some());

        let memories = service.get_memories().await;
        assert!(!memories.is_empty());
    }
}
