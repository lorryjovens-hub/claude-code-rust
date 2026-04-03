//! 状态管理系统
//! 
//! 这个模块实现了全局状态管理，对应 TypeScript 的 bootstrap/state.ts

pub mod app_state;
pub mod signal;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

// 重新导出主要类型
pub use app_state::{AppState, AppStateExt, new_app_state};
pub use signal::{Signal, SignalManager, on_session_switched, emit_session_switched};

/// 会话 ID
pub type SessionId = String;

/// 模型使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    /// 输入 token 数
    pub input_tokens: u64,
    
    /// 输出 token 数
    pub output_tokens: u64,
    
    /// 缓存读取输入 token 数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u64>,
    
    /// 缓存创建输入 token 数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u64>,
    
    /// Web 搜索请求数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search_requests: Option<u64>,
}

/// 模型设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSetting {
    /// 模型名称
    pub model: String,
    
    /// 模型来源
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// 会话定时任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCronTask {
    /// 任务 ID
    pub id: String,
    
    /// Cron 表达式
    pub cron: String,
    
    /// 提示内容
    pub prompt: String,
    
    /// 创建时间
    pub created_at: i64,
    
    /// 是否重复
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurring: Option<bool>,
    
    /// 代理 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
}

/// 调用的技能信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvokedSkillInfo {
    /// 技能名称
    pub skill_name: String,
    
    /// 技能路径
    pub skill_path: String,
    
    /// 内容
    pub content: String,
    
    /// 调用时间
    pub invoked_at: i64,
    
    /// 代理 ID
    pub agent_id: Option<String>,
}

/// 慢操作记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowOperation {
    /// 操作名称
    pub operation: String,
    
    /// 持续时间（毫秒）
    pub duration_ms: u64,
    
    /// 时间戳
    pub timestamp: i64,
}

/// 代理颜色名称
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentColorName {
    Red,
    Green,
    Blue,
    Yellow,
    Magenta,
    Cyan,
    White,
}

/// 全局状态
#[derive(Debug, Clone)]
pub struct State {
    // 项目信息
    /// 原始工作目录
    pub original_cwd: PathBuf,
    
    /// 项目根目录
    pub project_root: PathBuf,
    
    /// 当前工作目录
    pub cwd: PathBuf,
    
    // 会话信息
    /// 会话 ID
    pub session_id: SessionId,
    
    /// 父会话 ID
    pub parent_session_id: Option<SessionId>,
    
    // 使用统计
    /// 总成本（美元）
    pub total_cost_usd: f64,
    
    /// 总 API 持续时间（毫秒）
    pub total_api_duration: u64,
    
    /// 总工具持续时间（毫秒）
    pub total_tool_duration: u64,
    
    /// 开始时间
    pub start_time: i64,
    
    /// 最后交互时间
    pub last_interaction_time: i64,
    
    /// 总添加行数
    pub total_lines_added: u64,
    
    /// 总删除行数
    pub total_lines_removed: u64,
    
    /// 是否有未知模型成本
    pub has_unknown_model_cost: bool,
    
    /// 模型使用统计
    pub model_usage: HashMap<String, ModelUsage>,
    
    // 模型配置
    /// 主循环模型覆盖
    pub main_loop_model_override: Option<ModelSetting>,
    
    /// 初始主循环模型
    pub initial_main_loop_model: Option<ModelSetting>,
    
    // 代理状态
    /// 代理颜色映射
    pub agent_color_map: HashMap<String, AgentColorName>,
    
    /// 代理颜色索引
    pub agent_color_index: usize,
    
    // 插件状态
    /// 内联插件
    pub inline_plugins: Vec<String>,
    
    /// 使用协同插件
    pub use_cowork_plugins: bool,
    
    // 权限状态
    /// 会话绕过权限模式
    pub session_bypass_permissions_mode: bool,
    
    /// 会话信任已接受
    pub session_trust_accepted: bool,
    
    // 任务状态
    /// 定时任务启用
    pub scheduled_tasks_enabled: bool,
    
    /// 会话定时任务
    pub session_cron_tasks: Vec<SessionCronTask>,
    
    /// 会话创建的团队
    pub session_created_teams: HashSet<String>,
    
    // 会话标志
    /// 会话持久化禁用
    pub session_persistence_disabled: bool,
    
    /// 已退出计划模式
    pub has_exited_plan_mode: bool,
    
    /// 需要计划模式退出附件
    pub needs_plan_mode_exit_attachment: bool,
    
    /// 需要自动模式退出附件
    pub needs_auto_mode_exit_attachment: bool,
    
    /// LSP 推荐已显示
    pub lsp_recommendation_shown_this_session: bool,
    
    // 其他状态
    /// 是否交互式
    pub is_interactive: bool,
    
    /// 客户端类型
    pub client_type: String,
    
    /// 是否远程模式
    pub is_remote_mode: bool,
    
