//! Task Management Tool
//!
//! Manage tasks with operations including:
//! - create: Create a new task
//! - update: Update an existing task
//! - delete: Delete a task
//! - list: List all tasks
//! - complete: Mark a task as completed
//! - get: Get task details

use super::{Tool, ToolOutput, ToolError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub subject: String,
    pub description: String,
    pub status: TaskStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub tags: Vec<String>,
    pub priority: TaskPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

pub struct TaskManagementTool {
    tasks: Arc<RwLock<HashMap<String, Task>>>,
}

impl Default for TaskManagementTool {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskManagementTool {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn generate_id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }

    async fn create_task(&self, input: &serde_json::Value) -> Result<Task, ToolError> {
        let subject = input["subject"].as_str()
            .ok_or_else(|| ToolError {
                message: "subject is required".to_string(),
                code: Some("missing_subject".to_string()),
            })?;

        let description = input["description"].as_str()
            .ok_or_else(|| ToolError {
                message: "description is required".to_string(),
                code: Some("missing_description".to_string()),
            })?;

        let priority = input["priority"].as_str()
            .unwrap_or("medium")
            .parse::<TaskPriority>()
            .map_err(|_| ToolError {
                message: "Invalid priority. Must be low, medium, high, or critical".to_string(),
                code: Some("invalid_priority".to_string()),
            })?;

        let tags: Vec<String> = input["tags"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        let metadata: HashMap<String, serde_json::Value> = input["metadata"]
            .as_object()
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default();

        let now = chrono::Utc::now();
        let task = Task {
            id: self.generate_id(),
            subject: subject.to_string(),
            description: description.to_string(),
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
            metadata,
            tags,
            priority,
        };

        Ok(task)
    }

    async fn update_task(&self, task_id: &str, input: &serde_json::Value) -> Result<Task, ToolError> {
        let mut tasks = self.tasks.write().await;
        let task = tasks.get_mut(task_id)
            .ok_or_else(|| ToolError {
                message: format!("Task not found: {}", task_id),
                code: Some("task_not_found".to_string()),
            })?;

        if let Some(subject) = input["subject"].as_str() {
            task.subject = subject.to_string();
        }

        if let Some(description) = input["description"].as_str() {
            task.description = description.to_string();
        }

        if let Some(status_str) = input["status"].as_str() {
            task.status = status_str.parse::<TaskStatus>()
                .map_err(|_| ToolError {
                    message: "Invalid status. Must be pending, in_progress, completed, or deleted".to_string(),
                    code: Some("invalid_status".to_string()),
                })?;
        }

        if let Some(priority_str) = input["priority"].as_str() {
            task.priority = priority_str.parse::<TaskPriority>()
                .map_err(|_| ToolError {
                    message: "Invalid priority. Must be low, medium, high, or critical".to_string(),
                    code: Some("invalid_priority".to_string()),
                })?;
        }

        if let Some(tags_array) = input["tags"].as_array() {
            task.tags = tags_array.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
        }

        if let Some(metadata_obj) = input["metadata"].as_object() {
            for (key, value) in metadata_obj {
                task.metadata.insert(key.clone(), value.clone());
            }
        }

        task.updated_at = chrono::Utc::now();

        Ok(task.clone())
    }
}

#[async_trait]
impl Tool for TaskManagementTool {
    fn name(&self) -> &str {
        "task_management"
    }

    fn description(&self) -> &str {
        "Manage tasks with create, update, delete, list, and complete operations"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "description": "Task operation: create, update, delete, list, complete, get",
                    "enum": ["create", "update", "delete", "list", "complete", "get"]
                },
                "task_id": {
                    "type": "string",
                    "description": "Task ID (required for update, delete, complete, get)"
                },
                "subject": {
                    "type": "string",
                    "description": "Task subject/title (required for create)"
                },
                "description": {
                    "type": "string",
                    "description": "Task description (required for create)"
                },
                "status": {
                    "type": "string",
                    "description": "Task status: pending, in_progress, completed, deleted",
                    "enum": ["pending", "in_progress", "completed", "deleted"]
                },
                "priority": {
                    "type": "string",
                    "description": "Task priority: low, medium, high, critical",
                    "enum": ["low", "medium", "high", "critical"]
                },
                "tags": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "Tags for the task"
                },
                "metadata": {
                    "type": "object",
                    "description": "Additional metadata for the task"
                },
                "filter": {
                    "type": "object",
                    "description": "Filter criteria for listing tasks",
                    "properties": {
                        "status": {
                            "type": "string",
                            "description": "Filter by status"
                        },
                        "priority": {
                            "type": "string",
                            "description": "Filter by priority"
                        },
                        "tags": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "Filter by tags"
                        }
                    }
                }
            },
            "required": ["operation"]
        })
    }

    async fn execute(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let operation = input["operation"].as_str()
            .ok_or_else(|| ToolError {
                message: "operation is required".to_string(),
                code: Some("missing_parameter".to_string()),
            })?;

        match operation {
            "create" => self.handle_create(input).await,
            "update" => self.handle_update(input).await,
            "delete" => self.handle_delete(input).await,
            "list" => self.handle_list(input).await,
            "complete" => self.handle_complete(input).await,
            "get" => self.handle_get(input).await,
            _ => Err(ToolError {
                message: format!("Unknown task operation: {}", operation),
                code: Some("invalid_operation".to_string()),
            }),
        }
    }
}

