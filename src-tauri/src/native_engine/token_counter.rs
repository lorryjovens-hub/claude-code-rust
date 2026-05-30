use serde_json::Value;
use std::collections::HashMap;

pub struct TokenCounter;

impl TokenCounter {
    /// Simple estimation: English ~1 token/4 chars, Chinese ~1 token/1.5 chars
    /// Overall average: ~1 token/3 chars
    pub fn estimate(messages: &[Value]) -> usize {
        let total_chars: usize = messages
            .iter()
            .map(|m| {
                match &m["content"] {
                    Value::String(s) => s.len(),
                    Value::Array(parts) => parts
                        .iter()
                        .filter_map(|p| p.get("text").and_then(Value::as_str))
                        .map(|s| s.len())
                        .sum(),
                    _ => 0,
                }
            })
            .sum();
        
        // Rough average: 1 token ≈ 3 chars
        total_chars / 3
    }

    /// Estimate tokens for a single message
    pub fn estimate_message(msg: &Value) -> usize {
        let chars = match &msg["content"] {
            Value::String(s) => s.len(),
            Value::Array(parts) => parts
                .iter()
                .filter_map(|p| p.get("text").and_then(Value::as_str))
                .map(|s| s.len())
                .sum(),
            _ => 0,
        };
        chars / 3
    }

    /// Check if token count exceeds threshold
    pub fn exceeds_threshold(messages: &[Value], threshold: usize) -> bool {
        Self::estimate(messages) > threshold
    }

    /// Get maximum context window size for common models
    pub fn get_model_context_limit(model_id: &str) -> usize {
        let mut limits = HashMap::new();
        
        // OpenAI models
        limits.insert("gpt-4o", 128000);
        limits.insert("gpt-4o-2024-08-06", 128000);
        limits.insert("gpt-4o-2024-11-20", 128000);
        limits.insert("gpt-4o-mini", 128000);
        limits.insert("gpt-4-turbo", 128000);
        limits.insert("gpt-4-turbo-2024-04-09", 128000);
        limits.insert("gpt-4", 8192);
        limits.insert("gpt-4-32k", 32768);
        limits.insert("gpt-3.5-turbo", 16384);
        limits.insert("gpt-3.5-turbo-16k", 16384);
        
        // Anthropic models
        limits.insert("claude-3-5-sonnet-20241022", 200000);
        limits.insert("claude-3-opus-20240229", 200000);
        limits.insert("claude-3-sonnet-20240229", 200000);
        limits.insert("claude-3-haiku-20240307", 200000);
        limits.insert("claude-2.1", 200000);
        limits.insert("claude-2", 100000);
        
        // DeepSeek models
        limits.insert("deepseek-chat", 128000);
        limits.insert("deepseek-coder", 128000);
        
        // Default fallback for unknown models - use a conservative limit
        let default_limit = 80000;
        
        // Check if model_id contains any of the known patterns
        for (pattern, &limit) in &limits {
            if model_id.contains(pattern) {
                return limit;
            }
        }
        
        default_limit
    }

    /// Check if messages exceed a model's context limit, with safety margin
    pub fn exceeds_model_limit(messages: &[Value], model_id: &str, safety_margin: f64) -> bool {
        let model_limit = Self::get_model_context_limit(model_id);
        let estimated_tokens = Self::estimate(messages);
        let safe_limit = (model_limit as f64 * safety_margin) as usize;
        
        estimated_tokens > safe_limit
    }

    /// Calculate safe token limit for a model (with safety margin)
    pub fn get_safe_limit(model_id: &str, safety_margin: f64) -> usize {
        let model_limit = Self::get_model_context_limit(model_id);
        (model_limit as f64 * safety_margin) as usize
    }
}
