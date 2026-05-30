use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    pub id: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
    pub dependencies: Vec<String>,
    pub priority: u32,
    pub timeout: Option<Duration>,
    pub retry_policy: Option<RetryPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub backoff_factor: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,
            backoff_factor: 2.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub id: String,
    pub tool_name: String,
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub duration: Duration,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionPlan {
    pub layer_indices: Vec<Vec<String>>,
    pub all_tools: Vec<ToolCallRequest>,
    pub estimated_total_time: Duration,
    pub max_parallelism: usize,
}

#[derive(Debug, Clone, Serialize)]
pub enum ExecutionStatus {
    Running,
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionContext {
    pub execution_id: String,
    pub plan: ExecutionPlan,
    pub status: ExecutionStatus,
    #[serde(skip)]
    pub started_at: Option<Instant>,
    #[serde(skip)]
    pub completed_at: Option<Instant>,
    pub results: HashMap<String, ToolCallResult>,
    pub errors: Vec<String>,
}

impl ExecutionContext {
    pub fn elapsed_ms(&self) -> Option<u64> {
        match (&self.started_at, &self.completed_at) {
            (Some(start), Some(end)) => Some((*end - *start).as_millis() as u64),
            (Some(start), None) => Some(start.elapsed().as_millis() as u64),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExecutionEvent {
    PlanBuilt(ExecutionPlan),
    LayerStarted {
        layer: usize,
        tool_count: usize,
    },
    LayerCompleted {
        layer: usize,
        duration: Duration,
    },
    ToolStarted(String),
    ToolCompleted(ToolCallResult),
    ToolFailed {
        id: String,
        error: String,
    },
    Progress {
        completed: usize,
        total: usize,
    },
    AllCompleted {
        total_duration: Duration,
        success_count: usize,
        fail_count: usize,
    },
    Cancelled,
}

#[derive(Debug, Clone, Serialize)]
pub struct EngineStatistics {
    pub active_executions: usize,
    pub cache_size: usize,
    pub total_executions: u64,
    pub average_duration: Duration,
    pub tool_usage: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub struct ExecutionError {
    pub tool_id: String,
    pub error_type: ErrorType,
    pub message: String,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    PlanBuildFailed,
    ExecutionFailed,
    Timeout,
    ToolNotFound,
    Cancelled,
}