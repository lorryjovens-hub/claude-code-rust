//! 超级规划系统 (ULTRAPLAN)
//! 
//! 这个模块实现了复杂任务的智能规划和执行系统，支持多代理探索和远程执行。

use crate::error::Result;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 规划阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanPhase {
    /// 初始化阶段
    Initialization,
    
    /// 探索阶段
    Exploration,
    
    /// 分析阶段
    Analysis,
    
    /// 计划生成阶段
    PlanGeneration,
    
    /// 用户审核阶段
    UserReview,
    
    /// 执行阶段
    Execution,
    
    /// 完成阶段
    Completed,
    
    /// 失败阶段
    Failed,
}

/// 计划状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanState {
    /// 当前阶段
    pub phase: PlanPhase,
    
    /// 计划开始时间
    pub start_time: String,
    
    /// 最后更新时间
    pub last_update_time: String,
    
    /// 超时时间（分钟）
    pub timeout_minutes: u32,
    
    /// 是否被用户批准
    pub approved: bool,
    
    /// 执行目标位置
    pub execution_target: ExecutionTarget,
    
    /// 进度（0.0-1.0）
    pub progress: f32,
    
    /// 当前步骤
    pub current_step: Option<usize>,
}

impl Default for PlanState {
    fn default() -> Self {
        Self {
            phase: PlanPhase::Initialization,
            start_time: chrono::Utc::now().to_rfc3339(),
            last_update_time: chrono::Utc::now().to_rfc3339(),
            timeout_minutes: 30,
            approved: false,
            execution_target: ExecutionTarget::Local,
            progress: 0.0,
            current_step: None,
        }
    }
}

/// 执行目标位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionTarget {
    /// 本地执行
    Local,
    
    /// 远程执行
    Remote,
}

/// 计划任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanTask {
    /// 任务 ID
    pub id: String,
    
    /// 任务标题
    pub title: String,
    
    /// 任务描述
    pub description: String,
    
    /// 任务优先级
    pub priority: u8,
    
    /// 依赖的任务 ID
    pub dependencies: Vec<String>,
    
    /// 任务状态
    pub status: TaskStatus,
    
    /// 预估时间（分钟）
    pub estimated_minutes: Option<u32>,
    
    /// 实际耗时（分钟）
    pub actual_minutes: Option<u32>,
    
    /// 分配给的代理
    pub assigned_agent: Option<String>,
    
    /// 任务步骤
    pub steps: Vec<TaskStep>,
}

/// 任务步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep {
    /// 步骤描述
    pub description: String,
    
    /// 步骤顺序
    pub order: usize,
    
    /// 步骤状态
    pub status: StepStatus,
    
    /// 步骤结果
    pub result: Option<String>,
    
    /// 错误信息
    pub error: Option<String>,
}

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 待处理
    Pending,
    
    /// 进行中
    InProgress,
    
    /// 已完成
    Completed,
    
    /// 已阻塞
    Blocked,
    
    /// 已失败
    Failed,
    
    /// 已取消
    Cancelled,
}

/// 步骤状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    /// 待处理
    Pending,
    
    /// 进行中
    InProgress,
    
    /// 已完成
    Completed,
    
    /// 已失败
    Failed,
}

/// 探索发现
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationDiscovery {
    /// 发现类型
    pub discovery_type: DiscoveryType,
    
    /// 发现标题
    pub title: String,
    
    /// 详细描述
    pub description: String,
    
    /// 关联文件
    pub file_path: Option<String>,
    
    /// 重要性（1-10）
    pub importance: u8,
    
    /// 发现时间
    pub timestamp: String,
}

/// 发现类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoveryType {
    /// 代码结构
    CodeStructure,
    
    /// 依赖关系
    Dependency,
    
    /// 潜在问题
    PotentialIssue,
    
    /// 最佳实践
    BestPractice,
    
    /// 架构模式
    ArchitecturePattern,
}

/// 超级计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UltraPlan {
    /// 计划 ID
    pub id: String,
    
    /// 用户需求描述
    pub user_requirement: String,
    
    /// 初始种子计划
    pub seed_plan: Option<String>,
    
    /// 计划任务列表
    pub tasks: Vec<PlanTask>,
    
    /// 探索发现
    pub discoveries: Vec<ExplorationDiscovery>,
    
    /// 计划状态
    pub state: PlanState,
    
    /// 计划元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Ultraplan 管理器
#[derive(Debug)]
pub struct UltraPlanManager {
    /// 应用状态
    state: AppState,
    
    /// 当前活动的计划
    active_plan: Option<UltraPlan>,
    
    /// 已完成的计划
    completed_plans: Vec<UltraPlan>,
    
    /// 计划模板
    plan_templates: HashMap<String, String>,
}

