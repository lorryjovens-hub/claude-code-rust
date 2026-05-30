//! MemEx - Infinite Memory Engine for Claude Desktop (Tauri)
//!
//! Provides: semantic search, auto-ingest, context compression
//! Backend: Python MemEx HTTP API (http://127.0.0.1:8765)
//!
//! Also includes Caveman + RTK (Recursive Token Keeping) compression
//! and RLM (Reinforcement Learning from Memory) system

mod caveman_rtk;
pub use caveman_rtk::{CavemanRTKStats, CavemanRTKSystem, RTKConfig, TokenImportance};

use anyhow::{anyhow, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Data Types
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: String,
    pub content: String,
    pub importance: f64,
    pub created_at: String,
    pub metadata: Option<HashMap<String, String>>,
    pub similarity_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchRequest {
    pub query: String,
    pub top_k: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResponse {
    pub results: Vec<MemoryItem>,
    pub total_indexed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryIngestRequest {
    pub content: String,
    pub importance: Option<f64>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_memories: usize,
    pub total_tokens_approx: usize,
    pub backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub enabled: bool,
    pub backend_url: String,
    pub auto_ingest: bool,
    pub auto_search: bool,
    pub search_top_k: usize,
    pub compression_threshold_tokens: usize,
    pub min_importance_threshold: f64,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend_url: "http://127.0.0.1:8765".to_string(),
            auto_ingest: true,
            auto_search: true,
            search_top_k: 5,
            compression_threshold_tokens: 2_500_000,
            min_importance_threshold: 0.3,
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// MemEx HTTP Client
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Clone)]
pub struct MemExClient {
    client: reqwest::Client,
    pub base_url: String,
    config: Arc<RwLock<MemoryConfig>>,
}

impl MemExClient {
    pub fn new(base_url: Option<String>) -> Self {
        let config = MemoryConfig {
            backend_url: base_url.unwrap_or_else(|| "http://127.0.0.1:8765".to_string()),
            ..Default::default()
        };
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            base_url: config.backend_url.clone(),
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub async fn get_config(&self) -> MemoryConfig {
        self.config.read().await.clone()
    }

    pub fn get_config_blocking(&self) -> MemoryConfig {
        match self.config.try_read() {
            Ok(guard) => guard.clone(),
            Err(_) => MemoryConfig::default(),
        }
    }

    pub async fn update_config(&self, config: MemoryConfig) {
        *self.config.write().await = config;
    }

    pub async fn health_check(&self) -> Result<bool> {
        match self.client.get(format!("{}/health", self.base_url)).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    pub async fn search(&self, query: &str, top_k: Option<usize>) -> Result<Vec<MemoryItem>> {
        let config = self.config.read().await;
        if !config.enabled {
            return Ok(vec![]);
        }
        let req = MemorySearchRequest {
            query: query.to_string(),
            top_k: Some(top_k.unwrap_or(config.search_top_k)),
        };
        match self.client.post(format!("{}/search", self.base_url)).json(&req).send().await {
            Ok(resp) if resp.status().is_success() => {
                let result: MemorySearchResponse = resp.json().await?;
                debug!("[MemEx] Search: {} results", result.results.len());
                Ok(result.results)
            }
            Ok(resp) => { warn!("[MemEx] Search failed: {}", resp.status()); Ok(vec![]) }
            Err(e) => { warn!("[MemEx] Backend unreachable: {}", e); Ok(vec![]) }
        }
    }

    pub async fn ingest(&self, content: &str, importance: Option<f64>, metadata: Option<HashMap<String, String>>) -> Result<()> {
        let config = self.config.read().await;
        if !config.enabled { return Ok(()); }
        let importance = importance.unwrap_or_else(|| Self::estimate_importance(content));
        if importance < config.min_importance_threshold {
            debug!("[MemEx] Skip low-importance ({:.2})", importance);
            return Ok(());
        }
        let req = MemoryIngestRequest { content: content.to_string(), importance: Some(importance), metadata };
        match self.client.post(format!("{}/ingest", self.base_url)).json(&req).send().await {
            Ok(resp) if resp.status().is_success() => {
                debug!("[MemEx] Ingested (importance: {:.2})", importance);
            }
            Ok(resp) => warn!("[MemEx] Ingest failed: {}", resp.status()),
            Err(e) => warn!("[MemEx] Backend unreachable: {}", e),
        }
        Ok(())
    }

    pub async fn stats(&self) -> Result<MemoryStats> {
        match self.client.get(format!("{}/stats", self.base_url)).send().await {
            Ok(resp) if resp.status().is_success() => Ok(resp.json().await?),
            Ok(resp) => Err(anyhow!("Backend: {}", resp.status())),
            Err(e) => Err(anyhow!("Unreachable: {}", e)),
        }
    }

    pub async fn clear(&self) -> Result<()> {
        self.client.post(format!("{}/clear", self.base_url)).send().await?;
        Ok(())
    }

    fn estimate_importance(content: &str) -> f64 {
        let content_lower = content.to_lowercase();
        let mut score: f64 = 0.3;
        let high_signals = ["important", "critical", "key", "remember", "password", "token", "api key", "secret", "config", "必须记住", "重要", "关键", "密码"];
        for s in &high_signals { if content_lower.contains(s) { score += 0.15; } }
        let medium_signals = ["architecture", "design", "decision", "pattern", "workflow", "架构", "设计", "决策", "模式"];
        for s in &medium_signals { if content_lower.contains(s) { score += 0.08; } }
        if content.len() > 200 { score += 0.05; }
        if content.len() > 500 { score += 0.05; }
        score.min(1.0_f64)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Tiered Compressor
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Debug, Clone)]
pub enum MemoryTier {
    Hot,
    Warm,
    Cold,
}

#[derive(Debug, Clone)]
pub struct TieredMessage {
    pub role: String,
    pub content: String,
    pub tier: MemoryTier,
    pub summary: Option<String>,
    pub keywords: Vec<String>,
    pub tool_calls_count: usize,
    pub is_subtask_boundary: bool,
}

pub struct TieredCompressor {
    hot_threshold: usize,
    warm_threshold: usize,
}

impl TieredCompressor {
    pub fn new() -> Self {
        Self {
            hot_threshold: 10,
            warm_threshold: 30,
        }
    }

    pub fn with_thresholds(mut self, hot: usize, warm: usize) -> Self {
        self.hot_threshold = hot;
        self.warm_threshold = warm;
        self
    }

    pub fn classify_messages(&self, messages: &[Value]) -> Vec<TieredMessage> {
        let total_rounds = Self::count_rounds(messages);
        let boundaries = Self::detect_subtask_boundaries(messages);
        let boundary_set: HashSet<usize> = boundaries.into_iter().collect();

        let mut round_idx = 0usize;
        let mut result = Vec::with_capacity(messages.len());

        for (i, msg) in messages.iter().enumerate() {
            let role = msg["role"].as_str().unwrap_or("unknown").to_string();
            let content = Self::extract_text_content(msg);
            let tool_calls_count = Self::count_tool_uses_in_message(msg);
            let is_boundary = boundary_set.contains(&i);

            if role == "assistant" {
                round_idx += 1;
            }

            let rounds_from_end = total_rounds.saturating_sub(round_idx);
            let tier = if rounds_from_end < self.hot_threshold {
                MemoryTier::Hot
            } else if rounds_from_end < self.warm_threshold {
                MemoryTier::Warm
            } else {
                MemoryTier::Cold
            };

            result.push(TieredMessage {
                role,
                content,
                tier,
                summary: None,
                keywords: Vec::new(),
                tool_calls_count,
                is_subtask_boundary: is_boundary,
            });
        }

        result
    }

    fn count_rounds(messages: &[Value]) -> usize {
        messages.iter().filter(|m| m["role"].as_str() == Some("assistant")).count()
    }

    pub fn detect_subtask_boundaries(messages: &[Value]) -> Vec<usize> {
        let mut boundaries = Vec::new();
        let len = messages.len();
        if len < 3 {
            return boundaries;
        }

        for i in 0..len.saturating_sub(2) {
            let curr_role = messages[i]["role"].as_str().unwrap_or("");
            let next_role = messages[i + 1]["role"].as_str().unwrap_or("");
            let next2_role = messages[i + 2]["role"].as_str().unwrap_or("");

            if curr_role == "assistant" && Self::has_tool_use(&messages[i])
                && next_role == "user" && Self::has_tool_result(&messages[i + 1])
                && next2_role == "assistant" && !Self::has_tool_use(&messages[i + 2])
            {
                boundaries.push(i + 2);
            }
        }

        boundaries
    }

    fn has_tool_use(msg: &Value) -> bool {
        if let Some(arr) = msg["content"].as_array() {
            arr.iter().any(|b| b["type"].as_str() == Some("tool_use"))
        } else {
            false
        }
    }

    fn has_tool_result(msg: &Value) -> bool {
        if let Some(arr) = msg["content"].as_array() {
            arr.iter().any(|b| b["type"].as_str() == Some("tool_result"))
        } else {
            false
        }
    }

    fn extract_text_content(msg: &Value) -> String {
        if let Some(s) = msg["content"].as_str() {
            return s.to_string();
        }
        if let Some(arr) = msg["content"].as_array() {
            let parts: Vec<String> = arr.iter().filter_map(|b| {
                match b["type"].as_str() {
                    Some("text") => b["text"].as_str().map(String::from),
                    Some("tool_use") => Some(format!("[ToolCall: {}]",
                        b["name"].as_str().unwrap_or("?"))),
                    Some("tool_result") => Some(format!("[ToolResult: {}]",
                        b["content"].as_str().unwrap_or("").chars().take(200).collect::<String>())),
                    _ => None,
                }
            }).collect();
            return parts.join("\n");
        }
        String::new()
    }

    fn count_tool_uses_in_message(msg: &Value) -> usize {
        if let Some(arr) = msg["content"].as_array() {
            arr.iter().filter(|b| b["type"].as_str() == Some("tool_use")).count()
        } else {
            0
        }
    }

    pub fn generate_summary(messages: &[Value]) -> String {
        let boundaries = Self::detect_subtask_boundaries(messages);
        let mut subtask_ranges: Vec<(usize, usize)> = Vec::new();
        let mut start = 0;
        for &b in &boundaries {
            subtask_ranges.push((start, b));
            start = b;
        }
        if start < messages.len() {
            subtask_ranges.push((start, messages.len()));
        }

        if subtask_ranges.is_empty() {
            return ContextManager::simple_summarize(
                &messages.iter().map(|m| Self::extract_text_content(m)).collect::<Vec<_>>()
            );
        }

        let mut summary_parts = Vec::new();
        for (subtask_idx, (start, end)) in subtask_ranges.iter().enumerate() {
            let subtask_msgs = &messages[*start..*end];
            let mut tool_chain: Vec<String> = Vec::new();
            let mut result_hint = String::new();

            for msg in subtask_msgs {
                if let Some(arr) = msg["content"].as_array() {
                    for block in arr {
                        if block["type"].as_str() == Some("tool_use") {
                            if let Some(name) = block["name"].as_str() {
                                tool_chain.push(name.to_string());
                            }
                        }
                        if block["type"].as_str() == Some("tool_result") {
                            let content = block["content"].as_str().unwrap_or("");
                            if !content.is_empty() {
                                let hint: String = content.chars().take(80).collect();
                                result_hint = hint;
                            }
                        }
                    }
                }
            }

            let chain_str = if tool_chain.is_empty() {
                "no tools".to_string()
            } else {
                tool_chain.join("→")
            };

            let result_str = if result_hint.is_empty() {
                String::new()
            } else {
                format!(": {}", result_hint)
            };

            summary_parts.push(format!("[Subtask {}] {}{}", subtask_idx + 1, chain_str, result_str));
        }

        summary_parts.join("\n")
    }

    pub async fn generate_summary_async(messages: &[Value], memex: &MemExClient) -> String {
        let sync_summary = Self::generate_summary(messages);
        let combined_text: String = messages.iter()
            .map(|m| Self::extract_text_content(m))
            .collect::<Vec<_>>()
            .join("\n");

        if combined_text.len() < 200 {
            return sync_summary;
        }

        let prompt = format!(
            "Summarize the following conversation turns concisely, preserving key decisions, tool usage chains, and outcomes:\n\n{}",
            combined_text.chars().take(4000).collect::<String>()
        );

        match memex.search(&prompt, Some(1)).await {
            Ok(results) if !results.is_empty() => {
                if results[0].similarity_score.unwrap_or(0.0) > 0.85 {
                    format!("{}\n[LLM-enhanced] {}", sync_summary, &results[0].content.chars().take(500).collect::<String>())
                } else {
                    sync_summary
                }
            }
            _ => sync_summary,
        }
    }

    pub fn extract_keywords(messages: &[Value]) -> Vec<String> {
        let combined: String = messages.iter()
            .map(|m| Self::extract_text_content(m))
            .collect::<Vec<_>>()
            .join(" ");

        let mut keywords = Vec::new();

        if let Ok(file_pattern) = Regex::new(r"[\w./-]+\.[\w]+") {
            for cap in file_pattern.find_iter(&combined) {
                let f = cap.as_str().to_string();
                if f.len() > 3 && f.len() < 80 && !keywords.contains(&f) {
                    keywords.push(f);
                }
            }
        }

        let decision_signals = [
            "decided", "decision", "chosen", "selected", "concluded",
            "resolved", "fixed", "implemented", "refactored", "migrated",
            "决定", "选择", "结论", "解决", "修复", "实现", "重构", "迁移",
        ];
        let combined_lower = combined.to_lowercase();
        for signal in &decision_signals {
            if combined_lower.contains(signal) && !keywords.contains(&signal.to_string()) {
                keywords.push(signal.to_string());
            }
        }

        if let Ok(ident_pattern) = Regex::new(r"\b[A-Z][a-zA-Z]+(?:Trait|Impl|Struct|Enum|Class|Module|Component|Service|Manager|Handler)\b") {
            for cap in ident_pattern.find_iter(&combined) {
                let ident = cap.as_str().to_string();
                if !keywords.contains(&ident) {
                    keywords.push(ident);
                }
            }
        }

        if let Ok(fn_pattern) = Regex::new(r"\b(?:fn|def|function|async fn)\s+(\w+)") {
            for cap in fn_pattern.captures_iter(&combined) {
                if let Some(m) = cap.get(1) {
                    let name = m.as_str().to_string();
                    if !keywords.contains(&name) {
                        keywords.push(name);
                    }
                }
            }
        }

        keywords.truncate(30);
        keywords
    }

    pub fn compress(&self, messages: &[Value]) -> Vec<Value> {
        let total_rounds = Self::count_rounds(messages);
        if total_rounds <= self.hot_threshold {
            return messages.to_vec();
        }

        let classified = self.classify_messages(messages);

        let mut cold_end = 0usize;
        let mut warm_end = 0usize;
        {
            let mut round_idx = 0usize;
            for (i, tm) in classified.iter().enumerate() {
                if tm.role == "assistant" {
                    round_idx += 1;
                }
                let rounds_from_end = total_rounds.saturating_sub(round_idx);
                if rounds_from_end >= self.warm_threshold && cold_end == 0 {
                    cold_end = i + 1;
                }
                if rounds_from_end >= self.hot_threshold && warm_end == 0 {
                    warm_end = i + 1;
                }
            }
        }

        let mut result = Vec::new();

        if cold_end > 0 {
            let cold_msgs = &messages[..cold_end];
            let keywords = Self::extract_keywords(cold_msgs);
            let kw_str = keywords.join(", ");
            result.push(json!({
                "role": "system",
                "content": format!("[Earlier context keywords: {}] Use memory_search to retrieve details", kw_str)
            }));
        }

        if warm_end > cold_end {
            let warm_msgs = &messages[cold_end..warm_end];
            let summary = Self::generate_summary(warm_msgs);
            result.push(json!({
                "role": "system",
                "content": format!("[Summary of earlier subtask] {}", summary)
            }));
        }

        if warm_end < messages.len() {
            for msg in &messages[warm_end..] {
                result.push(msg.clone());
            }
        }

        if result.is_empty() {
            return messages.to_vec();
        }

        result
    }

    /// Aggressive compression for emergency situations when even standard compression is not enough
    /// Keeps only the most recent messages and compresses everything else heavily
    pub fn compress_aggressive(&self, messages: &[Value], keep_recent: usize) -> Vec<Value> {
        let total_len = messages.len();
        if total_len <= keep_recent {
            return messages.to_vec();
        }

        let mut result = Vec::new();

        // Add a super compact summary of everything before the recent messages
        let old_msgs = &messages[0..total_len - keep_recent];
        let keywords = Self::extract_keywords(old_msgs);
        let kw_str = if keywords.len() > 15 {
            keywords[0..15].join(", ")
        } else {
            keywords.join(", ")
        };

        result.push(json!({
            "role": "system",
            "content": format!("[Compressed context] Key topics: {}. Use memory_search for details.", kw_str)
        }));

        // Add the most recent messages
        for msg in &messages[total_len - keep_recent..] {
            result.push(msg.clone());
        }

        result
    }

    /// Progressive compression that applies multiple compression levels until within limits
    pub fn compress_progressive(
        &self,
        messages: &[Value],
        target_token_limit: usize,
        token_estimator: impl Fn(&[Value]) -> usize,
    ) -> Vec<Value> {
        let mut compressed = messages.to_vec();

        // First try standard compression
        let mut estimated = token_estimator(&compressed);
        if estimated <= target_token_limit {
            return compressed;
        }

        compressed = self.compress(&compressed);
        estimated = token_estimator(&compressed);
        if estimated <= target_token_limit {
            return compressed;
        }

        // Then try more aggressive compression with varying keep counts
        let keep_counts = [20, 15, 10, 8, 5];
        for &keep in &keep_counts {
            compressed = self.compress_aggressive(messages, keep);
            estimated = token_estimator(&compressed);
            if estimated <= target_token_limit {
                return compressed;
            }
        }

        // Last resort: keep only the absolute minimum (last 3 messages)
        self.compress_aggressive(messages, 3)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Context Manager
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[derive(Clone)]
pub struct ContextManager {
    memex: Arc<MemExClient>,
    caveman_rtk: Arc<Mutex<CavemanRTKSystem>>,
}

impl ContextManager {
    pub fn new(memex: Arc<MemExClient>) -> Self {
        Self {
            memex,
            caveman_rtk: Arc::new(Mutex::new(CavemanRTKSystem::new(None))),
        }
    }

    pub fn memex(&self) -> &MemExClient { &self.memex }

    /// Get Caveman RTK stats for dashboard
    pub async fn get_caveman_stats(&self) -> CavemanRTKStats {
        let system = self.caveman_rtk.lock().await;
        system.get_stats()
    }

    /// Add content to Caveman memory
    pub async fn add_to_caveman_memory(&self, content: &str, role: &str) -> Result<String> {
        let mut system = self.caveman_rtk.lock().await;
        system.add_memory_segment(content, role)
    }

    /// Get relevant context from Caveman
    pub async fn get_caveman_context(&self, query: &str, max_segments: usize) -> String {
        let mut system = self.caveman_rtk.lock().await;
        system.get_context_string(query, max_segments)
    }

    /// Record RLM feedback
    pub async fn record_rlm_feedback(&self, memory_id: &str, was_useful: bool, context: &str) {
        let mut system = self.caveman_rtk.lock().await;
        system.record_rlm_feedback(memory_id, was_useful, context);
    }

    /// Run RLM iteration
    pub async fn run_rlm_iteration(&self) -> u32 {
        let mut system = self.caveman_rtk.lock().await;
        system.rlm_iterate()
    }

    /// Called BEFORE API call: inject relevant memories into context
    pub async fn before_api_call(&self, conversation_id: &str, user_message: &str) -> Option<String> {
        let config = self.memex.get_config().await;
        if !config.enabled || !config.auto_search { return None; }

        let mut all_results = Vec::new();

        match self.memex.search(user_message, Some(config.search_top_k)).await {
            Ok(results) => all_results.extend(results),
            _ => {}
        }

        let cold_keywords = Self::extract_cold_keywords_from_message(user_message);
        for keyword in &cold_keywords {
            if let Ok(results) = self.memex.search(keyword, Some(3)).await {
                for r in results {
                    if !all_results.iter().any(|existing| existing.id == r.id) {
                        all_results.push(r);
                    }
                }
            }
        }

        if all_results.is_empty() {
            return None;
        }

        let memory_context: String = all_results.iter().enumerate()
            .map(|(i, m)| format!("[Memory {}] (importance: {:.2}) {}\n", i + 1, m.importance, m.content))
            .collect();
        let injected = format!(
            "<relevant_memories>\nThe following information from previous conversations may be relevant:\n\n{}</relevant_memories>\n\nUse this context if relevant.",
            memory_context
        );
        info!("[MemEx] Injected {} memories into conv {} (including cold memory retrieval)", all_results.len(), conversation_id);
        Some(injected)
    }

    fn extract_cold_keywords_from_message(message: &str) -> Vec<String> {
        let mut keywords = Vec::new();

        if let Ok(file_pattern) = Regex::new(r"[\w./-]+\.[\w]+") {
            for cap in file_pattern.find_iter(message) {
                let f = cap.as_str().to_string();
                if f.len() > 3 && !keywords.contains(&f) {
                    keywords.push(f);
                }
            }
        }

        let reference_signals = [
            "earlier", "before", "previous", "earlier context",
            "之前", "之前提到", "上文", "刚才",
        ];
        let msg_lower = message.to_lowercase();
        for signal in &reference_signals {
            if msg_lower.contains(signal) {
                keywords.push(signal.to_string());
                break;
            }
        }

        keywords.truncate(5);
        keywords
    }

    /// Called AFTER response: store conversation turn
    pub async fn after_response(&self, conversation_id: &str, user_message: &str, assistant_message: &str) {
        let config = self.memex.get_config().await;
        if !config.enabled || !config.auto_ingest { return; }
        let trunc = |s: &str| {
            if s.len() > 2000 {
                let safe_end = s.char_indices()
                    .take(2000)
                    .last()
                    .map(|(idx, ch)| idx + ch.len_utf8())
                    .unwrap_or(0);
                format!("{}...", &s[..safe_end])
            } else {
                s.to_string()
            }
        };
        let memory = format!("Conv {}:\nUser: {}\nAssistant: {}", conversation_id, trunc(user_message), trunc(assistant_message));
        let mut meta = HashMap::new();
        meta.insert("type".to_string(), "conversation".to_string());
        meta.insert("conversation_id".to_string(), conversation_id.to_string());
        let _ = self.memex.ingest(&memory, None, Some(meta)).await;
    }

    pub fn needs_compression(&self, message_count: usize, total_chars: usize) -> bool {
        let estimated_tokens = total_chars / 4;
        let config = self.memex.get_config_blocking();
        message_count > 50 || estimated_tokens > config.compression_threshold_tokens
    }

    pub async fn compress_context(&self, conversation_id: &str, old_messages: &[String]) -> Result<String> {
        let summary = Self::simple_summarize(old_messages);
        let mut meta = HashMap::new();
        meta.insert("type".to_string(), "context_compression".to_string());
        meta.insert("conversation_id".to_string(), conversation_id.to_string());
        meta.insert("compressed_count".to_string(), old_messages.len().to_string());
        let _ = self.memex.ingest(&summary, Some(0.9), Some(meta)).await;
        info!("[MemEx] Compressed {} messages for conv {}", old_messages.len(), conversation_id);
        Ok(summary)
    }

    fn simple_summarize(messages: &[String]) -> String {
        let combined = messages.join("\n");
        let key_signals = ["important", "key", "decision", "architecture", "design", "pattern", "bug", "fix", "error", "solution", "conclusion", "重要", "关键", "决定", "架构", "设计", "错误", "修复", "解决方案", "结论"];
        let mut key_sentences: Vec<&str> = Vec::new();
        for line in combined.lines() {
            let line_lower = line.to_lowercase();
            for signal in &key_signals {
                if line_lower.contains(signal) && !key_sentences.contains(&line) {
                    key_sentences.push(line);
                    break;
                }
            }
        }
        if key_sentences.is_empty() {
            let lines: Vec<&str> = combined.lines().collect();
            if lines.is_empty() {
                return "[Context Summary]\n(empty)".to_string();
            }
            let total = lines.len();
            let take = 10.min(total);
            let head_take = take.min(total.max(1) / 2).max(1);
            let tail_start = total.saturating_sub(take);
            // Avoid overlapping head and tail sections
            let tail_start = if tail_start <= head_take { total.min(head_take + take) } else { tail_start };
            let tail_start = tail_start.min(lines.len());
            let mut summary = String::from("[Context Summary]\n");
            for line in &lines[..head_take] { summary.push_str(line); summary.push('\n'); }
            if total > take * 2 && tail_start > head_take { summary.push_str("...\n"); }
            if tail_start < lines.len() {
                for line in &lines[tail_start..] { summary.push_str(line); summary.push('\n'); }
            }
            summary
        } else {
            format!("[Context Summary - {} key points]\n{}", key_sentences.len(), key_sentences.join("\n"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_importance() {
        let low = MemExClient::estimate_importance("hello world");
        let high = MemExClient::estimate_importance("IMPORTANT: The API key is sk-abc123. Remember this!");
        assert!(high > low);
    }

    #[test]
    fn test_summarize() {
        let msgs = vec!["The architecture uses microservices".to_string(), "Bug: memory leak in pool".to_string(), "Random chat".to_string()];
        let s = ContextManager::simple_summarize(&msgs);
        assert!(s.contains("architecture") || s.contains("Bug"));
    }
}
