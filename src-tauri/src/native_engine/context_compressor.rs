/// Context Compression Module
/// 
/// Provides intelligent context management to prevent context window overflow
/// while preserving critical conversation information.
/// 
/// Strategy (multi-level, progressive):
/// - Level 1: Tool output truncation (lightweight, ~80% retention)
/// - Level 2: Conversation summarization via LLM (~50% reduction)
/// - Level 3: Selective message pruning (~70% reduction)
/// - Level 4: Aggressive summarization (~90% reduction)
///
/// The compressor triggers compression when context usage exceeds configurable thresholds,
/// using the most aggressive level needed to fit within the model's context window.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Trigger compression when token usage exceeds this ratio of context window
    pub compression_threshold: f64,
    /// Target token count after compression (ratio of context window)
    pub target_ratio: f64,
    /// Number of recent turns to always preserve in full
    pub preserve_recent_turns: usize,
    /// Always preserve system prompt
    pub preserve_system_prompt: bool,
    /// Maximum tool output size before truncation (bytes)
    pub max_tool_output_bytes: usize,
    /// Max message content length before summarization (chars)
    pub max_message_content_chars: usize,
    /// Summarization prompt template
    pub summary_prompt: String,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            compression_threshold: 0.75,
            target_ratio: 0.55,
            preserve_recent_turns: 3,
            preserve_system_prompt: true,
            max_tool_output_bytes: 32000,
            max_message_content_chars: 200000,
            summary_prompt: r#"Summarize the following conversation in a concise but informative manner.
Focus on:
1. Key decisions made
2. Files modified or created
3. Important code snippets or configurations
4. Current state and pending tasks
5. Any errors encountered and their resolution

Preserve specific technical details like file paths, function names, and error messages.
Output format: a clear, structured summary."#.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContextWindow {
    pub model_id: String,
    pub total_tokens: u64,
    pub reserved_for_completion: u64,
    pub context_window: u64,
}

impl ContextWindow {
    pub fn available_tokens(&self) -> u64 {
        self.context_window.saturating_sub(self.total_tokens + self.reserved_for_completion)
    }

