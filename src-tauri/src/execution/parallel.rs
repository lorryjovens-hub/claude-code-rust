use crate::execution::cache::ResultCache;
use crate::execution::monitor::ExecutionMonitor;
use crate::execution::types::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore, broadcast};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub struct ParallelExecutionEngine {
    cache: Arc<ResultCache>,
    monitor: Arc<ExecutionMonitor>,
    max_parallelism: usize,
    default_timeout: Duration,
    event_tx: broadcast::Sender<ExecutionEvent>,
    active_executions: Arc<RwLock<HashMap<String, ExecutionContext>>>,
}

impl ParallelExecutionEngine {
    pub fn new(max_parallelism: usize) -> Self {
        let (event_tx, _) = broadcast::channel(10000);

        Self {
            cache: Arc::new(ResultCache::new(1000)),
            monitor: Arc::new(ExecutionMonitor::new()),
            max_parallelism,
            default_timeout: Duration::from_secs(30),
            event_tx,
            active_executions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_cache_capacity(mut self, capacity: usize) -> Self {
        self.cache = Arc::new(ResultCache::new(capacity));
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    pub async fn execute(
        &self,
        tools: Vec<ToolCallRequest>,
    ) -> Result<Vec<ToolCallResult>, ExecutionError> {
        let execution_id = Uuid::new_v4().to_string();

        let plan = self.build_execution_plan(&tools);

        let start = Instant::now();

        let _ = self.event_tx.send(ExecutionEvent::PlanBuilt(plan.clone()));

        let context = ExecutionContext {
            execution_id: execution_id.clone(),
            plan: plan.clone(),
            status: ExecutionStatus::Running,
            started_at: Some(start),
            completed_at: None,
            results: HashMap::new(),
            errors: Vec::new(),
        };

        self.active_executions.write().await
            .insert(execution_id.clone(), context);

        let mut all_results = Vec::new();
        let mut success_count = 0usize;
        let mut fail_count = 0usize;

        for (layer_idx, layer) in plan.layer_indices.iter().enumerate() {
            let layer_start = Instant::now();

            let layer_tools: Vec<&ToolCallRequest> = plan.all_tools.iter()
                .filter(|t| layer.contains(&t.id))
                .collect();

            let _ = self.event_tx.send(ExecutionEvent::LayerStarted {
                layer: layer_idx,
                tool_count: layer_tools.len(),
            });

            let semaphore = Arc::new(Semaphore::new(self.max_parallelism));

            let mut handles = Vec::new();

            for tool in &layer_tools {
                let sem = semaphore.clone();
                let cache = self.cache.clone();
                let monitor = self.monitor.clone();
                let event_tx = self.event_tx.clone();
                let execution_id = execution_id.clone();
                let tool = (*tool).clone();
                let timeout = tool.timeout.unwrap_or(self.default_timeout);

                let handle = tokio::spawn(async move {
                    let _permit = sem.acquire().await.unwrap();

                    let _ = event_tx.send(ExecutionEvent::ToolStarted(tool.id.clone()));

                    if let Some(cached) = cache.get(&tool).await {
                        debug!(target: "execution", "Cache hit for tool: {}", tool.tool_name);
                        let _ = event_tx.send(ExecutionEvent::ToolCompleted(cached.clone()));
                        return (tool.id.clone(), cached);
                    }

                    let result = tokio::time::timeout(
                        timeout,
                        execute_with_retry(&tool, &execution_id, &[]),
                    ).await;

                    let tool_result = match result {
                        Ok(Ok(output)) => {
                            cache.put(tool.clone(), output.clone()).await;
                            let _ = event_tx.send(ExecutionEvent::ToolCompleted(output.clone()));
                            output
                        }
                        Ok(Err(e)) => {
                            let err_result = ToolCallResult {
                                id: tool.id.clone(),
                                success: false,
                                output: serde_json::Value::Null,
                                error: Some(e.to_string()),
                                duration: Duration::ZERO,
                                tool_name: tool.tool_name.clone(),
                                retry_count: 0,
                            };
                            let _ = event_tx.send(ExecutionEvent::ToolFailed {
                                id: tool.id.clone(),
                                error: e.to_string(),
                            });
                            err_result
                        }
                        Err(_) => {
                            let timeout_result = ToolCallResult {
                                id: tool.id.clone(),
                                success: false,
                                output: serde_json::Value::Null,
                                error: Some("Tool execution timeout".into()),
                                duration: timeout,
                                tool_name: tool.tool_name.clone(),
                                retry_count: 0,
                            };
                            let _ = event_tx.send(ExecutionEvent::ToolFailed {
                                id: tool.id.clone(),
                                error: "Timeout".into(),
                            });
                            timeout_result
                        }
                    };

                    monitor.record_tool_execution(&tool_result).await;

                    (tool.id.clone(), tool_result)
                });

                handles.push(handle);
            }

            for handle in handles {
                match handle.await {
                    Ok((_tool_id, result)) => {
                        if result.success {
                            success_count += 1;
                        } else {
                            fail_count += 1;
                        }
                        all_results.push(result);
                    }
                    Err(e) => {
                        error!(target: "execution", "Tool execution task panicked: {}", e);
                        fail_count += 1;
                    }
                }
            }

            let layer_duration = layer_start.elapsed();
            let _ = self.event_tx.send(ExecutionEvent::LayerCompleted {
                layer: layer_idx,
                duration: layer_duration,
            });

            let _ = self.event_tx.send(ExecutionEvent::Progress {
                completed: all_results.len(),
                total: plan.all_tools.len(),
            });
        }

        let total_duration = start.elapsed();

        {
            let mut execs = self.active_executions.write().await;
            if let Some(ctx) = execs.get_mut(&execution_id) {
                ctx.status = if fail_count == 0 {
                    ExecutionStatus::Completed
                } else {
                    ExecutionStatus::Failed(format!("{} tools failed", fail_count))
                };
                ctx.completed_at = Some(Instant::now());
            }
        }

        let _ = self.event_tx.send(ExecutionEvent::AllCompleted {
            total_duration,
            success_count,
            fail_count,
        });

        info!(target: "execution", "Execution complete: success={}, failed={}, duration={:?}",
            success_count, fail_count, total_duration);

        Ok(all_results)
    }

    fn build_execution_plan(&self, tools: &[ToolCallRequest]) -> ExecutionPlan {
        let mut remaining: HashSet<&str> = tools.iter().map(|t| t.id.as_str()).collect();
        let mut executed: HashSet<String> = HashSet::new();
        let mut layers: Vec<Vec<String>> = Vec::new();

        while !remaining.is_empty() {
            let mut current_layer: Vec<String> = Vec::new();

            for tool in tools {
                if !remaining.contains(tool.id.as_str()) {
                    continue;
                }

                let all_deps_ready = tool.dependencies.iter().all(|dep_id| executed.contains(dep_id));

                if all_deps_ready {
                    current_layer.push(tool.id.clone());
                }
            }

            if current_layer.is_empty() {
                let pending: Vec<_> = remaining.iter().map(|s| s.to_string()).collect();
                warn!(target: "execution", "Circular or unsatisfiable dependency detected! Remaining: {:?}", pending);
                for tool in tools {
                    if remaining.contains(tool.id.as_str()) {
                        current_layer.push(tool.id.clone());
                    }
                }
            }

            for id in &current_layer {
                remaining.remove(id.as_str());
                executed.insert(id.clone());
            }

            layers.push(current_layer);
        }

        let estimated_total_time = layers.iter().fold(Duration::ZERO, |acc, layer| {
            let parallel = layer.len().min(self.max_parallelism);
            if parallel > 0 {
                acc + Duration::from_secs(2) * (layer.len() as u32 / parallel as u32)
            } else {
                acc
            }
        });

        ExecutionPlan {
            layer_indices: layers,
            all_tools: tools.to_vec(),
            estimated_total_time,
            max_parallelism: self.max_parallelism,
        }
    }

    pub async fn cancel_execution(&self, execution_id: &str) -> Result<(), String> {
        let mut execs = self.active_executions.write().await;
        if let Some(ctx) = execs.get_mut(execution_id) {
            ctx.status = ExecutionStatus::Cancelled;
            let _ = self.event_tx.send(ExecutionEvent::Cancelled);
            Ok(())
        } else {
            Err(format!("Execution not found: {}", execution_id))
        }
    }

    pub async fn get_execution_status(&self, execution_id: &str) -> Option<ExecutionContext> {
        self.active_executions.read().await
            .get(execution_id)
            .cloned()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ExecutionEvent> {
        self.event_tx.subscribe()
    }

    pub async fn get_statistics(&self) -> EngineStatistics {
        EngineStatistics {
            active_executions: self.active_executions.read().await.len(),
            cache_size: self.cache.size().await,
            total_executions: self.monitor.total_executions().await,
            average_duration: self.monitor.average_duration().await,
            tool_usage: self.monitor
                .tool_usage_stats()
                .into_iter()
                .collect(),
        }
    }

    pub fn clear_cache(&self) {
        tokio::spawn({
            let cache = self.cache.clone();
            async move {
                cache.clear().await;
            }
        });
    }
}

async fn execute_with_retry(
    tool: &ToolCallRequest,
    _execution_id: &str,
    _previous_results: &[ToolCallResult],
) -> Result<ToolCallResult, String> {
    let retry_policy = tool.retry_policy
        .as_ref()
        .cloned()
        .unwrap_or_default();

    let start = Instant::now();

    for attempt in 0..=retry_policy.max_retries {
        if attempt > 0 {
            let delay_ms = (retry_policy.base_delay_ms as f64
                * retry_policy.backoff_factor.powi(attempt as i32)) as u64;
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;

            debug!(target: "execution", "Retrying tool {} attempt {}", tool.tool_name, attempt);
        }

        let result = invoke_tool_handler(tool).await;

        match result {
            Ok(output) => {
                let duration = start.elapsed();
                return Ok(ToolCallResult {
                    id: tool.id.clone(),
                    success: true,
                    output,
                    error: None,
                    duration,
                    tool_name: tool.tool_name.clone(),
                    retry_count: attempt,
                });
            }
            Err(e) => {
                if attempt >= retry_policy.max_retries {
                    let duration = start.elapsed();
                    return Ok(ToolCallResult {
                        id: tool.id.clone(),
                        success: false,
                        output: serde_json::Value::Null,
                        error: Some(format!("Failed after {} retries: {}", attempt, e)),
                        duration,
                        tool_name: tool.tool_name.clone(),
                        retry_count: attempt,
                    });
                }
            }
        }
    }

    let duration = start.elapsed();
    Ok(ToolCallResult {
        id: tool.id.clone(),
        success: false,
        output: serde_json::Value::Null,
        error: Some("All retries exhausted".into()),
        duration,
        tool_name: tool.tool_name.clone(),
        retry_count: retry_policy.max_retries,
    })
}

async fn invoke_tool_handler(tool: &ToolCallRequest) -> Result<serde_json::Value, String> {
    Err(format!("Tool '{}' handler not registered via tool_registry", tool.tool_name))
}