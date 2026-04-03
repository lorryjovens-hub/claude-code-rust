//! 特性开关系统
//! 
//! 这个模块管理 Claude Code 的所有实验性功能和特性开关。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 特性开关枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeatureFlag {
    /// 主动建议模式
    Proactive,
    
    /// 时间感知系统
    Kairos,
    
    /// 语音交互模式
    VoiceMode,
    
    /// 超级规划系统
    Ultraplan,
    
    /// 未知功能
    Torch,
    
    /// 伴侣系统
    Buddy,
    
    /// 子代理分叉
    ForkSubagent,
    
    /// 远程控制模式
    BridgeMode,
    
    /// 守护进程模式
    Daemon,
    
    /// 协调器模式
    CoordinatorMode,
    
    /// 代理触发器
    AgentTriggers,
    
    /// Web 浏览器工具
    WebBrowserTool,
    
    /// 历史片段
    HistorySnip,
    
    /// 工作流脚本
    WorkflowScripts,
}

impl FeatureFlag {
    /// 获取特性的描述
    pub fn description(&self) -> &'static str {
        match self {
            FeatureFlag::Proactive => "主动建议模式 - AI 主动提供建议",
            FeatureFlag::Kairos => "时间感知系统 - 根据时间提供智能响应",
            FeatureFlag::VoiceMode => "语音交互模式 - 支持语音输入和输出",
            FeatureFlag::Ultraplan => "超级规划系统 - 复杂任务的智能规划",
            FeatureFlag::Torch => "未知功能 - 实验性功能",
            FeatureFlag::Buddy => "伴侣系统 - AI 伙伴陪伴式交互",
            FeatureFlag::ForkSubagent => "子代理分叉 - 支持创建子代理处理任务",
            FeatureFlag::BridgeMode => "远程控制模式 - 支持远程会话管理",
            FeatureFlag::Daemon => "守护进程模式 - 后台运行服务",
            FeatureFlag::CoordinatorMode => "协调器模式 - 多工作器协调",
            FeatureFlag::AgentTriggers => "代理触发器 - 基于事件触发代理",
            FeatureFlag::WebBrowserTool => "Web 浏览器工具 - 网页浏览和交互",
            FeatureFlag::HistorySnip => "历史片段 - 上下文历史片段",
            FeatureFlag::WorkflowScripts => "工作流脚本 - 自动化工作流脚本",
        }
    }
    
    /// 检查特性是否为实验性
    pub fn is_experimental(&self) -> bool {
        match self {
            FeatureFlag::Proactive => true,
            FeatureFlag::Kairos => true,
            FeatureFlag::VoiceMode => true,
            FeatureFlag::Ultraplan => false,
            FeatureFlag::Torch => true,
            FeatureFlag::Buddy => true,
            FeatureFlag::ForkSubagent => true,
            FeatureFlag::BridgeMode => false,
            FeatureFlag::Daemon => false,
            FeatureFlag::CoordinatorMode => true,
            FeatureFlag::AgentTriggers => true,
            FeatureFlag::WebBrowserTool => true,
            FeatureFlag::HistorySnip => true,
            FeatureFlag::WorkflowScripts => true,
        }
    }
    
    /// 获取所有特性
    pub fn all() -> &'static [FeatureFlag] {
        &[
            FeatureFlag::Proactive,
            FeatureFlag::Kairos,
            FeatureFlag::VoiceMode,
            FeatureFlag::Ultraplan,
            FeatureFlag::Torch,
            FeatureFlag::Buddy,
            FeatureFlag::ForkSubagent,
            FeatureFlag::BridgeMode,
            FeatureFlag::Daemon,
            FeatureFlag::CoordinatorMode,
            FeatureFlag::AgentTriggers,
            FeatureFlag::WebBrowserTool,
            FeatureFlag::HistorySnip,
            FeatureFlag::WorkflowScripts,
        ]
    }
}

impl std::fmt::Display for FeatureFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FeatureFlag::Proactive => "PROACTIVE",
            FeatureFlag::Kairos => "KAIROS",
            FeatureFlag::VoiceMode => "VOICE_MODE",
            FeatureFlag::Ultraplan => "ULTRAPLAN",
            FeatureFlag::Torch => "TORCH",
            FeatureFlag::Buddy => "BUDDY",
            FeatureFlag::ForkSubagent => "FORK_SUBAGENT",
            FeatureFlag::BridgeMode => "BRIDGE_MODE",
            FeatureFlag::Daemon => "DAEMON",
            FeatureFlag::CoordinatorMode => "COORDINATOR_MODE",
            FeatureFlag::AgentTriggers => "AGENT_TRIGGERS",
            FeatureFlag::WebBrowserTool => "WEB_BROWSER_TOOL",
            FeatureFlag::HistorySnip => "HISTORY_SNIP",
            FeatureFlag::WorkflowScripts => "WORKFLOW_SCRIPTS",
        };
        write!(f, "{}", s)
    }
}

