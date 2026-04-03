//! BUDDY 伴侣系统
//! 
//! 这个模块实现了 AI 伙伴陪伴式交互，提供更友好和个性化的用户体验。

use crate::error::Result;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 伙伴性格类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuddyPersonality {
    /// 友好型
    Friendly,
    
    /// 专业型
    Professional,
    
    /// 幽默型
    Humorous,
    
    /// 简洁型
    Concise,
    
    /// 导师型
    Mentoring,
    
    /// 伙伴型
    Buddy,
}

impl Default for BuddyPersonality {
    fn default() -> Self {
        BuddyPersonality::Friendly
    }
}

/// 伙伴状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BuddyState {
    /// 空闲
    Idle,
    
    /// 活跃
    Active,
    
    /// 思考中
    Thinking,
    
    /// 回复中
    Responding,
    
    /// 等待用户
    WaitingForUser,
}

impl Default for BuddyState {
    fn default() -> Self {
        BuddyState::Idle
    }
}

/// 伙伴配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuddyConfig {
    /// 伙伴名称
    pub name: String,
    
    /// 伙伴性格
    pub personality: BuddyPersonality,
    
    /// 是否启用
    pub enabled: bool,
    
    /// 对话风格
    pub conversation_style: ConversationStyle,
    
    /// 主动提示频率
    pub proactive_frequency: ProactiveFrequency,
    
    /// 自定义问候语
    pub custom_greetings: Vec<String>,
}

impl Default for BuddyConfig {
    fn default() -> Self {
        Self {
            name: "Claude".to_string(),
            personality: BuddyPersonality::Friendly,
            enabled: false,
            conversation_style: ConversationStyle::default(),
            proactive_frequency: ProactiveFrequency::Normal,
            custom_greetings: Vec::new(),
        }
    }
}

/// 对话风格
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversationStyle {
    /// 正式
    Formal,
    
    /// 随意
    Casual,
    
    /// 半正式
    SemiFormal,
}

impl Default for ConversationStyle {
    fn default() -> Self {
        ConversationStyle::Casual
    }
}

/// 主动提示频率
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProactiveFrequency {
    /// 从不
    Never,
    
    /// 很少
    Rare,
    
    /// 正常
    Normal,
    
    /// 频繁
    Frequent,
    
    /// 非常频繁
    VeryFrequent,
}

impl Default for ProactiveFrequency {
    fn default() -> Self {
        ProactiveFrequency::Normal
    }
}

/// 对话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuddyMessage {
    /// 消息 ID
    pub id: String,
    
    /// 发送者
    pub sender: MessageSender,
    
    /// 消息内容
    pub content: String,
    
    /// 时间戳
    pub timestamp: String,
    
    /// 消息类型
    pub message_type: MessageType,
    
    /// 情感标签
    pub emotion: Option<Emotion>,
}

/// 消息发送者
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageSender {
    /// 用户
    User,
    
    /// 伙伴
    Buddy,
    
    /// 系统
    System,
}

/// 消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// 问候
    Greeting,
    
    /// 问题
    Question,
    
    /// 回答
    Answer,
    
    /// 建议
    Suggestion,
    
    /// 提醒
    Reminder,
    
    /// 告别
    Farewell,
    
    /// 普通
    Normal,
}

/// 情感类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Emotion {
    /// 开心
    Happy,
    
    /// 中性
    Neutral,
    
    /// 思考
    Thinking,
    
    /// 鼓励
    Encouraging,
    
    /// 严肃
    Serious,
    
    /// 好奇
    Curious,
}

/// 对话历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationHistory {
    /// 消息列表
    pub messages: Vec<BuddyMessage>,
    
    /// 对话开始时间
    pub start_time: String,
    
    /// 最后活动时间
    pub last_activity_time: String,
    
    /// 消息计数
    pub message_count: usize,
}

impl Default for ConversationHistory {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            start_time: chrono::Utc::now().to_rfc3339(),
            last_activity_time: chrono::Utc::now().to_rfc3339(),
            message_count: 0,
        }
    }
}

/// Buddy 管理器
#[derive(Debug)]
pub struct BuddyManager {
    /// 应用状态
    app_state: AppState,
    
    /// 伙伴配置
    config: BuddyConfig,
    
    /// 伙伴状态
    buddy_state: BuddyState,
    
    /// 对话历史
    conversation_history: ConversationHistory,
    
    /// 用户偏好
    user_preferences: HashMap<String, serde_json::Value>,
}

