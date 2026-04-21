//! Context Management - Context window and token management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEntry {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub token_count: usize,
    pub priority: ContextPriority,
    pub source: ContextSource,
}

impl ContextEntry {
    pub fn new(role: &str, content: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            role: role.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
            token_count: Self::estimate_tokens(content),
            priority: ContextPriority::Normal,
            source: ContextSource::User,
        }
    }
    
    pub fn system(content: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            role: "system".to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
            token_count: Self::estimate_tokens(content),
            priority: ContextPriority::Critical,
            source: ContextSource::System,
        }
    }
    
    pub fn assistant(content: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            role: "assistant".to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
            token_count: Self::estimate_tokens(content),
            priority: ContextPriority::Normal,
            source: ContextSource::Assistant,
        }
    }
    
    pub fn with_priority(mut self, priority: ContextPriority) -> Self {
        self.priority = priority;
        self
    }
    
    fn estimate_tokens(text: &str) -> usize {
        text.split_whitespace().count() / 3 * 4
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContextSource {
    System,
    User,
    Assistant,
    Tool,
    Memory,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindow {
    pub max_tokens: usize,
    pub reserved_tokens: usize,
    pub entries: VecDeque<ContextEntry>,
    pub total_tokens: usize,
}

impl ContextWindow {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            reserved_tokens: max_tokens / 10,
            entries: VecDeque::new(),
            total_tokens: 0,
        }
    }
    
    pub fn available_tokens(&self) -> usize {
        self.max_tokens.saturating_sub(self.reserved_tokens).saturating_sub(self.total_tokens)
    }
    
    pub fn can_fit(&self, tokens: usize) -> bool {
        self.available_tokens() >= tokens
    }
    
    pub fn add(&mut self, entry: ContextEntry) -> bool {
        if !self.can_fit(entry.token_count) {
            self.evict(entry.token_count);
        }
        
        if self.can_fit(entry.token_count) {
            self.total_tokens += entry.token_count;
            self.entries.push_back(entry);
            return true;
        }
        
        false
    }
    
    fn evict(&mut self, needed_tokens: usize) {
        let mut freed = 0;
        
        let mut to_remove = Vec::new();
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.priority == ContextPriority::Critical {
                continue;
            }
            
            if freed >= needed_tokens {
                break;
            }
            
            freed += entry.token_count;
            to_remove.push(i);
        }
        
        for i in to_remove.into_iter().rev() {
            if let Some(entry) = self.entries.remove(i) {
                self.total_tokens -= entry.token_count;
            }
        }
    }
    
    pub fn clear(&mut self) {
        self.entries.clear();
        self.total_tokens = 0;
    }
    
    pub fn to_messages(&self) -> Vec<crate::api::ChatMessage> {
        self.entries.iter()
            .map(|e| crate::api::ChatMessage {
                role: e.role.clone(),
                content: Some(e.content.clone()),
                tool_calls: None,
                tool_call_id: None,
            })
            .collect()
    }
}

pub struct ContextManager {
    window: Arc<RwLock<ContextWindow>>,
    summaries: Arc<RwLock<Vec<ContextSummary>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSummary {
    pub id: String,
    pub summary: String,
    pub original_entries: usize,
    pub original_tokens: usize,
    pub created_at: DateTime<Utc>,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            window: Arc::new(RwLock::new(ContextWindow::new(128000))),
            summaries: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub fn with_max_tokens(max_tokens: usize) -> Self {
        Self {
            window: Arc::new(RwLock::new(ContextWindow::new(max_tokens))),
            summaries: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub async fn add(&self, entry: ContextEntry) -> bool {
        let mut window = self.window.write().await;
        window.add(entry)
    }
    
    pub async fn add_user(&self, content: &str) -> bool {
        self.add(ContextEntry::new("user", content)).await
    }
    
    pub async fn add_assistant(&self, content: &str) -> bool {
        self.add(ContextEntry::assistant(content)).await
    }
    
    pub async fn add_system(&self, content: &str) -> bool {
        self.add(ContextEntry::system(content)).await
    }
    
    pub async fn get_messages(&self) -> Vec<crate::api::ChatMessage> {
        let window = self.window.read().await;
        window.to_messages()
    }
    
    pub async fn get_entries(&self) -> Vec<ContextEntry> {
        let window = self.window.read().await;
        window.entries.iter().cloned().collect()
    }
    
    pub async fn clear(&self) {
        let mut window = self.window.write().await;
        window.clear();
    }
    
    pub async fn stats(&self) -> ContextStats {
        let window = self.window.read().await;
        ContextStats {
            total_entries: window.entries.len(),
            total_tokens: window.total_tokens,
            max_tokens: window.max_tokens,
            available_tokens: window.available_tokens(),
            utilization: window.total_tokens as f64 / window.max_tokens as f64,
        }
    }
    
    pub async fn summarize(&self, summary: &str) -> ContextSummary {
        let window = self.window.read().await;
        
        let ctx_summary = ContextSummary {
            id: uuid::Uuid::new_v4().to_string(),
            summary: summary.to_string(),
            original_entries: window.entries.len(),
            original_tokens: window.total_tokens,
            created_at: Utc::now(),
        };
        
        let mut summaries = self.summaries.write().await;
        summaries.push(ctx_summary.clone());
        
        ctx_summary
    }
    
    pub async fn get_summaries(&self) -> Vec<ContextSummary> {
        let summaries = self.summaries.read().await;
        summaries.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStats {
    pub total_entries: usize,
    pub total_tokens: usize,
    pub max_tokens: usize,
    pub available_tokens: usize,
    pub utilization: f64,
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}
