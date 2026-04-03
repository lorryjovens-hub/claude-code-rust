//! FORK_SUBAGENT 子代理分叉
//! 
//! 这个模块实现了子代理分叉功能，支持创建独立的子代理来处理特定任务，
//! 并与主代理共享部分状态。

use crate::error::Result;
use crate::state::{AppState, SessionId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;

/// 子代理状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubagentState {
    /// 已创建
    Created,
    
    /// 初始化中
    Initializing,
    
    /// 运行中
    Running,
    
    /// 已暂停
    Paused,
    
    /// 已完成
    Completed,
    
    /// 已失败
    Failed,
    
    /// 已取消
    Cancelled,
}

impl Default for SubagentState {
    fn default() -> Self {
        SubagentState::Created
    }
}

/// 子代理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentConfig {
    /// 子代理名称
    pub name: String,
    
    /// 代理类型
    pub agent_type: String,
    
    /// 任务描述
    pub task_description: String,
    
    /// 共享缓存安全参数
    pub cache_safe_params: CacheSafeParams,
    
    /// 子代理上下文覆盖
    pub context_overrides: Option<SubagentContextOverrides>,
    
    /// 最大输出 token 数
    pub max_output_tokens: Option<u32>,
    
    /// 工具白名单
    pub allowed_tools: Vec<String>,
    
    /// 是否继承主代理权限
    pub inherit_permissions: bool,
}

impl Default for SubagentConfig {
    fn default() -> Self {
        Self {
            name: "subagent".to_string(),
            agent_type: "general-purpose".to_string(),
            task_description: String::new(),
            cache_safe_params: CacheSafeParams::default(),
            context_overrides: None,
            max_output_tokens: None,
            allowed_tools: Vec::new(),
            inherit_permissions: true,
        }
    }
}

/// 缓存安全参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSafeParams {
    /// 系统提示
    pub system_prompt: String,
    
    /// 用户上下文
    pub user_context: HashMap<String, String>,
    
    /// 系统上下文
    pub system_context: HashMap<String, String>,
    
    /// 分叉上下文消息
    pub fork_context_messages: Vec<String>,
}

impl Default for CacheSafeParams {
    fn default() -> Self {
        Self {
            system_prompt: String::new(),
            user_context: HashMap::new(),
            system_context: HashMap::new(),
            fork_context_messages: Vec::new(),
        }
    }
}

/// 子代理上下文覆盖
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentContextOverrides {
    /// 模型覆盖
    pub model_override: Option<String>,
    
    /// 温度覆盖
    pub temperature_override: Option<f32>,
    
    /// 工作目录覆盖
    pub cwd_override: Option<String>,
}

/// 子代理消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentMessage {
    /// 消息 ID
    pub id: String,
    
    /// 消息类型
    pub message_type: SubagentMessageType,
    
    /// 消息内容
    pub content: serde_json::Value,
    
    /// 时间戳
    pub timestamp: String,
}

/// 子代理消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubagentMessageType {
    /// 开始
    Start,
    
    /// 状态更新
    StatusUpdate,
    
    /// 进度更新
    ProgressUpdate,
    
    /// 结果
    Result,
    
    /// 错误
    Error,
    
    /// 日志
    Log,
    
    /// 完成
    Complete,
    
    /// 取消
    Cancel,
}

/// 子代理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentResult {
    /// 成功标志
    pub success: bool,
    
    /// 输出内容
    pub output: Option<String>,
    
    /// 错误信息
    pub error: Option<String>,
    
    /// 工具使用统计
    pub tool_usage: HashMap<String, u32>,
    
    /// Token 使用
    pub token_usage: TokenUsage,
    
    /// 持续时间（毫秒）
    pub duration_ms: u64,
}

/// Token 使用统计
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    /// 输入 tokens
    pub input_tokens: u64,
    
    /// 输出 tokens
    pub output_tokens: u64,
    
    /// 总 tokens
    pub total_tokens: u64,
}

/// 子代理信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentInfo {
    /// 子代理 ID
    pub id: String,
    
    /// 会话 ID
    pub session_id: SessionId,
    
    /// 父会话 ID
    pub parent_session_id: SessionId,
    
    /// 配置
    pub config: SubagentConfig,
    
    /// 状态
    pub state: SubagentState,
    
    /// 进度 (0.0-1.0)
    pub progress: f32,
    
    /// 创建时间
    pub created_at: String,
    
    /// 开始时间
    pub started_at: Option<String>,
    
    /// 完成时间
    pub completed_at: Option<String>,
    
    /// 结果
    pub result: Option<SubagentResult>,
}

/// 子代理管理器
#[derive(Debug)]
pub struct SubagentManager {
    /// 应用状态
    state: AppState,
    
    /// 子代理映射
    subagents: HashMap<String, SubagentInfo>,
    
    /// 消息发送器
    message_senders: HashMap<String, mpsc::Sender<SubagentMessage>>,
    
    /// 消息接收器
    message_receivers: HashMap<String, mpsc::Receiver<SubagentMessage>>,
}

