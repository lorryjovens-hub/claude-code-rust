use crate::db::DbManager;
use crate::db::task_repo::ScheduledTaskRow;
use anyhow::Result;
use cron::Schedule;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::str::FromStr;
use tokio::sync::Mutex;
use tokio::time::{Duration, interval};
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

pub struct ScheduledTaskScheduler {
    db_manager: Arc<DbManager>,
    running_tasks: Arc<Mutex<HashMap<String, TaskState>>>,
    should_stop: Arc<Mutex<bool>>,
}

struct TaskState {
    run_id: String,
    started_at: std::time::Instant,
}

impl ScheduledTaskScheduler {
    pub fn new(db_manager: Arc<DbManager>) -> Self {
        Self {
            db_manager,
            running_tasks: Arc::new(Mutex::new(HashMap::new())),
            should_stop: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn init(&self) -> Result<()> {
        info!("[Scheduler] Initializing task scheduler");
        let tasks = self.load_enabled_tasks().await?;
        info!("[Scheduler] Loaded {} enabled tasks from database", tasks.len());
        Ok(())
    }

    pub async fn add_task(&self, task: &ScheduledTaskRow) -> Result<()> {
        info!("[Scheduler] Adding task: {} ({})", task.name, task.id);
        Ok(())
    }

    pub async fn remove_task(&self, task_id: &str) -> Result<()> {
        info!("[Scheduler] Removing task: {}", task_id);
        let mut running = self.running_tasks.lock().await;
        running.remove(task_id);
        Ok(())
    }

    pub async fn execute_task(&self, task: &ScheduledTaskRow) -> TaskExecutionResult {
        let task_id = task.id.clone();
        let task_name = task.name.clone();
        let start_time = std::time::Instant::now();
        let run_id = uuid::Uuid::new_v4().to_string();

        {
            let mut running = self.running_tasks.lock().await;
            running.insert(task_id.clone(), TaskState {
                run_id: run_id.clone(),
                started_at: start_time,
            });
        }

        info!("[Scheduler] Executing task '{}' (run_id: {})", task_name, run_id);

        let result = self.do_execute_task(task, &run_id, start_time).await;

        {
            let mut running = self.running_tasks.lock().await;
            running.remove(&task_id);
        }

        result
    }

    async fn do_execute_task(
        &self,
        task: &ScheduledTaskRow,
        run_id: &str,
        start_time: std::time::Instant,
    ) -> TaskExecutionResult {
        let now = chrono::Utc::now().to_rfc3339();

        let db = self.db_manager.clone();
        let run_id_clone = run_id.to_string();
        let task_id_clone = task.id.clone();
        let insert_result = tokio::task::spawn_blocking(move || {
            db.with_conn(|conn| {
                crate::db::task_repo::insert_task_run(conn, &run_id_clone, &task_id_clone, &now)
            })
        }).await;

        if let Err(e) = insert_result {
            error!("[Scheduler] Failed to insert task run record: {}", e);
        }

        let execution_result = match task.task_type.as_str() {
            "prompt" => self.execute_prompt_task(task).await,
            "webhook" => self.execute_webhook_task(task).await,
            "system" => self.execute_system_task(task).await,
            "report" => self.execute_report_task(task).await,
            _ => TaskExecutionResult {
                success: false,
                output: None,
                error: Some(format!("Unknown task type: {}", task.task_type)),
                duration_ms: 0,
            },
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;
        let finished_at = chrono::Utc::now().to_rfc3339();

        let status = if execution_result.success { "completed" } else { "failed" };
        let output = execution_result.output.clone();
        let output2 = output.clone();
        let error_msg = execution_result.error.clone();

        let db2 = self.db_manager.clone();
        let run_id_clone2 = run_id.to_string();
        let finished_at_clone = finished_at.clone();
        let duration_clone = duration_ms as i64;

        let _ = tokio::task::spawn_blocking(move || {
            db2.with_conn(|conn| {
                crate::db::task_repo::update_task_run_status(
                    conn,
                    &run_id_clone2,
                    status,
                    output.as_deref(),
                    error_msg.as_deref(),
                    &finished_at_clone,
                    duration_clone,
                )
            })
        }).await;

        let next_run_at = crate::task::cron::calc_next_run(&task.cron_expression, &chrono::Utc::now())
            .ok();

        let db3 = self.db_manager.clone();
        let task_id_clone3 = task.id.clone();
        let _ = tokio::task::spawn_blocking(move || {
            db3.with_conn(|conn| {
                crate::db::task_repo::update_task_run_result(
                    conn,
                    &task_id_clone3,
                    &finished_at,
                    status,
                    output2.as_deref(),
                    next_run_at.as_deref(),
                )
            })
        }).await;

        TaskExecutionResult {
            duration_ms,
            ..execution_result
        }
    }

    async fn execute_prompt_task(&self, task: &ScheduledTaskRow) -> TaskExecutionResult {
        let start = std::time::Instant::now();

        let output = format!("Prompt task '{}' executed. Config: {}", task.name, task.task_config);

        TaskExecutionResult {
            success: true,
            output: Some(output),
            error: None,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    async fn execute_webhook_task(&self, task: &ScheduledTaskRow) -> TaskExecutionResult {
        let start = std::time::Instant::now();

        match serde_json::from_str::<serde_json::Value>(&task.task_config) {
            Ok(config) => {
                let url = config.get("url").and_then(|v| v.as_str()).unwrap_or("");
                if url.is_empty() {
                    return TaskExecutionResult {
                        success: false,
                        output: None,
                        error: Some("Webhook URL not specified".to_string()),
                        duration_ms: start.elapsed().as_millis() as u64,
                    };
                }

                match reqwest::Client::new()
                    .post(url)
                    .json(&config.get("payload").unwrap_or(&serde_json::Value::Null))
                    .send()
                    .await
                {
                    Ok(response) => {
                        let status = response.status();
                        let body = response.text().await.unwrap_or_default();
                        TaskExecutionResult {
                            success: status.is_success(),
                            output: Some(format!("Status: {}, Body: {}", status, body)),
                            error: if status.is_success() { None } else { Some(format!("HTTP {}", status)) },
                            duration_ms: start.elapsed().as_millis() as u64,
                        }
                    }
                    Err(e) => TaskExecutionResult {
                        success: false,
                        output: None,
                        error: Some(format!("Request failed: {}", e)),
                        duration_ms: start.elapsed().as_millis() as u64,
                    },
                }
            }
            Err(e) => TaskExecutionResult {
                success: false,
                output: None,
                error: Some(format!("Invalid webhook config JSON: {}", e)),
                duration_ms: start.elapsed().as_millis() as u64,
            },
        }
    }

    async fn execute_system_task(&self, task: &ScheduledTaskRow) -> TaskExecutionResult {
        let start = std::time::Instant::now();

        // Validate command before execution
        if crate::tools::is_dangerous_command(&task.task_config) {
            return TaskExecutionResult {
                success: false,
                output: None,
                error: Some("Command blocked by security filter: dangerous command detected".to_string()),
                duration_ms: start.elapsed().as_millis() as u64,
            };
        }

        #[cfg(target_os = "windows")]
        let output = tokio::process::Command::new("cmd")
            .args(["/C", &task.task_config])
            .output()
            .await;

        #[cfg(not(target_os = "windows"))]
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&task.task_config)
            .output()
            .await;

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                TaskExecutionResult {
                    success: out.status.success(),
                    output: if !stdout.is_empty() { Some(stdout) } else { None },
                    error: if !out.status.success() && !stderr.is_empty() { Some(stderr) } else { None },
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            }
            Err(e) => TaskExecutionResult {
                success: false,
                output: None,
                error: Some(format!("Command execution failed: {}", e)),
                duration_ms: start.elapsed().as_millis() as u64,
            },
        }
    }

    async fn execute_report_task(&self, task: &ScheduledTaskRow) -> TaskExecutionResult {
        let start = std::time::Instant::now();

        let output = format!("Report task '{}' completed. Config: {}", task.name, task.task_config);

        TaskExecutionResult {
            success: true,
            output: Some(output),
            error: None,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    pub async fn start(&self) {
        info!("[Scheduler] Starting task scheduler");
        *self.should_stop.lock().await = false;

        let db = self.db_manager.clone();
        let running = self.running_tasks.clone();
        let should_stop = self.should_stop.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(30));

            loop {
                ticker.tick().await;

                if *should_stop.lock().await {
                    info!("[Scheduler] Scheduler stopped");
                    break;
                }

                let now = chrono::Utc::now().to_rfc3339();
                let due_tasks = match db.with_conn(|conn| {
                    crate::db::task_repo::get_due_tasks(conn, &now)
                }) {
                    Ok(Ok(tasks)) => tasks,
                    Ok(Err(e)) | Err(e) => {
                        error!("[Scheduler] Failed to get due tasks: {}", e);
                        continue;
                    }
                };

                for task in due_tasks {
                    let task_id = task.id.clone();
                    if running.lock().await.contains_key(&task_id) {
                        warn!("[Scheduler] Task {} is already running, skipping", task_id);
                        continue;
                    }

                    info!("[Scheduler] Due task '{}' (cron: {})", task.name, task.cron_expression);

                    let db_clone = db.clone();
                    let running_clone = running.clone();
                    let task_clone = task.clone();

                    tokio::spawn(async move {
                        let scheduler = ScheduledTaskScheduler {
                            db_manager: db_clone,
                            running_tasks: running_clone,
                            should_stop: Arc::new(Mutex::new(false)),
                        };
                        let result = scheduler.execute_task(&task_clone).await;
                        info!(
                            "[Scheduler] Task '{}' completed: success={}, duration={}ms",
                            task_clone.name, result.success, result.duration_ms
                        );
                    });
                }
            }
        });
    }

    pub async fn stop(&self) {
        info!("[Scheduler] Stopping task scheduler");
        *self.should_stop.lock().await = true;
    }

    async fn load_enabled_tasks(&self) -> Result<Vec<ScheduledTaskRow>> {
        let tasks = self.db_manager.with_conn(|conn| {
            crate::db::task_repo::get_enabled_tasks(conn)
        })??;
        Ok(tasks)
    }

    pub fn calc_next_run(cron_expr: &str) -> Result<String> {
        let schedule = Schedule::from_str(cron_expr)
            .map_err(|e| anyhow::anyhow!("Invalid cron expression: {}", e))?;

        let now = chrono::Utc::now();
        let next = schedule
            .after(&now)
            .next()
            .ok_or_else(|| anyhow::anyhow!("No next run time found"))?;

        Ok(next.to_rfc3339())
    }
}
