//! COORDINATOR_MODE 协调器模式
//! 
//! 这个模块实现了多工作器协调系统，主协调器管理多个子工作器并行执行任务。

use crate::error::Result;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 协调器模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoordinatorMode {
    /// 简单模式（受限工具集）
    Simple,
    
    /// 完整模式（所有工具）
    Full,
}

impl Default for CoordinatorMode {
    fn default() -> Self {
        CoordinatorMode::Simple
    }
}

/// 工作器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerState {
    /// 空闲
    Idle,
    
    /// 分配中
    Assigning,
    
    /// 工作中
    Working,
    
    /// 已完成
    Completed,
    
    /// 错误
    Error,
}

impl Default for WorkerState {
    fn default() -> Self {
        WorkerState::Idle
    }
}

/// 工作器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    /// 工作器名称
    pub name: String,
    
    /// 工具子集
    pub allowed_tools: Vec<String>,
    
    /// 临时目录
    pub scratchpad_dir: Option<String>,
    
    /// 是否隔离
    pub isolated: bool,
    
    /// 最大并行任务数
    pub max_concurrent_tasks: u32,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            name: "worker".to_string(),
            allowed_tools: vec![
                "bash".to_string(),
                "file_read".to_string(),
                "file_edit".to_string(),
            ],
            scratchpad_dir: None,
            isolated: true,
            max_concurrent_tasks: 1,
        }
    }
}

/// 工作器信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerInfo {
    /// 工作器 ID
    pub id: String,
    
    /// 工作器配置
    pub config: WorkerConfig,
    
    /// 工作器状态
    pub state: WorkerState,
    
    /// 当前任务
    pub current_task: Option<CoordinatorTask>,
    
    /// 已完成任务数
    pub completed_tasks: u32,
    
    /// 创建时间
    pub created_at: String,
}

/// 协调器任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinatorTask {
    /// 任务 ID
    pub id: String,
    
    /// 任务描述
    pub description: String,
    
    /// 任务优先级
    pub priority: u8,
    
    /// 依赖任务 ID
    pub dependencies: Vec<String>,
    
    /// 分配的工作器
    pub assigned_worker: Option<String>,
    
    /// 任务状态
    pub state: TaskState,
    
    /// 任务结果
    pub result: Option<TaskResult>,
    
    /// 创建时间
    pub created_at: String,
}

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    /// 待处理
    Pending,
    
    /// 已分配
    Assigned,
    
    /// 执行中
    InProgress,
    
    /// 已完成
    Completed,
    
    /// 已失败
    Failed,
    
    /// 已取消
    Cancelled,
}

impl Default for TaskState {
    fn default() -> Self {
        TaskState::Pending
    }
}

/// 任务结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// 成功
    pub success: bool,
    
    /// 输出
    pub output: Option<String>,
    
    /// 错误
    pub error: Option<String>,
    
    /// 持续时间（毫秒）
    pub duration_ms: u64,
}

/// 协调器配置
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CoordinatorConfig {
    /// 协调器模式
    pub mode: CoordinatorMode,
    
    /// 是否启用
    pub enabled: bool,
    
    /// 最大工作器数
    pub max_workers: u32,
    
    /// 任务超时（分钟）
    pub task_timeout_minutes: u32,
    
    /// 是否自动缩放
    pub auto_scale: bool,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            mode: CoordinatorMode::Simple,
            enabled: false,
            max_workers: 4,
            task_timeout_minutes: 30,
            auto_scale: true,
        }
    }
}

/// 协调器管理器
#[derive(Debug)]
pub struct CoordinatorManager {
    /// 应用状态
    state: AppState,
    
    /// 协调器配置
    config: CoordinatorConfig,
    
    /// 工作器池
    workers: HashMap<String, WorkerInfo>,
    
    /// 任务队列
    task_queue: Vec<CoordinatorTask>,
    
    /// 任务映射
    tasks: HashMap<String, CoordinatorTask>,
}

