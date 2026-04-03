//! AGENT_TRIGGERS 代理触发器
//! 
//! 这个模块实现了代理触发器系统，允许基于事件自动触发代理行为。

use crate::error::Result;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// 触发器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TriggerType {
    /// 时间触发
    Time,
    
    /// 文件变化触发
    FileChange,
    
    /// 用户输入触发
    UserInput,
    
    /// 系统事件触发
    SystemEvent,
    
    /// 自定义触发
    Custom,
}

/// 触发器条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerCondition {
    /// 触发器类型
    pub trigger_type: TriggerType,
    
    /// 条件配置
    pub config: HashMap<String, serde_json::Value>,
    
    /// 是否启用
    pub enabled: bool,
}

/// 触发器动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerAction {
    /// 动作类型
    pub action_type: String,
    
    /// 动作参数
    pub params: HashMap<String, serde_json::Value>,
}

/// 代理触发器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTrigger {
    /// 触发器 ID
    pub id: String,
    
    /// 触发器名称
    pub name: String,
    
    /// 触发条件
    pub condition: TriggerCondition,
    
    /// 触发动作
    pub action: TriggerAction,
    
    /// 触发次数
    pub trigger_count: usize,
    
    /// 最大触发次数（None 表示无限制）
    pub max_triggers: Option<usize>,
    
    /// 冷却时间（秒）
    pub cooldown_seconds: u64,
    
    /// 上次触发时间
    pub last_triggered: Option<String>,
    
    /// 是否启用
    pub enabled: bool,
}

/// 触发器事件
#[derive(Debug, Clone)]
pub struct TriggerEvent {
    /// 事件类型
    pub event_type: TriggerType,
    
    /// 事件数据
    pub data: HashMap<String, serde_json::Value>,
    
    /// 时间戳
    pub timestamp: String,
}

/// 事件监听器类型
pub type EventListener = Box<dyn Fn(&TriggerEvent) -> Result<()> + Send + Sync>;

/// 代理触发器管理器
pub struct AgentTriggerManager {
    /// 应用状态
    state: AppState,
    
    /// 触发器列表
    triggers: HashMap<String, AgentTrigger>,
    
    /// 事件队列
    event_queue: Vec<TriggerEvent>,
    
    /// 事件监听器
    listeners: HashMap<TriggerType, Vec<EventListener>>,
}

impl std::fmt::Debug for AgentTriggerManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentTriggerManager")
            .field("trigger_count", &self.triggers.len())
            .field("event_queue_len", &self.event_queue.len())
            .finish()
    }
}