/// 特性管理器
#[derive(Debug)]
pub struct FeatureManager {
    /// 特性状态
    features: HashMap<FeatureFlag, bool>,
    
    /// 特性默认状态
    defaults: HashMap<FeatureFlag, bool>,
}

impl Default for FeatureManager {
    fn default() -> Self {
        let mut defaults = HashMap::new();
        
        // 设置默认状态
        defaults.insert(FeatureFlag::Proactive, false);
        defaults.insert(FeatureFlag::Kairos, false);
        defaults.insert(FeatureFlag::VoiceMode, false);
        defaults.insert(FeatureFlag::Ultraplan, false);
        defaults.insert(FeatureFlag::Torch, false);
        defaults.insert(FeatureFlag::Buddy, false);
        defaults.insert(FeatureFlag::ForkSubagent, false);
        defaults.insert(FeatureFlag::BridgeMode, false);
        defaults.insert(FeatureFlag::Daemon, false);
        defaults.insert(FeatureFlag::CoordinatorMode, false);
        defaults.insert(FeatureFlag::AgentTriggers, false);
        defaults.insert(FeatureFlag::WebBrowserTool, false);
        defaults.insert(FeatureFlag::HistorySnip, false);
        defaults.insert(FeatureFlag::WorkflowScripts, false);
        
        Self {
            features: defaults.clone(),
            defaults,
        }
    }
}

impl FeatureManager {
    /// 创建新的特性管理器
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 检查特性是否启用
    pub fn is_enabled(&self, feature: FeatureFlag) -> bool {
        *self.features.get(&feature).unwrap_or(&false)
    }
    
    /// 启用特性
    pub fn enable(&mut self, feature: FeatureFlag) {
        self.features.insert(feature, true);
    }
    
    /// 禁用特性
    pub fn disable(&mut self, feature: FeatureFlag) {
        self.features.insert(feature, false);
    }
    
    /// 切换特性状态
    pub fn toggle(&mut self, feature: FeatureFlag) {
        let current = self.is_enabled(feature);
        self.features.insert(feature, !current);
    }
    
    /// 重置到默认状态
    pub fn reset(&mut self, feature: FeatureFlag) {
        if let Some(&default) = self.defaults.get(&feature) {
            self.features.insert(feature, default);
        }
    }
    
    /// 重置所有特性
    pub fn reset_all(&mut self) {
        self.features = self.defaults.clone();
    }
    
    /// 获取所有启用的特性
    pub fn enabled_features(&self) -> Vec<FeatureFlag> {
        self.features
            .iter()
            .filter(|(_, &enabled)| enabled)
            .map(|(&feature, _)| feature)
            .collect()
    }
    
    /// 获取所有特性及其状态
    pub fn all_features(&self) -> Vec<(FeatureFlag, bool)> {
        FeatureFlag::all()
            .iter()
            .map(|&feature| (feature, self.is_enabled(feature)))
            .collect()
    }
    
    /// 从环境变量加载特性配置
    pub fn load_from_env(&mut self) {
        for &feature in FeatureFlag::all() {
            let env_var = format!("CLAUDE_CODE_{}", feature);
            if let Ok(value) = std::env::var(env_var) {
                let enabled = value.to_lowercase() == "true" || value == "1";
                self.features.insert(feature, enabled);
            }
        }
    }
    
    /// 序列化特性状态
    pub fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        for (&feature, &enabled) in &self.features {
            map.insert(feature.to_string(), serde_json::Value::Bool(enabled));
        }
        serde_json::Value::Object(map)
    }
    
    /// 从 JSON 反序列化特性状态
    pub fn from_json(json: &serde_json::Value) -> Option<Self> {
        let mut manager = Self::new();
        
        if let Some(obj) = json.as_object() {
            for (&feature, _) in FeatureFlag::all().iter().zip(std::iter::repeat(())) {
                if let Some(&serde_json::Value::Bool(enabled)) = obj.get(&feature.to_string()) {
                    manager.features.insert(feature, enabled);
                }
            }
        }
        
        Some(manager)
    }
}

/// 特性模块子模块
pub mod proactive;
pub mod kairos;
pub mod voice;
pub mod ultraplan;
pub mod torch;
pub mod buddy;
pub mod fork_subagent;
pub mod coordinator;
pub mod agent_triggers;
pub mod web_browser;
pub mod history_snip;
pub mod workflow;