impl UltraPlanManager {
    /// 创建新的 Ultraplan 管理器
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            active_plan: None,
            completed_plans: Vec::new(),
            plan_templates: HashMap::new(),
        }
    }
    
    /// 创建新计划
    pub fn create_plan(
        &mut self,
        user_requirement: String,
        seed_plan: Option<String>,
    ) -> UltraPlan {
        let plan_id = generate_plan_id();
        
        let plan = UltraPlan {
            id: plan_id,
            user_requirement,
            seed_plan,
            tasks: Vec::new(),
            discoveries: Vec::new(),
            state: PlanState::default(),
            metadata: HashMap::new(),
        };
        
        self.active_plan = Some(plan.clone());
        plan
    }
    
    /// 获取当前活动计划
    pub fn active_plan(&self) -> Option<&UltraPlan> {
        self.active_plan.as_ref()
    }
    
    /// 获取可变的当前活动计划
    pub fn active_plan_mut(&mut self) -> Option<&mut UltraPlan> {
        self.active_plan.as_mut()
    }
    
    /// 更新计划阶段
    pub fn update_phase(&mut self, phase: PlanPhase) -> Result<()> {
        if let Some(plan) = &mut self.active_plan {
            plan.state.phase = phase;
            plan.state.last_update_time = chrono::Utc::now().to_rfc3339();
            Ok(())
        } else {
            Err("No active plan".into())
        }
    }
    
    /// 添加任务到计划
    pub fn add_task(&mut self, task: PlanTask) -> Result<()> {
        if let Some(plan) = &mut self.active_plan {
            plan.tasks.push(task);
            Ok(())
        } else {
            Err("No active plan".into())
        }
    }
    
    /// 添加探索发现
    pub fn add_discovery(&mut self, discovery: ExplorationDiscovery) -> Result<()> {
        if let Some(plan) = &mut self.active_plan {
            plan.discoveries.push(discovery);
            Ok(())
        } else {
            Err("No active plan".into())
        }
    }
    
    /// 批准计划
    pub fn approve_plan(&mut self) -> Result<()> {
        if let Some(plan) = &mut self.active_plan {
            plan.state.approved = true;
            plan.state.phase = PlanPhase::Execution;
            Ok(())
        } else {
            Err("No active plan".into())
        }
    }
    
    /// 拒绝计划
    pub fn reject_plan(&mut self) -> Result<()> {
        if let Some(plan) = &mut self.active_plan {
            plan.state.approved = false;
            plan.state.phase = PlanPhase::PlanGeneration;
            Ok(())
        } else {
            Err("No active plan".into())
        }
    }
    
    /// 设置执行目标
    pub fn set_execution_target(&mut self, target: ExecutionTarget) -> Result<()> {
        if let Some(plan) = &mut self.active_plan {
            plan.state.execution_target = target;
            Ok(())
        } else {
            Err("No active plan".into())
        }
    }
    
    /// 更新进度
    pub fn update_progress(&mut self, progress: f32) -> Result<()> {
        if let Some(plan) = &mut self.active_plan {
            plan.state.progress = progress.clamp(0.0, 1.0);
            Ok(())
        } else {
            Err("No active plan".into())
        }
    }
    
    /// 标记计划为完成
    pub fn complete_plan(&mut self) -> Result<()> {
        if let Some(mut plan) = self.active_plan.take() {
            plan.state.phase = PlanPhase::Completed;
            plan.state.progress = 1.0;
            self.completed_plans.push(plan);
            Ok(())
        } else {
            Err("No active plan".into())
        }
    }
    
    /// 标记计划为失败
    pub fn fail_plan(&mut self, error: String) -> Result<()> {
        if let Some(mut plan) = self.active_plan.take() {
            plan.state.phase = PlanPhase::Failed;
            plan.metadata.insert("error".to_string(), error.into());
            self.completed_plans.push(plan);
            Ok(())
        } else {
            Err("No active plan".into())
        }
    }
    
    /// 获取已完成的计划
    pub fn completed_plans(&self) -> &[UltraPlan] {
        &self.completed_plans
    }
    
    /// 添加计划模板
    pub fn add_plan_template(&mut self, name: String, template: String) {
        self.plan_templates.insert(name, template);
    }
    
    /// 获取计划模板
    pub fn get_plan_template(&self, name: &str) -> Option<&String> {
        self.plan_templates.get(name)
    }
    
    /// 生成计划提示
    pub fn build_plan_prompt(&self, blurb: &str, seed_plan: Option<&str>) -> String {
        let mut parts = Vec::new();
        
        if let Some(seed) = seed_plan {
            parts.push("Here is a draft plan to refine:".to_string());
            parts.push(String::new());
            parts.push(seed.to_string());
            parts.push(String::new());
        }
        
        parts.push(ULTRAPLAN_INSTRUCTIONS.to_string());
        
        if !blurb.is_empty() {
            parts.push(String::new());
            parts.push(blurb.to_string());
        }
        
        parts.join("\n")
    }
}

/// Ultraplan 指令（占位符）
const ULTRAPLAN_INSTRUCTIONS: &str = r#"
# Ultraplan Instructions

This is a placeholder for the actual Ultraplan instructions.

The real implementation would include detailed instructions for:
1. Analyzing the codebase
2. Generating a comprehensive plan
3. Breaking down tasks
4. Identifying dependencies
5. Estimating timelines
"#;

/// 生成计划 ID
fn generate_plan_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("plan_{}", rng.gen::<u64>())
}

/// 创建代码结构发现
pub fn create_code_structure_discovery(
    title: String,
    description: String,
    file_path: Option<String>,
    importance: u8,
) -> ExplorationDiscovery {
    ExplorationDiscovery {
        discovery_type: DiscoveryType::CodeStructure,
        title,
        description,
        file_path,
        importance: importance.clamp(1, 10),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }
}

/// 创建计划任务
pub fn create_plan_task(
    title: String,
    description: String,
    priority: u8,
    steps: Vec<String>,
) -> PlanTask {
    let task_steps = steps
        .into_iter()
        .enumerate()
        .map(|(i, desc)| TaskStep {
            description: desc,
            order: i,
            status: StepStatus::Pending,
            result: None,
            error: None,
        })
        .collect();
    
    PlanTask {
        id: generate_task_id(),
        title,
        description,
        priority: priority.clamp(1, 10),
        dependencies: Vec::new(),
        status: TaskStatus::Pending,
        estimated_minutes: None,
        actual_minutes: None,
        assigned_agent: None,
        steps: task_steps,
    }
}

/// 生成任务 ID
fn generate_task_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("task_{}", rng.gen::<u64>())
}