impl AgentTriggerManager {
    /// 创建新的触发器管理器
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            triggers: HashMap::new(),
            event_queue: Vec::new(),
            listeners: HashMap::new(),
        }
    }
    
    /// 创建触发器
    pub fn create_trigger(
        &mut self,
        name: String,
        condition: TriggerCondition,
        action: TriggerAction,
        cooldown_seconds: u64,
        max_triggers: Option<usize>,
    ) -> AgentTrigger {
        let id = generate_trigger_id();
        
        let trigger = AgentTrigger {
            id: id.clone(),
            name,
            condition,
            action,
            trigger_count: 0,
            max_triggers,
            cooldown_seconds,
            last_triggered: None,
            enabled: true,
        };
        
        self.triggers.insert(id.clone(), trigger.clone());
        trigger
    }
    
    /// 删除触发器
    pub fn delete_trigger(&mut self, trigger_id: &str) -> bool {
        self.triggers.remove(trigger_id).is_some()
    }
    
    /// 获取触发器
    pub fn get_trigger(&self, trigger_id: &str) -> Option<&AgentTrigger> {
        self.triggers.get(trigger_id)
    }
    
    /// 列出所有触发器
    pub fn list_triggers(&self) -> Vec<&AgentTrigger> {
        self.triggers.values().collect()
    }
    
    /// 启用触发器
    pub fn enable_trigger(&mut self, trigger_id: &str) -> Result<()> {
        if let Some(trigger) = self.triggers.get_mut(trigger_id) {
            trigger.enabled = true;
            Ok(())
        } else {
            Err(format!("Trigger '{}' not found", trigger_id).into())
        }
    }
    
    /// 禁用触发器
    pub fn disable_trigger(&mut self, trigger_id: &str) -> Result<()> {
        if let Some(trigger) = self.triggers.get_mut(trigger_id) {
            trigger.enabled = false;
            Ok(())
        } else {
            Err(format!("Trigger '{}' not found", trigger_id).into())
        }
    }
    
    /// 提交事件
    pub fn submit_event(&mut self, event: TriggerEvent) {
        self.event_queue.push(event);
    }
    
    /// 处理事件队列
    pub async fn process_events(&mut self) -> Vec<String> {
        let mut triggered_ids = Vec::new();
        let events: Vec<_> = self.event_queue.drain(..).collect();
        
        for event in events {
            if let Some(listeners) = self.listeners.get(&event.event_type) {
                for listener in listeners {
                    let _ = listener(&event);
                }
            }
            
            // 收集需要触发的触发器 ID
            let mut to_trigger = Vec::new();
            
            for (id, trigger) in &self.triggers {
                if !trigger.enabled {
                    continue;
                }
                
                if trigger.condition.trigger_type != event.event_type {
                    continue;
                }
                
                if let Some(max) = trigger.max_triggers {
                    if trigger.trigger_count >= max {
                        continue;
                    }
                }
                
                if let Some(last) = &trigger.last_triggered {
                    if let Ok(last_time) = chrono::DateTime::parse_from_rfc3339(last) {
                        let now = chrono::Utc::now();
                        let elapsed = now.signed_duration_since(last_time);
                        if elapsed.num_seconds() < trigger.cooldown_seconds as i64 {
                            continue;
                        }
                    }
                }
                
                if self.evaluate_condition(&trigger.condition, &event) {
                    to_trigger.push(id.clone());
                }
            }
            
            // 执行触发的动作
            for id in to_trigger {
                if let Some(trigger) = self.triggers.get(&id) {
                    let action = trigger.action.clone();
                    if let Err(e) = self.execute_action(&action).await {
                        eprintln!("Error executing action: {}", e);
                    }
                }
                
                if let Some(trigger) = self.triggers.get_mut(&id) {
                    trigger.trigger_count += 1;
                    trigger.last_triggered = Some(chrono::Utc::now().to_rfc3339());
                    triggered_ids.push(id);
                }
            }
        }
        
        triggered_ids
    }
    
    /// 评估条件
    fn evaluate_condition(&self, _condition: &TriggerCondition, _event: &TriggerEvent) -> bool {
        true
    }
    
    /// 执行动作
    async fn execute_action(&self, _action: &TriggerAction) -> Result<()> {
        Ok(())
    }
    
    /// 添加事件监听器
    pub fn add_listener<F>(&mut self, trigger_type: TriggerType, listener: F)
    where
        F: Fn(&TriggerEvent) -> Result<()> + Send + Sync + 'static,
    {
        self.listeners
            .entry(trigger_type)
            .or_insert_with(Vec::new)
            .push(Box::new(listener));
    }
    
    /// 创建时间触发器
    pub fn create_time_trigger(
        &mut self,
        name: String,
        duration: Duration,
        action: TriggerAction,
    ) -> AgentTrigger {
        let mut config = HashMap::new();
        config.insert("duration_ms".to_string(), (duration.as_millis() as u64).into());
        
        let condition = TriggerCondition {
            trigger_type: TriggerType::Time,
            config,
            enabled: true,
        };
        
        self.create_trigger(name, condition, action, 0, None)
    }
    
    /// 创建文件变化触发器
    pub fn create_file_change_trigger(
        &mut self,
        name: String,
        file_pattern: String,
        action: TriggerAction,
    ) -> AgentTrigger {
        let mut config = HashMap::new();
        config.insert("pattern".to_string(), file_pattern.into());
        
        let condition = TriggerCondition {
            trigger_type: TriggerType::FileChange,
            config,
            enabled: true,
        };
        
        self.create_trigger(name, condition, action, 5, None)
    }
}

/// 生成触发器 ID
fn generate_trigger_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("trigger_{}", rng.gen::<u64>())
}
