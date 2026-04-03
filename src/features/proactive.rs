//! 主动建议模式 (PROACTIVE)
//! 
//! 这个模块实现了 AI 主动提供建议的功能，无需用户显式触发。

use crate::error::Result;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 主动建议类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProactiveSuggestionType {
    /// 代码改进建议
    CodeImprovement,
    
    /// 潜在错误检测
    PotentialError,
    
    /// 最佳实践建议
    BestPractice,
    
    /// 重构建议
    Refactoring,
    
    /// 文档建议
    Documentation,
    
    /// 测试建议
    Testing,
    
    /// 性能优化建议
    Performance,
}

/// 主动建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProactiveSuggestion {
    /// 建议类型
    pub suggestion_type: ProactiveSuggestionType,
    
    /// 建议标题
    pub title: String,
    
    /// 详细描述
    pub description: String,
    
    /// 关联文件路径
    pub file_path: Option<String>,
    
    /// 关联代码行
    pub line_number: Option<usize>,
    
    /// 优先级 (1-10, 10最高)
    pub priority: u8,
    
    /// 建议创建时间
    pub created_at: String,
    
    /// 是否被用户忽略
    pub ignored: bool,
    
    /// 建议的解决方案
    pub solution: Option<String>,
}

/// 主动建议管理器
#[derive(Debug)]
pub struct ProactiveManager {
    /// 应用状态
    state: AppState,
    
    /// 待处理的建议
    pending_suggestions: Vec<ProactiveSuggestion>,
    
    /// 已显示的建议
    shown_suggestions: Vec<ProactiveSuggestion>,
    
    /// 建议触发阈值
    threshold: u8,
    
    /// 是否启用自动触发
    auto_trigger: bool,
    
    /// 检查间隔
    check_interval: Duration,
}

impl ProactiveManager {
    /// 创建新的主动建议管理器
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            pending_suggestions: Vec::new(),
            shown_suggestions: Vec::new(),
            threshold: 5,
            auto_trigger: true,
            check_interval: Duration::from_secs(60),
        }
    }
    
    /// 设置优先级阈值
    pub fn set_threshold(&mut self, threshold: u8) {
        self.threshold = threshold.clamp(1, 10);
    }
    
    /// 启用/禁用自动触发
    pub fn set_auto_trigger(&mut self, enabled: bool) {
        self.auto_trigger = enabled;
    }
    
    /// 添加建议
    pub fn add_suggestion(&mut self, suggestion: ProactiveSuggestion) {
        if suggestion.priority >= self.threshold {
            self.pending_suggestions.push(suggestion);
        }
    }
    
    /// 批量添加建议
    pub fn add_suggestions(&mut self, suggestions: Vec<ProactiveSuggestion>) {
        for suggestion in suggestions {
            self.add_suggestion(suggestion);
        }
    }
    
    /// 获取待处理的建议
    pub fn pending_suggestions(&self) -> &[ProactiveSuggestion] {
        &self.pending_suggestions
    }
    
    /// 获取优先级最高的建议
    pub fn top_suggestion(&self) -> Option<&ProactiveSuggestion> {
        self.pending_suggestions
            .iter()
            .max_by_key(|s| s.priority)
    }
    
    /// 标记建议为已显示
    pub fn mark_as_shown(&mut self, index: usize) -> Option<ProactiveSuggestion> {
        if index < self.pending_suggestions.len() {
            let suggestion = self.pending_suggestions.remove(index);
            self.shown_suggestions.push(suggestion.clone());
            Some(suggestion)
        } else {
            None
        }
    }
    
    /// 忽略建议
    pub fn ignore_suggestion(&mut self, index: usize) {
        if let Some(suggestion) = self.pending_suggestions.get_mut(index) {
            suggestion.ignored = true;
        }
    }
    
    /// 清空所有待处理建议
    pub fn clear_pending(&mut self) {
        self.pending_suggestions.clear();
    }
    
    /// 按类型过滤建议
    pub fn filter_by_type(&self, suggestion_type: ProactiveSuggestionType) -> Vec<&ProactiveSuggestion> {
        self.pending_suggestions
            .iter()
            .filter(|s| s.suggestion_type == suggestion_type)
            .collect()
    }
    
    /// 按优先级排序
    pub fn sort_by_priority(&mut self) {
        self.pending_suggestions.sort_by(|a, b| b.priority.cmp(&a.priority));
    }
    
    /// 创建代码改进建议
    pub fn create_code_improvement(
        title: String,
        description: String,
        file_path: String,
        solution: String,
    ) -> ProactiveSuggestion {
        ProactiveSuggestion {
            suggestion_type: ProactiveSuggestionType::CodeImprovement,
            title,
            description,
            file_path: Some(file_path),
            line_number: None,
            priority: 7,
            created_at: chrono::Utc::now().to_rfc3339(),
            ignored: false,
            solution: Some(solution),
        }
    }
    
    /// 创建潜在错误建议
    pub fn create_potential_error(
        title: String,
        description: String,
        file_path: String,
        line_number: usize,
        priority: u8,
    ) -> ProactiveSuggestion {
        ProactiveSuggestion {
            suggestion_type: ProactiveSuggestionType::PotentialError,
            title,
            description,
            file_path: Some(file_path),
            line_number: Some(line_number),
            priority: priority.clamp(1, 10),
            created_at: chrono::Utc::now().to_rfc3339(),
            ignored: false,
            solution: None,
        }
    }
}

/// 运行主动建议检查
pub async fn run_proactive_check(_manager: &mut ProactiveManager) -> Result<()> {
    // TODO: 实现代码分析和建议生成
    // 这个函数应该：
    // 1. 扫描当前代码库
    // 2. 分析潜在问题
    // 3. 生成建议
    // 4. 添加到管理器
    
    Ok(())
}