impl SubagentManager {
    /// 创建新的子代理管理器
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            subagents: HashMap::new(),
            message_senders: HashMap::new(),
            message_receivers: HashMap::new(),
        }
    }
    
    /// 创建子代理
    pub fn create_subagent(
        &mut self,
        config: SubagentConfig,
        parent_session_id: SessionId,
    ) -> Result<SubagentInfo> {
        let subagent_id = generate_subagent_id();
        let session_id = generate_session_id();
        
        let (tx, rx) = mpsc::channel(100);
        
        let subagent_info = SubagentInfo {
            id: subagent_id.clone(),
            session_id,
            parent_session_id,
            config,
            state: SubagentState::Created,
            progress: 0.0,
            created_at: chrono::Utc::now().to_rfc3339(),
            started_at: None,
            completed_at: None,
            result: None,
        };
        
        self.subagents.insert(subagent_id.clone(), subagent_info.clone());
        self.message_senders.insert(subagent_id.clone(), tx);
        self.message_receivers.insert(subagent_id, rx);
        
        Ok(subagent_info)
    }
    
    /// 启动子代理
    pub async fn start_subagent(&mut self, subagent_id: &str) -> Result<()> {
        if let Some(subagent) = self.subagents.get_mut(subagent_id) {
            subagent.state = SubagentState::Initializing;
            
            if let Some(tx) = self.message_senders.get(subagent_id) {
                let message = SubagentMessage {
                    id: generate_message_id(),
                    message_type: SubagentMessageType::Start,
                    content: serde_json::json!({}),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                tx.send(message).await.ok();
            }
            
            subagent.state = SubagentState::Running;
            subagent.started_at = Some(chrono::Utc::now().to_rfc3339());
            
            Ok(())
        } else {
            Err(format!("Subagent '{}' not found", subagent_id).into())
        }
    }
    
    /// 取消子代理
    pub async fn cancel_subagent(&mut self, subagent_id: &str) -> Result<()> {
        if let Some(subagent) = self.subagents.get_mut(subagent_id) {
            if let Some(tx) = self.message_senders.get(subagent_id) {
                let message = SubagentMessage {
                    id: generate_message_id(),
                    message_type: SubagentMessageType::Cancel,
                    content: serde_json::json!({}),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                tx.send(message).await.ok();
            }
            
            subagent.state = SubagentState::Cancelled;
            Ok(())
        } else {
            Err(format!("Subagent '{}' not found", subagent_id).into())
        }
    }
    
    /// 获取子代理信息
    pub fn get_subagent(&self, subagent_id: &str) -> Option<&SubagentInfo> {
        self.subagents.get(subagent_id)
    }
    
    /// 获取所有子代理
    pub fn list_subagents(&self) -> Vec<&SubagentInfo> {
        self.subagents.values().collect()
    }
    
    /// 获取运行中的子代理
    pub fn running_subagents(&self) -> Vec<&SubagentInfo> {
        self.subagents
            .values()
            .filter(|s| s.state == SubagentState::Running)
            .collect()
    }
    
    /// 更新子代理进度
    pub fn update_progress(&mut self, subagent_id: &str, progress: f32) -> Result<()> {
        if let Some(subagent) = self.subagents.get_mut(subagent_id) {
            subagent.progress = progress.clamp(0.0, 1.0);
            Ok(())
        } else {
            Err(format!("Subagent '{}' not found", subagent_id).into())
        }
    }
    
    /// 设置子代理结果
    pub fn set_result(&mut self, subagent_id: &str, result: SubagentResult) -> Result<()> {
        if let Some(subagent) = self.subagents.get_mut(subagent_id) {
            subagent.result = Some(result);
            subagent.state = SubagentState::Completed;
            subagent.completed_at = Some(chrono::Utc::now().to_rfc3339());
            Ok(())
        } else {
            Err(format!("Subagent '{}' not found", subagent_id).into())
        }
    }
    
    /// 设置子代理错误
    pub fn set_error(&mut self, subagent_id: &str, error: String) -> Result<()> {
        if let Some(subagent) = self.subagents.get_mut(subagent_id) {
            subagent.result = Some(SubagentResult {
                success: false,
                output: None,
                error: Some(error),
                tool_usage: HashMap::new(),
                token_usage: TokenUsage::default(),
                duration_ms: 0,
            });
            subagent.state = SubagentState::Failed;
            subagent.completed_at = Some(chrono::Utc::now().to_rfc3339());
            Ok(())
        } else {
            Err(format!("Subagent '{}' not found", subagent_id).into())
        }
    }
    
    /// 清理已完成的子代理
    pub fn cleanup_completed(&mut self) {
        self.subagents.retain(|_, s| {
            !matches!(
                s.state,
                SubagentState::Completed | SubagentState::Failed | SubagentState::Cancelled
            )
        });
    }
}

/// 生成子代理 ID
fn generate_subagent_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("subagent_{}", rng.gen::<u64>())
}

/// 生成会话 ID
fn generate_session_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("session_{}", rng.gen::<u64>())
}

/// 生成消息 ID
fn generate_message_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("msg_{}", rng.gen::<u64>())
}