impl CoordinatorManager {
    /// 创建新的协调器管理器
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            config: CoordinatorConfig::default(),
            workers: HashMap::new(),
            task_queue: Vec::new(),
            tasks: HashMap::new(),
        }
    }
    
    /// 获取配置
    pub fn config(&self) -> &CoordinatorConfig {
        &self.config
    }
    
    /// 启用协调器
    pub fn enable(&mut self) {
        self.config.enabled = true;
    }
    
    /// 禁用协调器
    pub fn disable(&mut self) {
        self.config.enabled = false;
    }
    
    /// 设置模式
    pub fn set_mode(&mut self, mode: CoordinatorMode) {
        self.config.mode = mode;
    }
    
    /// 获取协调器用户上下文
    pub fn get_user_context(&self, scratchpad_dir: Option<String>) -> HashMap<String, String> {
        let worker_tools = match self.config.mode {
            CoordinatorMode::Simple => vec![
                "bash".to_string(),
                "file_read".to_string(),
                "file_edit".to_string(),
            ],
            CoordinatorMode::Full => vec![
                "bash".to_string(),
                "file_read".to_string(),
                "file_edit".to_string(),
                "file_write".to_string(),
                "glob".to_string(),
                "grep".to_string(),
                "git".to_string(),
            ],
        };
        
        let mut context = HashMap::new();
        context.insert("coordinator_mode".to_string(), "true".to_string());
        context.insert("worker_tools".to_string(), worker_tools.join(", "));
        context.insert("scratchpad_dir".to_string(), scratchpad_dir.unwrap_or_default());
        
        context
    }
    
    /// 创建工作器
    pub fn create_worker(&mut self, config: WorkerConfig) -> Result<WorkerInfo> {
        if self.workers.len() >= self.config.max_workers as usize {
            return Err("Maximum number of workers reached".into());
        }
        
        let worker_id = generate_worker_id();
        
        let worker_info = WorkerInfo {
            id: worker_id.clone(),
            config,
            state: WorkerState::Idle,
            current_task: None,
            completed_tasks: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        self.workers.insert(worker_id.clone(), worker_info.clone());
        
        Ok(worker_info)
    }
    
    /// 提交任务
    pub fn submit_task(
        &mut self,
        description: String,
        priority: u8,
        dependencies: Vec<String>,
    ) -> CoordinatorTask {
        let task_id = generate_task_id();
        
        let task = CoordinatorTask {
            id: task_id.clone(),
            description,
            priority: priority.clamp(1, 10),
            dependencies,
            assigned_worker: None,
            state: TaskState::Pending,
            result: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        self.tasks.insert(task_id.clone(), task.clone());
        self.task_queue.push(task.clone());
        
        task
    }
    
    /// 分配任务给空闲工作器
    pub fn assign_tasks(&mut self) -> Vec<(String, String)> {
        let mut assignments = Vec::new();
        
        self.task_queue.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        let idle_workers: Vec<_> = self
            .workers
            .iter()
            .filter(|(_, w)| w.state == WorkerState::Idle)
            .map(|(id, _)| id.clone())
            .collect();
        
        for worker_id in idle_workers {
            if let Some(task_index) = self.task_queue.iter().position(|task| {
                task.state == TaskState::Pending
                    && task
                        .dependencies
                        .iter()
                        .all(|dep_id| self.tasks.get(dep_id).map_or(false, |t| t.state == TaskState::Completed))
            }) {
                let mut task = self.task_queue.remove(task_index);
                task.state = TaskState::Assigned;
                task.assigned_worker = Some(worker_id.clone());
                
                if let Some(worker) = self.workers.get_mut(&worker_id) {
                    worker.state = WorkerState::Assigning;
                    worker.current_task = Some(task.clone());
                }
                
                self.tasks.insert(task.id.clone(), task.clone());
                assignments.push((worker_id, task.id));
            }
        }
        
        assignments
    }
    
    /// 更新任务状态
    pub fn update_task_state(&mut self, task_id: &str, state: TaskState) -> Result<()> {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.state = state;
            
            if let Some(worker_id) = &task.assigned_worker {
                if let Some(worker) = self.workers.get_mut(worker_id) {
                    match state {
                        TaskState::InProgress => worker.state = WorkerState::Working,
                        TaskState::Completed | TaskState::Failed | TaskState::Cancelled => {
                            worker.state = WorkerState::Idle;
                            worker.current_task = None;
                            if state == TaskState::Completed {
                                worker.completed_tasks += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
            
            Ok(())
        } else {
            Err(format!("Task '{}' not found", task_id).into())
        }
    }
    
    /// 设置任务结果
    pub fn set_task_result(&mut self, task_id: &str, result: TaskResult) -> Result<()> {
        let new_state = if let Some(task) = self.tasks.get(task_id) {
            if task.result.as_ref().map_or(false, |r| r.success) {
                TaskState::Completed
            } else {
                TaskState::Failed
            }
        } else {
            return Err(format!("Task '{}' not found", task_id).into());
        };
        
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.result = Some(result);
        }
        
        self.update_task_state(task_id, new_state)?;
        Ok(())
    }
    
    /// 获取工作器
    pub fn get_worker(&self, worker_id: &str) -> Option<&WorkerInfo> {
        self.workers.get(worker_id)
    }
    
    /// 列出所有工作器
    pub fn list_workers(&self) -> Vec<&WorkerInfo> {
        self.workers.values().collect()
    }
    
    /// 获取任务
    pub fn get_task(&self, task_id: &str) -> Option<&CoordinatorTask> {
        self.tasks.get(task_id)
    }
    
    /// 列出所有任务
    pub fn list_tasks(&self) -> Vec<&CoordinatorTask> {
        self.tasks.values().collect()
    }
    
    /// 获取队列中的任务
    pub fn queued_tasks(&self) -> Vec<&CoordinatorTask> {
        self.task_queue.iter().collect()
    }
    
    /// 取消任务
    pub fn cancel_task(&mut self, task_id: &str) -> Result<()> {
        if let Some(task) = self.tasks.get_mut(task_id) {
            if task.state == TaskState::Pending || task.state == TaskState::Assigned {
                task.state = TaskState::Cancelled;
                
                if let Some(worker_id) = &task.assigned_worker {
                    if let Some(worker) = self.workers.get_mut(worker_id) {
                        worker.state = WorkerState::Idle;
                        worker.current_task = None;
                    }
                }
                
                Ok(())
            } else {
                Err("Task cannot be cancelled in current state".into())
            }
        } else {
            Err(format!("Task '{}' not found", task_id).into())
        }
    }
}

/// 生成工作器 ID
fn generate_worker_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("worker_{}", rng.gen::<u64>())
}

/// 生成任务 ID
fn generate_task_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("coord_task_{}", rng.gen::<u64>())
}
