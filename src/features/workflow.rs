//! WORKFLOW_SCRIPTS 工作流脚本
//! 
//! 这个模块实现了自动化工作流脚本功能，允许定义和执行复杂的工作流程。

use crate::error::Result;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 工作流状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowState {
    /// 草稿
    Draft,
    
    /// 已启用
    Enabled,
    
    /// 运行中
    Running,
    
    /// 已暂停
    Paused,
    
    /// 已完成
    Completed,
    
    /// 已失败
    Failed,
    
    /// 已禁用
    Disabled,
}

impl Default for WorkflowState {
    fn default() -> Self {
        WorkflowState::Draft
    }
}

/// 步骤状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepState {
    /// 待处理
    Pending,
    
    /// 运行中
    Running,
    
    /// 已完成
    Completed,
    
    /// 已跳过
    Skipped,
    
    /// 已失败
    Failed,
}

impl Default for StepState {
    fn default() -> Self {
        StepState::Pending
    }
}

/// 工作流定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    /// 工作流 ID
    pub id: String,
    
    /// 工作流名称
    pub name: String,
    
    /// 工作流描述
    pub description: String,
    
    /// 工作流版本
    pub version: String,
    
    /// 工作流状态
    pub state: WorkflowState,
    
    /// 工作流步骤
    pub steps: Vec<WorkflowStep>,
    
    /// 变量定义
    pub variables: HashMap<String, String>,
    
    /// 触发器
    pub triggers: Vec<WorkflowTrigger>,
    
    /// 创建时间
    pub created_at: String,
    
    /// 最后更新时间
    pub updated_at: String,
    
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 工作流步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// 步骤 ID
    pub id: String,
    
    /// 步骤名称
    pub name: String,
    
    /// 步骤类型
    pub step_type: StepType,
    
    /// 步骤配置
    pub config: HashMap<String, serde_json::Value>,
    
    /// 依赖的步骤 ID
    pub dependencies: Vec<String>,
    
    /// 步骤状态
    pub state: StepState,
    
    /// 步骤条件
    pub condition: Option<String>,
    
    /// 超时时间（秒）
    pub timeout_seconds: Option<u32>,
    
    /// 重试次数
    pub retry_count: u32,
    
    /// 当前重试
    pub current_retry: u32,
    
    /// 步骤结果
    pub result: Option<StepResult>,
}

/// 步骤类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepType {
    /// 命令执行
    Command,
    
    /// 工具调用
    ToolCall,
    
    /// AI 代理
    Agent,
    
    /// 条件判断
    Condition,
    
    /// 循环
    Loop,
    
    /// 并行执行
    Parallel,
    
    /// 延迟
    Delay,
    
    /// 用户输入
    UserInput,
    
    /// 自定义脚本
    CustomScript,
}

impl Default for StepType {
    fn default() -> Self {
        StepType::Command
    }
}

/// 步骤结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// 是否成功
    pub success: bool,
    
    /// 输出内容
    pub output: Option<String>,
    
    /// 错误信息
    pub error: Option<String>,
    
    /// 输出变量
    pub output_variables: HashMap<String, String>,
    
    /// 执行时间（毫秒）
    pub duration_ms: u64,
}

/// 工作流触发器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTrigger {
    /// 触发器 ID
    pub id: String,
    
    /// 触发器类型
    pub trigger_type: TriggerType,
    
    /// 触发器配置
    pub config: HashMap<String, serde_json::Value>,
    
    /// 是否启用
    pub enabled: bool,
}

/// 触发器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerType {
    /// 手动触发
    Manual,
    
    /// 定时触发
    Scheduled,
    
    /// 文件变化
    FileChange,
    
    /// Git 事件
    GitEvent,
    
    /// Webhook
    Webhook,
    
    /// 命令触发
    Command,
}

impl Default for TriggerType {
    fn default() -> Self {
        TriggerType::Manual
    }
}

/// 工作流执行实例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    /// 执行 ID
    pub execution_id: String,
    
    /// 工作流 ID
    pub workflow_id: String,
    
    /// 工作流版本
    pub workflow_version: String,
    
    /// 执行状态
    pub state: WorkflowState,
    
    /// 步骤执行状态
    pub step_states: HashMap<String, StepState>,
    
    /// 开始时间
    pub started_at: String,
    
    /// 结束时间
    pub ended_at: Option<String>,
    
    /// 输入变量
    pub input_variables: HashMap<String, String>,
    
    /// 输出变量
    pub output_variables: HashMap<String, String>,
    
    /// 执行日志
    pub logs: Vec<WorkflowLogEntry>,
}

/// 工作流日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowLogEntry {
    /// 时间戳
    pub timestamp: String,
    
    /// 日志级别
    pub level: LogLevel,
    
    /// 步骤 ID
    pub step_id: Option<String>,
    
    /// 日志消息
    pub message: String,
}

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    /// 调试
    Debug,
    
    /// 信息
    Info,
    
    /// 警告
    Warning,
    
    /// 错误
    Error,
}

/// 工作流管理器
#[derive(Debug)]
pub struct WorkflowManager {
    /// 应用状态
    state: AppState,
    
    /// 工作流定义
    workflows: HashMap<String, WorkflowDefinition>,
    
    /// 工作流执行
    executions: HashMap<String, WorkflowExecution>,
    
    /// 工作流模板
    templates: HashMap<String, String>,
}

