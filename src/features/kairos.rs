//! 时间感知系统 (KAIROS)
//! 
//! 这个模块实现了基于时间的智能响应系统，根据时间、日期和用户模式提供
//! 个性化的建议和响应。

use crate::state::AppState;
use chrono::{DateTime, Datelike, Duration, Timelike, Utc, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// 时间段类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimePeriod {
    /// 清晨 (5:00-8:00)
    EarlyMorning,
    
    /// 早上 (8:00-12:00)
    Morning,
    
    /// 中午 (12:00-14:00)
    Noon,
    
    /// 下午 (14:00-18:00)
    Afternoon,
    
    /// 傍晚 (18:00-21:00)
    Evening,
    
    /// 晚上 (21:00-24:00)
    Night,
    
    /// 深夜 (0:00-5:00)
    LateNight,
}

/// 用户活动模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivityPattern {
    /// 工作日活动模式
    pub weekday_patterns: HashMap<Weekday, Vec<TimePeriod>>,
    
    /// 最活跃的时间段
    pub most_active_periods: Vec<TimePeriod>,
    
    /// 平均会话时长（分钟）
    pub average_session_duration: f64,
    
    /// 常用工具统计
    pub tool_usage: HashMap<String, u64>,
    
    /// 记录开始时间
    pub pattern_start_date: DateTime<Utc>,
}

impl Default for UserActivityPattern {
    fn default() -> Self {
        Self {
            weekday_patterns: HashMap::new(),
            most_active_periods: Vec::new(),
            average_session_duration: 0.0,
            tool_usage: HashMap::new(),
            pattern_start_date: Utc::now(),
        }
    }
}

/// 时间感知建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeAwareSuggestion {
    /// 建议类型
    pub suggestion_type: TimeSuggestionType,
    
    /// 建议标题
    pub title: String,
    
    /// 详细描述
    pub description: String,
    
    /// 建议的优先级
    pub priority: u8,
    
    /// 适用的时间段
    pub applicable_periods: Vec<TimePeriod>,
    
    /// 适用的星期
    pub applicable_weekdays: Option<HashSet<Weekday>>,
}

/// 时间建议类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeSuggestionType {
    /// 休息提醒
    BreakReminder,
    
    /// 任务建议
    TaskSuggestion,
    
    /// 工具推荐
    ToolRecommendation,
    
    /// 工作模式切换
    WorkModeSwitch,
    
    /// 会话总结
    SessionSummary,
    
    /// 目标提醒
    GoalReminder,
}

/// Kairos 管理器
#[derive(Debug)]
pub struct KairosManager {
    /// 应用状态
    state: AppState,
    
    /// 用户活动模式
    activity_pattern: UserActivityPattern,
    
    /// 当前时间
    current_time: DateTime<Utc>,
    
    /// 会话开始时间
    session_start_time: DateTime<Utc>,
    
    /// 时间感知建议
    suggestions: Vec<TimeAwareSuggestion>,
    
    /// 是否启用自动提醒
    auto_reminders: bool,
    
    /// 提醒间隔
    reminder_interval: Duration,
}

impl KairosManager {
    /// 创建新的 Kairos 管理器
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            activity_pattern: UserActivityPattern::default(),
            current_time: Utc::now(),
            session_start_time: Utc::now(),
            suggestions: Vec::new(),
            auto_reminders: true,
            reminder_interval: Duration::minutes(60),
        }
    }
    
    /// 获取当前时间段
    pub fn current_period(&self) -> TimePeriod {
        let hour = self.current_time.hour();
        match hour {
            0..=4 => TimePeriod::LateNight,
            5..=7 => TimePeriod::EarlyMorning,
            8..=11 => TimePeriod::Morning,
            12..=13 => TimePeriod::Noon,
            14..=17 => TimePeriod::Afternoon,
            18..=20 => TimePeriod::Evening,
            21..=23 => TimePeriod::Night,
            _ => TimePeriod::Night,
        }
    }
    
    /// 获取当前星期几
    pub fn current_weekday(&self) -> Weekday {
        self.current_time.weekday()
    }
    
    /// 是否是工作日
    pub fn is_weekday(&self) -> bool {
        !matches!(
            self.current_weekday(),
            Weekday::Sat | Weekday::Sun
        )
    }
    
    /// 获取会话持续时间
    pub fn session_duration(&self) -> Duration {
        self.current_time.signed_duration_since(self.session_start_time)
    }
    
    /// 记录工具使用
    pub fn record_tool_usage(&mut self, tool_name: &str) {
        *self.activity_pattern.tool_usage.entry(tool_name.to_string()).or_insert(0) += 1;
    }
    
    /// 记录当前活动
    pub fn record_activity(&mut self) {
        let weekday = self.current_weekday();
        let period = self.current_period();
        
        self.activity_pattern
            .weekday_patterns
            .entry(weekday)
            .or_insert_with(Vec::new)
            .push(period);
    }
    
    /// 添加时间感知建议
    pub fn add_suggestion(&mut self, suggestion: TimeAwareSuggestion) {
        self.suggestions.push(suggestion);
    }
    
    /// 获取适用于当前时间的建议
    pub fn applicable_suggestions(&self) -> Vec<&TimeAwareSuggestion> {
        let current_period = self.current_period();
        let current_weekday = self.current_weekday();
        
        self.suggestions
            .iter()
            .filter(|s| {
                s.applicable_periods.contains(&current_period)
                    && s.applicable_weekdays
                        .as_ref()
                        .map(|w| w.contains(&current_weekday))
                        .unwrap_or(true)
            })
            .collect()
    }
    
    /// 创建休息提醒
    pub fn create_break_reminder() -> TimeAwareSuggestion {
        TimeAwareSuggestion {
            suggestion_type: TimeSuggestionType::BreakReminder,
            title: "休息提醒".to_string(),
            description: "您已经工作了一段时间，建议休息一下。".to_string(),
            priority: 5,
            applicable_periods: vec![
                TimePeriod::Morning,
                TimePeriod::Afternoon,
                TimePeriod::Evening,
            ],
            applicable_weekdays: None,
        }
    }
    
    /// 创建任务建议
    pub fn create_task_suggestion(
        title: String,
        description: String,
        priority: u8,
        periods: Vec<TimePeriod>,
    ) -> TimeAwareSuggestion {
        TimeAwareSuggestion {
            suggestion_type: TimeSuggestionType::TaskSuggestion,
            title,
            description,
            priority: priority.clamp(1, 10),
            applicable_periods: periods,
            applicable_weekdays: None,
        }
    }
    
    /// 更新当前时间
    pub fn update_time(&mut self) {
        self.current_time = Utc::now();
    }
    
    /// 设置自动提醒
    pub fn set_auto_reminders(&mut self, enabled: bool) {
        self.auto_reminders = enabled;
    }
    
    /// 设置提醒间隔
    pub fn set_reminder_interval(&mut self, interval: Duration) {
        self.reminder_interval = interval;
    }
    
    /// 检查是否应该显示提醒
    pub fn should_show_reminder(&self) -> bool {
        if !self.auto_reminders {
            return false;
        }
        
        let elapsed = self.session_duration();
        elapsed >= self.reminder_interval
    }
}

/// 获取一天中的问候语
pub fn get_time_greeting() -> &'static str {
    let hour = Utc::now().hour();
    match hour {
        5..=11 => "早上好！",
        12..=17 => "下午好！",
        18..=21 => "晚上好！",
        _ => "你好！",
    }
}