impl BuddyManager {
    /// 创建新的 Buddy 管理器
    pub fn new(app_state: AppState) -> Self {
        Self {
            app_state,
            config: BuddyConfig::default(),
            buddy_state: BuddyState::Idle,
            conversation_history: ConversationHistory::default(),
            user_preferences: HashMap::new(),
        }
    }
    
    /// 获取配置
    pub fn config(&self) -> &BuddyConfig {
        &self.config
    }
    
    /// 获取可变配置
    pub fn config_mut(&mut self) -> &mut BuddyConfig {
        &mut self.config
    }
    
    /// 启用 Buddy
    pub fn enable(&mut self) {
        self.config.enabled = true;
        self.buddy_state = BuddyState::Active;
    }
    
    /// 禁用 Buddy
    pub fn disable(&mut self) {
        self.config.enabled = false;
        self.buddy_state = BuddyState::Idle;
    }
    
    /// 设置名称
    pub fn set_name(&mut self, name: String) {
        self.config.name = name;
    }
    
    /// 设置性格
    pub fn set_personality(&mut self, personality: BuddyPersonality) {
        self.config.personality = personality;
    }
    
    /// 设置对话风格
    pub fn set_conversation_style(&mut self, style: ConversationStyle) {
        self.config.conversation_style = style;
    }
    
    /// 设置主动提示频率
    pub fn set_proactive_frequency(&mut self, frequency: ProactiveFrequency) {
        self.config.proactive_frequency = frequency;
    }
    
    /// 添加自定义问候语
    pub fn add_custom_greeting(&mut self, greeting: String) {
        self.config.custom_greetings.push(greeting);
    }
    
    /// 发送消息
    pub fn send_message(&mut self, content: String, message_type: MessageType) -> Result<BuddyMessage> {
        let message = BuddyMessage {
            id: generate_message_id(),
            sender: MessageSender::Buddy,
            content,
            timestamp: chrono::Utc::now().to_rfc3339(),
            message_type,
            emotion: None,
        };
        
        self.conversation_history.messages.push(message.clone());
        self.conversation_history.message_count += 1;
        self.conversation_history.last_activity_time = message.timestamp.clone();
        
        Ok(message)
    }
    
    /// 接收用户消息
    pub fn receive_user_message(&mut self, content: String) -> Result<BuddyMessage> {
        let message = BuddyMessage {
            id: generate_message_id(),
            sender: MessageSender::User,
            content,
            timestamp: chrono::Utc::now().to_rfc3339(),
            message_type: MessageType::Normal,
            emotion: None,
        };
        
        self.conversation_history.messages.push(message.clone());
        self.conversation_history.message_count += 1;
        self.conversation_history.last_activity_time = message.timestamp.clone();
        
        Ok(message)
    }
    
    /// 获取问候语
    pub fn get_greeting(&self) -> String {
        if !self.config.custom_greetings.is_empty() {
            let index = rand::random::<usize>() % self.config.custom_greetings.len();
            self.config.custom_greetings[index].clone()
        } else {
            match self.config.personality {
                BuddyPersonality::Friendly => "你好！很高兴见到你！",
                BuddyPersonality::Professional => "您好，准备好开始工作了。",
                BuddyPersonality::Humorous => "嘿！准备好一起编程了吗？",
                BuddyPersonality::Concise => "你好。",
                BuddyPersonality::Mentoring => "欢迎！让我们一起学习和成长。",
                BuddyPersonality::Buddy => "嘿，伙计！准备好写代码了吗？",
            }.to_string()
        }
    }
    
    /// 获取对话历史
    pub fn conversation_history(&self) -> &ConversationHistory {
        &self.conversation_history
    }
    
    /// 获取最近的消息
    pub fn recent_messages(&self, count: usize) -> Vec<&BuddyMessage> {
        let start = self.conversation_history.messages.len().saturating_sub(count);
        self.conversation_history.messages[start..].iter().collect()
    }
    
    /// 设置用户偏好
    pub fn set_user_preference(&mut self, key: &str, value: serde_json::Value) {
        self.user_preferences.insert(key.to_string(), value);
    }
    
    /// 获取用户偏好
    pub fn get_user_preference(&self, key: &str) -> Option<&serde_json::Value> {
        self.user_preferences.get(key)
    }
    
    /// 清空对话历史
    pub fn clear_history(&mut self) {
        self.conversation_history = ConversationHistory::default();
    }
}

/// 生成消息 ID
fn generate_message_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("msg_{}", rng.gen::<u64>())
}
