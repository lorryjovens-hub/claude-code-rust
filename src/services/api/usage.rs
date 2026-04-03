//! 使用统计
//! 
//! 这个模块实现了使用统计功能

use serde::{Deserialize, Serialize};

/// 使用统计
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UsageStats {
    /// 输入 token 数
    pub input_tokens: u64,
    
    /// 输出 token 数
    pub output_tokens: u64,
    
    /// 总 token 数
    pub total_tokens: u64,
    
    /// 请求数
    pub request_count: u64,
}

impl Default for UsageStats {
    fn default() -> Self {
        Self {
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            request_count: 0,
        }
    }
}

impl UsageStats {
    /// 创建新的使用统计
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 添加使用量
    pub fn add_usage(&mut self, input: u64, output: u64) {
        self.input_tokens += input;
        self.output_tokens += output;
        self.total_tokens += input + output;
        self.request_count += 1;
    }
    
    /// 重置统计
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    
    /// 获取总 token 数
    pub fn total_tokens(&self) -> u64 {
        self.total_tokens
    }
    
    /// 获取请求数
    pub fn request_count(&self) -> u64 {
        self.request_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_usage_stats() {
        let mut stats = UsageStats::new();
        
        stats.add_usage(100, 50);
        assert_eq!(stats.input_tokens, 100);
        assert_eq!(stats.output_tokens, 50);
        assert_eq!(stats.total_tokens, 150);
        assert_eq!(stats.request_count, 1);
        
        stats.add_usage(200, 100);
        assert_eq!(stats.input_tokens, 300);
        assert_eq!(stats.output_tokens, 150);
        assert_eq!(stats.total_tokens, 450);
        assert_eq!(stats.request_count, 2);
    }
    
    #[test]
    fn test_usage_stats_reset() {
        let mut stats = UsageStats::new();
        
        stats.add_usage(100, 50);
        stats.reset();
        
        assert_eq!(stats.input_tokens, 0);
        assert_eq!(stats.output_tokens, 0);
        assert_eq!(stats.total_tokens, 0);
        assert_eq!(stats.request_count, 0);
    }
}