    /// 调用的技能
    pub invoked_skills: HashMap<String, InvokedSkillInfo>,
    
    /// 慢操作列表
    pub slow_operations: Vec<SlowOperation>,
}

impl Default for State {
    fn default() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let now = chrono::Utc::now().timestamp_millis();
        
        Self {
            original_cwd: cwd.clone(),
            project_root: cwd.clone(),
            cwd,
            session_id: uuid::Uuid::new_v4().to_string(),
            parent_session_id: None,
            total_cost_usd: 0.0,
            total_api_duration: 0,
            total_tool_duration: 0,
            start_time: now,
            last_interaction_time: now,
            total_lines_added: 0,
            total_lines_removed: 0,
            has_unknown_model_cost: false,
            model_usage: HashMap::new(),
            main_loop_model_override: None,
            initial_main_loop_model: None,
            agent_color_map: HashMap::new(),
            agent_color_index: 0,
            inline_plugins: Vec::new(),
            use_cowork_plugins: false,
            session_bypass_permissions_mode: false,
            session_trust_accepted: false,
            scheduled_tasks_enabled: false,
            session_cron_tasks: Vec::new(),
            session_created_teams: HashSet::new(),
            session_persistence_disabled: false,
            has_exited_plan_mode: false,
            needs_plan_mode_exit_attachment: false,
            needs_auto_mode_exit_attachment: false,
            lsp_recommendation_shown_this_session: false,
            is_interactive: true,
            client_type: "cli".to_string(),
            is_remote_mode: false,
            invoked_skills: HashMap::new(),
            slow_operations: Vec::new(),
        }
    }
}

impl State {
    /// 创建新状态
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 获取总持续时间
    pub fn get_total_duration(&self) -> i64 {
        chrono::Utc::now().timestamp_millis() - self.start_time
    }
    
    /// 获取总输入 token 数
    pub fn get_total_input_tokens(&self) -> u64 {
        self.model_usage.values().map(|u| u.input_tokens).sum()
    }
    
    /// 获取总输出 token 数
    pub fn get_total_output_tokens(&self) -> u64 {
        self.model_usage.values().map(|u| u.output_tokens).sum()
    }
    
    /// 添加成本
    pub fn add_cost(&mut self, cost: f64, model: String, usage: ModelUsage) {
        self.total_cost_usd += cost;
        self.model_usage.insert(model, usage);
    }
    
    /// 添加 API 持续时间
    pub fn add_api_duration(&mut self, duration: u64) {
        self.total_api_duration += duration;
    }
    
    /// 添加工具持续时间
    pub fn add_tool_duration(&mut self, duration: u64) {
        self.total_tool_duration += duration;
    }
    
    /// 添加行变更
    pub fn add_lines_changed(&mut self, added: u64, removed: u64) {
        self.total_lines_added += added;
        self.total_lines_removed += removed;
    }
    
    /// 更新最后交互时间
    pub fn update_last_interaction_time(&mut self) {
        self.last_interaction_time = chrono::Utc::now().timestamp_millis();
    }
    
    /// 重置成本状态
    pub fn reset_cost_state(&mut self) {
        self.total_cost_usd = 0.0;
        self.total_api_duration = 0;
        self.total_tool_duration = 0;
        self.start_time = chrono::Utc::now().timestamp_millis();
        self.total_lines_added = 0;
        self.total_lines_removed = 0;
        self.has_unknown_model_cost = false;
        self.model_usage.clear();
    }
    
    /// 重新生成会话 ID
    pub fn regenerate_session_id(&mut self, set_current_as_parent: bool) -> SessionId {
        if set_current_as_parent {
            self.parent_session_id = Some(self.session_id.clone());
        }
        self.session_id = uuid::Uuid::new_v4().to_string();
        self.session_id.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_state_default() {
        let state = State::default();
        assert!(!state.session_id.is_empty());
        assert_eq!(state.total_cost_usd, 0.0);
        assert!(state.is_interactive);
    }
    
    #[test]
    fn test_state_add_cost() {
        let mut state = State::new();
        
        let usage = ModelUsage {
            input_tokens: 100,
            output_tokens: 50,
            cache_read_input_tokens: None,
            cache_creation_input_tokens: None,
            web_search_requests: None,
        };
        
        state.add_cost(0.01, "claude-3-opus".to_string(), usage);
        
        assert_eq!(state.total_cost_usd, 0.01);
        assert_eq!(state.get_total_input_tokens(), 100);
        assert_eq!(state.get_total_output_tokens(), 50);
    }
    
    #[test]
    fn test_state_regenerate_session_id() {
        let mut state = State::new();
        let old_id = state.session_id.clone();
        
        let new_id = state.regenerate_session_id(true);
        
        assert_ne!(old_id, new_id);
        assert_eq!(state.parent_session_id, Some(old_id));
    }
}