impl TaskManagementTool {
    async fn handle_create(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let task = self.create_task(&input).await?;
        let task_id = task.id.clone();

        let mut tasks = self.tasks.write().await;
        tasks.insert(task_id.clone(), task);

        Ok(ToolOutput {
            output_type: "json".to_string(),
            content: serde_json::json!({
                "success": true,
                "message": "Task created successfully",
                "task_id": task_id
            }).to_string(),
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn handle_update(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let task_id = input["task_id"].as_str()
            .ok_or_else(|| ToolError {
                message: "task_id is required for update".to_string(),
                code: Some("missing_task_id".to_string()),
            })?;

        let task = self.update_task(task_id, &input).await?;

        Ok(ToolOutput {
            output_type: "json".to_string(),
            content: serde_json::json!({
                "success": true,
                "message": "Task updated successfully",
                "task": task
            }).to_string(),
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn handle_delete(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let task_id = input["task_id"].as_str()
            .ok_or_else(|| ToolError {
                message: "task_id is required for delete".to_string(),
                code: Some("missing_task_id".to_string()),
            })?;

        let mut tasks = self.tasks.write().await;
        let removed = tasks.remove(task_id);

        if removed.is_some() {
            Ok(ToolOutput {
                output_type: "json".to_string(),
                content: serde_json::json!({
                    "success": true,
                    "message": "Task deleted successfully"
                }).to_string(),
                metadata: std::collections::HashMap::new(),
            })
        } else {
            Err(ToolError {
                message: format!("Task not found: {}", task_id),
                code: Some("task_not_found".to_string()),
            })
        }
    }

    async fn handle_list(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let tasks = self.tasks.read().await;
        let filter = &input["filter"];

        let filtered_tasks: Vec<&Task> = tasks.values()
            .filter(|task| {
                if let Some(status_filter) = filter["status"].as_str() {
                    let task_status = match task.status {
                        TaskStatus::Pending => "pending",
                        TaskStatus::InProgress => "in_progress",
                        TaskStatus::Completed => "completed",
                        TaskStatus::Deleted => "deleted",
                    };
                    if task_status != status_filter {
                        return false;
                    }
                }

                if let Some(priority_filter) = filter["priority"].as_str() {
                    let task_priority = match task.priority {
                        TaskPriority::Low => "low",
                        TaskPriority::Medium => "medium",
                        TaskPriority::High => "high",
                        TaskPriority::Critical => "critical",
                    };
                    if task_priority != priority_filter {
                        return false;
                    }
                }

                if let Some(tags_filter) = filter["tags"].as_array() {
                    let filter_tags: Vec<String> = tags_filter.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                    if !filter_tags.iter().all(|tag| task.tags.contains(tag)) {
                        return false;
                    }
                }

                true
            })
            .collect();

        Ok(ToolOutput {
            output_type: "json".to_string(),
            content: serde_json::json!({
                "success": true,
                "tasks": filtered_tasks,
                "count": filtered_tasks.len()
            }).to_string(),
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn handle_complete(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let task_id = input["task_id"].as_str()
            .ok_or_else(|| ToolError {
                message: "task_id is required for complete".to_string(),
                code: Some("missing_task_id".to_string()),
            })?;

        let mut tasks = self.tasks.write().await;
        let task = tasks.get_mut(task_id)
            .ok_or_else(|| ToolError {
                message: format!("Task not found: {}", task_id),
                code: Some("task_not_found".to_string()),
            })?;

        task.status = TaskStatus::Completed;
        task.updated_at = chrono::Utc::now();

        Ok(ToolOutput {
            output_type: "json".to_string(),
            content: serde_json::json!({
                "success": true,
                "message": "Task marked as completed",
                "task": task
            }).to_string(),
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn handle_get(&self, input: serde_json::Value) -> Result<ToolOutput, ToolError> {
        let task_id = input["task_id"].as_str()
            .ok_or_else(|| ToolError {
                message: "task_id is required for get".to_string(),
                code: Some("missing_task_id".to_string()),
            })?;

        let tasks = self.tasks.read().await;
        let task = tasks.get(task_id)
            .ok_or_else(|| ToolError {
                message: format!("Task not found: {}", task_id),
                code: Some("task_not_found".to_string()),
            })?;

        Ok(ToolOutput {
            output_type: "json".to_string(),
            content: serde_json::json!({
                "success": true,
                "task": task
            }).to_string(),
            metadata: std::collections::HashMap::new(),
        })
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(TaskStatus::Pending),
            "in_progress" => Ok(TaskStatus::InProgress),
            "completed" => Ok(TaskStatus::Completed),
            "deleted" => Ok(TaskStatus::Deleted),
            _ => Err(()),
        }
    }
}

impl std::str::FromStr for TaskPriority {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(TaskPriority::Low),
            "medium" => Ok(TaskPriority::Medium),
            "high" => Ok(TaskPriority::High),
            "critical" => Ok(TaskPriority::Critical),
            _ => Err(()),
        }
    }
}