    pub fn usage_ratio(&self) -> f64 {
        if self.context_window == 0 {
            0.0
        } else {
            (self.total_tokens as f64) / (self.context_window as f64)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionLevel {
    /// No compression needed
    None,
    /// Light: Truncate long tool outputs, remove empty messages
    Light,
    /// Medium: Summarize old turns, preserve recent turns
    Medium,
    /// Heavy: Aggressive summarization of entire conversation
    Heavy,
    /// Critical: Must reduce context immediately
    Critical,
}

/// Represents a conversation turn (user message + assistant response)
#[derive(Debug, Clone)]
pub struct ConversationTurn {
    pub turn_index: usize,
    pub user_message: Option<serde_json::Value>,
    pub assistant_messages: Vec<serde_json::Value>,
    pub is_summarized: bool,
    pub estimated_tokens: u64,
}

/// Context Compressor
pub struct ContextCompressor {
    config: CompressionConfig,
}

impl ContextCompressor {
    pub fn new(config: Option<CompressionConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
        }
    }

    /// Determine if and what level of compression is needed
    pub fn determine_compression_level(&self, context: &ContextWindow) -> CompressionLevel {
        let ratio = context.usage_ratio();

        if ratio >= 0.95 {
            CompressionLevel::Critical
        } else if ratio >= self.config.compression_threshold {
            if ratio >= 0.85 {
                CompressionLevel::Heavy
            } else {
                CompressionLevel::Medium
            }
        } else if ratio >= 0.6 {
            CompressionLevel::Light
        } else {
            CompressionLevel::None
        }
    }

    /// Estimate token count for a message (rough approximation: 4 chars ≈ 1 token)
    pub fn estimate_message_tokens(msg: &serde_json::Value) -> u64 {
        let content = msg.get("content")
            .and_then(|c| c.as_str())
            .unwrap_or("");
        // Rough estimate: ~4 chars per token for English
        let char_tokens = content.len() as u64 / 4;
        
        // Add overhead for JSON structure
        let json_overhead: u64 = 10;
        
        char_tokens + json_overhead
    }

    /// Estimate total tokens for a list of messages
    pub fn estimate_tokens(messages: &[serde_json::Value]) -> u64 {
        messages.iter().map(Self::estimate_message_tokens).sum()
    }

    /// Apply compression to messages based on the determined level
    pub async fn compress(
        &self,
        messages: Vec<serde_json::Value>,
        system_prompt: Option<String>,
        level: CompressionLevel,
        provider: Option<&crate::native_engine::provider_manager::ResolvedProvider>,
    ) -> (Vec<serde_json::Value>, CompressionMetadata) {
        match level {
            CompressionLevel::None => {
                let original_count = messages.len();
                (messages, CompressionMetadata {
                    was_compressed: false,
                    level: CompressionLevel::None,
                    original_count,
                    compressed_count: original_count,
                    token_reduction: 0,
                })
            }
            CompressionLevel::Light => self.compression_light(messages, system_prompt),
            CompressionLevel::Medium => {
                self.compression_medium(messages, system_prompt, provider).await
            }
            CompressionLevel::Heavy => {
                self.compression_heavy(messages, system_prompt, provider).await
            }
            CompressionLevel::Critical => {
                self.compression_critical(messages, system_prompt, provider).await
            }
        }
    }

    /// Level 1: Light compression - truncate tool outputs, remove empty messages
    fn compression_light(
        &self,
        messages: Vec<serde_json::Value>,
        system_prompt: Option<String>,
    ) -> (Vec<serde_json::Value>, CompressionMetadata) {
        let original_count = messages.len();
        let original_tokens = Self::estimate_tokens(&messages);
        let mut compressed = Vec::new();

        // Preserve system prompt if configured
        if self.config.preserve_system_prompt {
            if let Some(sys) = &system_prompt {
                compressed.push(serde_json::json!({
                    "role": "system",
                    "content": sys
                }));
            }
        }

        for msg in &messages {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("");
            
            // Skip empty assistant messages
            if role == "assistant" {
                let content = msg.get("content").and_then(|c| c.as_str()).unwrap_or("");
                if content.trim().is_empty() {
                    continue;
                }
            }

            // Truncate long tool outputs
            if role == "tool" || role == "function" {
                let content = msg.get("content").and_then(|c| c.as_str()).unwrap_or("");
                if content.len() > self.config.max_tool_output_bytes {
                    let safe_end = content.char_indices()
                        .take(self.config.max_tool_output_bytes)
                        .last()
                        .map(|(idx, _)| idx)
                        .unwrap_or(0);
                    let truncated = &content[..safe_end];
                    let mut compressed_msg = msg.clone();
                    if let Some(obj) = compressed_msg.as_object_mut() {
                        obj.insert("content".to_string(), serde_json::json!(
                            format!("{}... [truncated, {} bytes total]", truncated, content.len())
                        ));
                        obj.insert("_compressed".to_string(), serde_json::json!(true));
                    }
                    compressed.push(compressed_msg);
                } else {
                    compressed.push(msg.clone());
                }
            } else {
                // Truncate very long messages
                let content = msg.get("content").and_then(|c| c.as_str()).unwrap_or("");
                if content.len() > self.config.max_message_content_chars && role == "user" {
                    let safe_end = content.char_indices()
                        .take(self.config.max_message_content_chars)
                        .last()
                        .map(|(idx, _)| idx)
                        .unwrap_or(0);
                    let truncated = &content[..safe_end];
                    let mut compressed_msg = msg.clone();
                    if let Some(obj) = compressed_msg.as_object_mut() {
                        obj.insert("content".to_string(), serde_json::json!(
                            format!("{}... [truncated]", truncated)
                        ));
                        obj.insert("_compressed".to_string(), serde_json::json!(true));
                    }
                    compressed.push(compressed_msg);
                } else {
                    compressed.push(msg.clone());
                }
            }
        }

        let compressed_tokens = Self::estimate_tokens(&compressed);
        let token_reduction = original_tokens.saturating_sub(compressed_tokens);

        tracing::info!(
            module = "ContextCompressor",
            "Light compression: {} → {} messages, tokens: {} → {} (saved {})",
            original_count, compressed.len(), original_tokens, compressed_tokens, token_reduction
        );

        let compressed_count = compressed.len();
        let was_compressed = original_count != compressed_count || token_reduction > 0;

        (compressed, CompressionMetadata {
            was_compressed,
            level: CompressionLevel::Light,
            original_count,
            compressed_count,
            token_reduction,
        })
    }

    /// Level 2: Medium compression - summarize old turns, preserve recent turns
    async fn compression_medium(
        &self,
        messages: Vec<serde_json::Value>,
        system_prompt: Option<String>,
        provider: Option<&crate::native_engine::provider_manager::ResolvedProvider>,
    ) -> (Vec<serde_json::Value>, CompressionMetadata) {
        let original_count = messages.len();
        let original_tokens = Self::estimate_tokens(&messages);

        // Extract system prompt
        let (system_msgs, conversation_msgs): (Vec<_>, Vec<_>) = messages.iter()
            .partition(|m| m.get("role").and_then(|r| r.as_str()) == Some("system"));

        // Parse turns
        let turns = self.parse_turns(&conversation_msgs);
        let total_turns = turns.len();

        let mut compressed = Vec::new();

        // Preserve system prompt
        if self.config.preserve_system_prompt {
            if let Some(sys) = &system_prompt {
                compressed.push(serde_json::json!({
                    "role": "system",
                    "content": sys
                }));
            } else {
                for sys_msg in system_msgs {
                    compressed.push(sys_msg.clone());
                }
            }
        }

        // Calculate how many turns to summarize
        let preserve_count = self.config.preserve_recent_turns.min(total_turns);
        let summarize_count = total_turns.saturating_sub(preserve_count);

        if summarize_count > 0 {
            let turns_to_summarize: Vec<&ConversationTurn> = turns.iter().take(summarize_count).collect();
            let turns_to_preserve: Vec<&ConversationTurn> = turns.iter().skip(summarize_count).collect();

            // Generate summary of old turns
            let summary_text = self.generate_summary_text(&turns_to_summarize, provider).await;

            // Add summary as a single message
            compressed.push(serde_json::json!({
                "role": "assistant",
                "content": format!("[Conversation Summary]\n\n{}", summary_text),
                "_is_summary": true,
                "_summarized_turns": summarize_count,
            }));

            // Add preserved recent turns
            for turn in turns_to_preserve {
                if let Some(user_msg) = &turn.user_message {
                    compressed.push(user_msg.clone());
                }
                for assistant_msg in &turn.assistant_messages {
                    compressed.push(assistant_msg.clone());
                }
            }
        } else {
            // All turns are within preserve count, just add them
            for turn in turns.iter() {
                if let Some(user_msg) = &turn.user_message {
                    compressed.push(user_msg.clone());
                }
                for assistant_msg in &turn.assistant_messages {
                    compressed.push(assistant_msg.clone());
                }
            }
        }

        let compressed_tokens = Self::estimate_tokens(&compressed);
        let token_reduction = original_tokens.saturating_sub(compressed_tokens);

        tracing::info!(
            module = "ContextCompressor",
            "Medium compression: {} → {} messages ({} turns summarized), tokens: {} → {} (saved {})",
            original_count, compressed.len(), summarize_count, original_tokens, compressed_tokens, token_reduction
        );

        let compressed_count = compressed.len();

        (compressed, CompressionMetadata {
            was_compressed: summarize_count > 0,
            level: CompressionLevel::Medium,
            original_count,
            compressed_count,
            token_reduction,
        })
    }

    /// Level 3: Heavy compression - aggressive summarization
    async fn compression_heavy(
        &self,
        messages: Vec<serde_json::Value>,
        system_prompt: Option<String>,
        provider: Option<&crate::native_engine::provider_manager::ResolvedProvider>,
    ) -> (Vec<serde_json::Value>, CompressionMetadata) {
        let original_count = messages.len();
        let original_tokens = Self::estimate_tokens(&messages);

        let (system_msgs, conversation_msgs): (Vec<_>, Vec<_>) = messages.iter()
            .partition(|m| m.get("role").and_then(|r| r.as_str()) == Some("system"));

        let turns = self.parse_turns(&conversation_msgs);
        let total_turns = turns.len();

        let mut compressed = Vec::new();

        // Preserve system prompt
        if self.config.preserve_system_prompt {
            if let Some(sys) = &system_prompt {
                compressed.push(serde_json::json!({
                    "role": "system",
                    "content": sys
                }));
            } else {
                for sys_msg in system_msgs {
                    compressed.push(sys_msg.clone());
                }
            }
        }

        // Summarize most turns, preserve only the last 1-2
        let preserve_count = std::cmp::min(2, total_turns);
        let summarize_count = total_turns.saturating_sub(preserve_count);

        if summarize_count > 0 {
            let turns_to_summarize: Vec<&ConversationTurn> = turns.iter().take(summarize_count).collect();
            let turns_to_preserve: Vec<&ConversationTurn> = turns.iter().skip(summarize_count).collect();

            let summary_text = self.generate_summary_text(&turns_to_summarize, provider).await;

            compressed.push(serde_json::json!({
                "role": "assistant",
                "content": format!("[Compressed Context]\n\nThe following is a summary of {} conversation turns:\n\n{}", summarize_count, summary_text),
                "_is_summary": true,
                "_summarized_turns": summarize_count,
            }));

            for turn in turns_to_preserve {
                if let Some(user_msg) = &turn.user_message {
                    compressed.push(user_msg.clone());
                }
                for assistant_msg in &turn.assistant_messages {
                    compressed.push(assistant_msg.clone());
                }
            }
        } else {
            for turn in turns.iter() {
                if let Some(user_msg) = &turn.user_message {
                    compressed.push(user_msg.clone());
                }
                for assistant_msg in &turn.assistant_messages {
                    compressed.push(assistant_msg.clone());
                }
            }
        }

        let compressed_tokens = Self::estimate_tokens(&compressed);
        let token_reduction = original_tokens.saturating_sub(compressed_tokens);

        let compressed_count = compressed.len();

        tracing::info!(
            module = "ContextCompressor",
            "Heavy compression: {} → {} messages ({} turns summarized), tokens: {} → {} (saved {})",
            original_count, compressed_count, summarize_count, original_tokens, compressed_tokens, token_reduction
        );

        (compressed, CompressionMetadata {
            was_compressed: summarize_count > 0,
            level: CompressionLevel::Heavy,
            original_count,
            compressed_count,
            token_reduction,
        })
    }

    /// Level 4: Critical compression - maximum reduction
    async fn compression_critical(
        &self,
        messages: Vec<serde_json::Value>,
        system_prompt: Option<String>,
        provider: Option<&crate::native_engine::provider_manager::ResolvedProvider>,
    ) -> (Vec<serde_json::Value>, CompressionMetadata) {
        let original_count = messages.len();
        let original_tokens = Self::estimate_tokens(&messages);

        let (system_msgs, conversation_msgs): (Vec<_>, Vec<_>) = messages.iter()
            .partition(|m| m.get("role").and_then(|r| r.as_str()) == Some("system"));

        let turns = self.parse_turns(&conversation_msgs);
        let total_turns = turns.len();

        let mut compressed = Vec::new();

        // Preserve only the most essential system prompt
        if self.config.preserve_system_prompt {
            if let Some(sys) = &system_prompt {
                // Truncate system prompt if too long
                let sys_content = if sys.len() > 2000 {
                    format!("{}... [system prompt truncated]", &sys[..1000])
                } else {
                    sys.clone()
                };
                compressed.push(serde_json::json!({
                    "role": "system",
                    "content": sys_content
                }));
            } else {
                for sys_msg in system_msgs {
                    compressed.push(sys_msg.clone());
                }
            }
        }

        // Generate comprehensive summary of ALL turns except the very last one
        let turns_to_summarize: Vec<&ConversationTurn> = if total_turns > 1 {
            turns.iter().take(total_turns - 1).collect()
        } else {
            turns.iter().collect()
        };

        let summary_text = self.generate_summary_text(&turns_to_summarize, provider).await;

        compressed.push(serde_json::json!({
            "role": "assistant",
            "content": format!("[CRITICAL: Context Compressed]\n\nFull conversation summary ({} turns):\n\n{}", total_turns, summary_text),
            "_is_summary": true,
            "_summarized_turns": total_turns,
            "_critical_compression": true,
        }));

        // Only keep the absolute last turn
        if let Some(last_turn) = turns.last() {
            if let Some(user_msg) = &last_turn.user_message {
                compressed.push(user_msg.clone());
            }
        }

        let compressed_tokens = Self::estimate_tokens(&compressed);
        let token_reduction = original_tokens.saturating_sub(compressed_tokens);

        let compressed_count = compressed.len();

        tracing::info!(
            module = "ContextCompressor",
            "CRITICAL compression: {} → {} messages ({} turns summarized), tokens: {} → {} (saved {})",
            original_count, compressed_count, total_turns, original_tokens, compressed_tokens, token_reduction
        );

        (compressed, CompressionMetadata {
            was_compressed: true,
            level: CompressionLevel::Critical,
            original_count,
            compressed_count,
            token_reduction,
        })
    }

    /// Parse messages into conversation turns
    fn parse_turns(&self, messages: &[&serde_json::Value]) -> Vec<ConversationTurn> {
        let mut turns = Vec::new();
        let mut current_turn: Option<ConversationTurn> = None;
        let mut turn_index = 0;

        for msg in messages {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("");

            if role == "user" {
                // Start a new turn
                if let Some(turn) = current_turn.take() {
                    turns.push(turn);
                }
                current_turn = Some(ConversationTurn {
                    turn_index,
                    user_message: Some((*msg).clone()),
                    assistant_messages: Vec::new(),
                    is_summarized: false,
                    estimated_tokens: Self::estimate_message_tokens(msg),
                });
                turn_index += 1;
            } else if role == "assistant" || role == "tool" || role == "function" {
                if let Some(ref mut turn) = current_turn {
                    turn.assistant_messages.push((*msg).clone());
                    turn.estimated_tokens += Self::estimate_message_tokens(msg);
                }
            }
        }

        if let Some(turn) = current_turn {
            turns.push(turn);
        }

        turns
    }

    /// Generate summary text for a set of turns
    async fn generate_summary_text(
        &self,
        turns: &[&ConversationTurn],
        _provider: Option<&crate::native_engine::provider_manager::ResolvedProvider>,
    ) -> String {
        if turns.is_empty() {
            return "No conversation history to summarize.".to_string();
        }

        // Build a text representation of the turns
        let mut summary_parts = Vec::new();

        for (i, turn) in turns.iter().enumerate() {
            if let Some(user_msg) = &turn.user_message {
                let content = user_msg.get("content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("");
                
                // Truncate long user messages for summary
                let truncated = if content.len() > 500 {
                    format!("{}...", &content[..200])
                } else {
                    content.to_string()
                };
                
                summary_parts.push(format!("**Turn {}**: User asked: {}", i + 1, truncated));
            }

            // Extract key information from assistant messages
            for (_j, assistant_msg) in turn.assistant_messages.iter().enumerate() {
                let role = assistant_msg.get("role").and_then(|r| r.as_str()).unwrap_or("");
                let content = assistant_msg.get("content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("");

                if role == "tool" || role == "function" {
                    // Extract tool call summary
                    if let Some(tool_name) = assistant_msg.get("name").and_then(|n| n.as_str()) {
                        // Check if there's a tool use in previous messages
                        summary_parts.push(format!("  - Used tool: {}", tool_name));
                    }
                } else if !content.trim().is_empty() {
                    let truncated = if content.len() > 300 {
                        format!("{}...", &content[..150])
                    } else {
                        content.to_string()
                    };
                    summary_parts.push(format!("  - Assistant: {}", truncated));
                }
            }
        }

        // Build a structured summary
        let mut summary = String::new();
        summary.push_str(&format!("Conversation Summary ({} turns):\n\n", turns.len()));
        
        // Group by topic/action
        let mut tool_calls = Vec::new();
        let mut key_decisions = Vec::new();
        let mut user_queries = Vec::new();

        for part in &summary_parts {
            if part.contains("Used tool:") {
                tool_calls.push(part.clone());
            } else if part.contains("User asked:") {
                user_queries.push(part.clone());
            } else {
                key_decisions.push(part.clone());
            }
        }

        if !user_queries.is_empty() {
            summary.push_str("### User Queries\n");
            for q in &user_queries {
                summary.push_str(&format!("- {}\n", q));
            }
            summary.push('\n');
        }

        if !tool_calls.is_empty() {
            summary.push_str("### Tools Used\n");
            for t in &tool_calls {
                summary.push_str(&format!("- {}\n", t));
            }
            summary.push('\n');
        }

        if !key_decisions.is_empty() {
            summary.push_str("### Key Points\n");
            for d in &key_decisions {
                summary.push_str(&format!("- {}\n", d));
            }
            summary.push('\n');
        }

        // Current state inference
        if let Some(last_turn) = turns.last() {
            if let Some(user_msg) = &last_turn.user_message {
                let content = user_msg.get("content")
                    .and_then(|c| c.as_str())
                    .unwrap_or("");
                summary.push_str(&format!("**Latest request**: {}\n", content));
            }
        }

        summary
    }
}

/// Metadata about the compression operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionMetadata {
    pub was_compressed: bool,
    pub level: CompressionLevel,
    pub original_count: usize,
    pub compressed_count: usize,
    pub token_reduction: u64,
}

impl std::fmt::Display for CompressionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompressionLevel::None => write!(f, "None"),
            CompressionLevel::Light => write!(f, "Light"),
            CompressionLevel::Medium => write!(f, "Medium"),
            CompressionLevel::Heavy => write!(f, "Heavy"),
            CompressionLevel::Critical => write!(f, "Critical"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_level_determination() {
        let compressor = ContextCompressor::new(None);
        let config = CompressionConfig::default();

        // Below threshold - no compression
        let low_usage = ContextWindow {
            model_id: "test".to_string(),
            total_tokens: 50000,
            reserved_for_completion: 10000,
            context_window: 128000,
        };
        assert_eq!(compressor.determine_compression_level(&low_usage), CompressionLevel::None);

        // Light compression needed
        let medium_usage = ContextWindow {
            model_id: "test".to_string(),
            total_tokens: 90000,
            reserved_for_completion: 10000,
            context_window: 128000,
        };
        assert_eq!(compressor.determine_compression_level(&medium_usage), CompressionLevel::Light);

        // Critical - near limit
        let high_usage = ContextWindow {
            model_id: "test".to_string(),
            total_tokens: 120000,
            reserved_for_completion: 10000,
            context_window: 128000,
        };
        assert_eq!(compressor.determine_compression_level(&high_usage), CompressionLevel::Critical);
    }

    #[test]
    fn test_token_estimation() {
        let msg = serde_json::json!({
            "role": "user",
            "content": "Hello, this is a test message with about 40 characters."
        });
        let tokens = ContextCompressor::estimate_message_tokens(&msg);
        assert!(tokens > 0);
    }

    #[test]
    fn test_turn_parsing() {
        let compressor = ContextCompressor::new(None);
        let messages = vec![
            serde_json::json!({"role": "user", "content": "Question 1"}),
            serde_json::json!({"role": "assistant", "content": "Answer 1"}),
            serde_json::json!({"role": "user", "content": "Question 2"}),
            serde_json::json!({"role": "assistant", "content": "Answer 2"}),
        ];
        let refs: Vec<&serde_json::Value> = messages.iter().collect();
        let turns = compressor.parse_turns(&refs);
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].turn_index, 0);
        assert_eq!(turns[1].turn_index, 1);
    }
}