impl WorkflowManager {
    /// 创建新的工作流管理器
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            workflows: HashMap::new(),
            executions: HashMap::new(),
            templates: HashMap::new(),
        }
    }
    
    /// 创建工作流定义
    pub fn create_workflow(
        &mut self,
        name: String,
        description: String,
    ) -> WorkflowDefinition {
        let workflow_id = generate_workflow_id();
        
        let workflow = WorkflowDefinition {
            id: workflow_id.clone(),
            name,
            description,
            version: "1.0.0".to_string(),
            state: WorkflowState::Draft,
            steps: Vec::new(),
            variables: HashMap::new(),
            triggers: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            metadata: HashMap::new(),
        };
        
        self.workflows.insert(workflow_id.clone(), workflow.clone());
        workflow
    }
    
    /// 获取工作流定义
    pub fn get_workflow(&self, workflow_id: &str) -> Option<&WorkflowDefinition> {
        self.workflows.get(workflow_id)
    }
    
    /// 获取可变的工作流定义
    pub fn get_workflow_mut(&mut self, workflow_id: &str) -> Option<&mut WorkflowDefinition> {
        self.workflows.get_mut(workflow_id)
    }
    
    /// 列出所有工作流
    pub fn list_workflows(&self) -> Vec<&WorkflowDefinition> {
        self.workflows.values().collect()
    }
    
    /// 删除工作流
    pub fn delete_workflow(&mut self, workflow_id: &str) -> Result<()> {
        if self.workflows.remove(workflow_id).is_some() {
            Ok(())
        } else {
            Err(format!("Workflow '{}' not found", workflow_id).into())
        }
    }
    
    /// 启用工作流
    pub fn enable_workflow(&mut self, workflow_id: &str) -> Result<()> {
        if let Some(workflow) = self.workflows.get_mut(workflow_id) {
            workflow.state = WorkflowState::Enabled;
            Ok(())
        } else {
            Err(format!("Workflow '{}' not found", workflow_id).into())
        }
    }
    
    /// 禁用工作流
    pub fn disable_workflow(&mut self, workflow_id: &str) -> Result<()> {
        if let Some(workflow) = self.workflows.get_mut(workflow_id) {
            workflow.state = WorkflowState::Disabled;
            Ok(())
        } else {
            Err(format!("Workflow '{}' not found", workflow_id).into())
        }
    }
    
    /// 开始执行工作流
    pub async fn start_execution(
        &mut self,
        workflow_id: &str,
        input_variables: HashMap<String, String>,
    ) -> Result<WorkflowExecution> {
        if let Some(workflow) = self.workflows.get(workflow_id) {
            let execution_id = generate_execution_id();
            
            let mut step_states = HashMap::new();
            for step in &workflow.steps {
                step_states.insert(step.id.clone(), StepState::Pending);
            }
            
            let execution = WorkflowExecution {
                execution_id: execution_id.clone(),
                workflow_id: workflow_id.to_string(),
                workflow_version: workflow.version.clone(),
                state: WorkflowState::Running,
                step_states,
                started_at: chrono::Utc::now().to_rfc3339(),
                ended_at: None,
                input_variables,
                output_variables: HashMap::new(),
                logs: Vec::new(),
            };
            
            self.executions.insert(execution_id.clone(), execution.clone());
            Ok(execution)
        } else {
            Err(format!("Workflow '{}' not found", workflow_id).into())
        }
    }
    
    /// 获取执行
    pub fn get_execution(&self, execution_id: &str) -> Option<&WorkflowExecution> {
        self.executions.get(execution_id)
    }
    
    /// 列出所有执行
    pub fn list_executions(&self) -> Vec<&WorkflowExecution> {
        self.executions.values().collect()
    }
    
    /// 添加工作流模板
    pub fn add_template(&mut self, name: String, template: String) {
        self.templates.insert(name, template);
    }
    
    /// 获取工作流模板
    pub fn get_template(&self, name: &str) -> Option<&String> {
        self.templates.get(name)
    }
    
    /// 创建命令步骤
    pub fn create_command_step(
        name: String,
        command: String,
        dependencies: Vec<String>,
    ) -> WorkflowStep {
        let mut config = HashMap::new();
        config.insert("command".to_string(), command.into());
        
        WorkflowStep {
            id: generate_step_id(),
            name,
            step_type: StepType::Command,
            config,
            dependencies,
            state: StepState::Pending,
            condition: None,
            timeout_seconds: None,
            retry_count: 0,
            current_retry: 0,
            result: None,
        }
    }
    
    /// 创建工具调用步骤
    pub fn create_tool_step(
        name: String,
        tool_name: String,
        tool_params: serde_json::Value,
        dependencies: Vec<String>,
    ) -> WorkflowStep {
        let mut config = HashMap::new();
        config.insert("tool_name".to_string(), tool_name.into());
        config.insert("params".to_string(), tool_params);
        
        WorkflowStep {
            id: generate_step_id(),
            name,
            step_type: StepType::ToolCall,
            config,
            dependencies,
            state: StepState::Pending,
            condition: None,
            timeout_seconds: None,
            retry_count: 0,
            current_retry: 0,
            result: None,
        }
    }
}

/// 生成工作流 ID
fn generate_workflow_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("workflow_{}", rng.gen::<u64>())
}

/// 生成步骤 ID
fn generate_step_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("step_{}", rng.gen::<u64>())
}

/// 生成执行 ID
fn generate_execution_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("execution_{}", rng.gen::<u64>())
}
