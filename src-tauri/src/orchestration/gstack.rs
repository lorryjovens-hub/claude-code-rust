use crate::native_engine::anthropic_client::{AnthropicClient, AnthropicContent, AnthropicMessage};
use crate::native_engine::openai_client::{OpenAIClient, OpenAIContent, OpenAIMessage};
use crate::native_engine::provider_manager::{ApiFormat, ResolvedProvider};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::AgentRole;
use super::RequirementsAnalysis;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPlan {
    pub id: String,
    pub objective: String,
    pub tasks: Vec<TaskDefinition>,
    pub phases: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub agent_role: AgentRole,
    pub dependencies: Vec<String>,
    pub phase: String,
    pub estimated_duration_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprint {
    pub id: String,
    pub name: String,
    pub start_date: String,
    pub end_date: String,
    pub tasks: Vec<String>,
    pub completed_tasks: Vec<String>,
}

#[derive(Clone)]
pub struct GStackManager {
    plans_dir: PathBuf,
    plans: HashMap<String, WorkPlan>,
}

impl GStackManager {
    pub fn new(data_dir: &Path) -> Self {
        let plans_dir = data_dir.join("workplans");
        if !plans_dir.exists() {
            let _ = fs::create_dir_all(&plans_dir);
        }
        
        let mut manager = Self {
            plans_dir,
            plans: HashMap::new(),
        };
        
        manager.load_plans();
        manager
    }

    fn load_plans(&mut self) {
        if !self.plans_dir.exists() {
            return;
        }
        
        let entries = match fs::read_dir(&self.plans_dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Some(id) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(plan) = serde_json::from_str::<WorkPlan>(&content) {
                                self.plans.insert(id.to_string(), plan);
                            }
                        }
                    }
                }
            }
        }
    }

    pub async fn generate_workplan(
        &self,
        requirements: &RequirementsAnalysis,
        provider: &ResolvedProvider,
    ) -> Result<WorkPlan> {
        let system_prompt = r#"You are a senior Project Manager AI agent specialized in creating detailed work plans.

Given requirements analysis, generate a comprehensive work plan with tasks.

Output ONLY valid JSON in this exact format:
{
  "tasks": [
    {
      "id": "TASK-001",
      "name": "Task name",
      "description": "Detailed task description",
      "agent_role": "ProductManager|Architect|Developer|Reviewer|DevOps|Analyst|Designer",
      "dependencies": ["TASK-002", "TASK-003"],
      "phase": "planning|design|implementation|review|deployment",
      "estimated_duration_minutes": 60
    }
  ],
  "phases": ["planning", "design", "implementation", "review", "deployment"]
}

Guidelines:
- Create 5-10 tasks that cover all requirements
- Define clear dependencies between tasks
- Assign appropriate agent roles
- Group tasks into logical phases
- Estimate durations realistically
- Ensure no circular dependencies"#;

        let requirements_json = serde_json::to_string(requirements)?;
        let user_message = format!(
            "Based on the following requirements analysis, generate a detailed work plan:\n\n{}",
            requirements_json
        );

        let response_text = match provider.provider.api_format {
            ApiFormat::Anthropic => self.call_anthropic(provider, &system_prompt, &user_message).await?,
            ApiFormat::OpenAI => self.call_openai(provider, &system_prompt, &user_message).await?,
        };

        let json_str = response_text
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        match serde_json::from_str::<serde_json::Value>(json_str) {
            Ok(parsed) => {
                let mut tasks: Vec<TaskDefinition> = serde_json::from_value(
                    parsed.get("tasks").cloned().unwrap_or_default()
                ).unwrap_or_default();
                
                let phases: Vec<String> = serde_json::from_value(
                    parsed.get("phases").cloned().unwrap_or_default()
                ).unwrap_or_else(|_| vec!["planning".to_string(), "design".to_string(), "implementation".to_string(), "review".to_string(), "deployment".to_string()]);

                tasks = self.post_process_tasks(tasks);

                let plan = WorkPlan {
                    id: format!("plan-{}", uuid::Uuid::new_v4()),
                    objective: requirements.goal.clone(),
                    tasks,
                    phases,
                    created_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default()
                        .as_millis() as u64,
                    updated_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default()
                        .as_millis() as u64,
                };

                Ok(plan)
            }
            Err(e) => {
                tracing::warn!(module = "GStack", "Failed to parse workplan: {}", e);
                Ok(self.generate_fallback_workplan(requirements))
            }
        }
    }

    fn post_process_tasks(&self, mut tasks: Vec<TaskDefinition>) -> Vec<TaskDefinition> {
        let task_ids: Vec<String> = tasks.iter().map(|t| t.id.clone()).collect();
        
        for task in &mut tasks {
            task.dependencies.retain(|dep| task_ids.contains(dep));
            
            if task.estimated_duration_minutes == 0 {
                task.estimated_duration_minutes = 60;
            }
        }
        
        tasks
    }

    fn generate_fallback_workplan(&self, requirements: &RequirementsAnalysis) -> WorkPlan {
        WorkPlan {
            id: format!("plan-{}", uuid::Uuid::new_v4()),
            objective: requirements.goal.clone(),
            tasks: vec![
                TaskDefinition {
                    id: "TASK-001".to_string(),
                    name: "Analyze Requirements".to_string(),
                    description: "Analyze and understand all requirements".to_string(),
                    agent_role: AgentRole::ProductManager,
                    dependencies: Vec::new(),
                    phase: "planning".to_string(),
                    estimated_duration_minutes: 120,
                },
                TaskDefinition {
                    id: "TASK-002".to_string(),
                    name: "Design Architecture".to_string(),
                    description: "Design system architecture based on requirements".to_string(),
                    agent_role: AgentRole::Architect,
                    dependencies: vec!["TASK-001".to_string()],
                    phase: "design".to_string(),
                    estimated_duration_minutes: 180,
                },
                TaskDefinition {
                    id: "TASK-003".to_string(),
                    name: "Implement Core Features".to_string(),
                    description: "Implement main features according to design".to_string(),
                    agent_role: AgentRole::Developer,
                    dependencies: vec!["TASK-002".to_string()],
                    phase: "implementation".to_string(),
                    estimated_duration_minutes: 480,
                },
                TaskDefinition {
                    id: "TASK-004".to_string(),
                    name: "Code Review".to_string(),
                    description: "Review implementation for quality and security".to_string(),
                    agent_role: AgentRole::Reviewer,
                    dependencies: vec!["TASK-003".to_string()],
                    phase: "review".to_string(),
                    estimated_duration_minutes: 120,
                },
                TaskDefinition {
                    id: "TASK-005".to_string(),
                    name: "Deploy".to_string(),
                    description: "Deploy to production environment".to_string(),
                    agent_role: AgentRole::DevOps,
                    dependencies: vec!["TASK-004".to_string()],
                    phase: "deployment".to_string(),
                    estimated_duration_minutes: 60,
                },
            ],
            phases: vec!["planning".to_string(), "design".to_string(), "implementation".to_string(), "review".to_string(), "deployment".to_string()],
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default()
                .as_millis() as u64,
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default()
                .as_millis() as u64,
        }
    }

    async fn call_anthropic(&self, provider: &ResolvedProvider, system_prompt: &str, user_message: &str) -> Result<String> {
        let client = AnthropicClient::new();
        let messages = vec![AnthropicMessage {
            role: "user".to_string(),
            content: AnthropicContent::Text(user_message.to_string()),
        }];
        let response = client
            .send_message(provider, messages, Some(system_prompt), vec![], 8192)
            .await?;
        let text = response
            .content
            .iter()
            .filter_map(|block| match block {
                crate::native_engine::anthropic_client::ContentBlock::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");
        Ok(text)
    }

    async fn call_openai(&self, provider: &ResolvedProvider, system_prompt: &str, user_message: &str) -> Result<String> {
        let client = OpenAIClient::new();
        let messages = vec![OpenAIMessage {
            role: "user".to_string(),
            content: OpenAIContent::Text(user_message.to_string()),
            tool_calls: None,
            tool_call_id: None,
            reasoning_content: None,
        }];
        let response = client
            .send_message(provider, messages, Some(system_prompt), vec![], 8192)
            .await?;
        let text = response
            .choices
            .first()
            .map(|c| match &c.message.content {
                OpenAIContent::Text(t) => t.clone(),
                OpenAIContent::Multi(parts) => parts
                    .iter()
                    .filter_map(|p| match p {
                        crate::native_engine::openai_client::OpenAIContentPart::Text { text } => Some(text.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join(""),
            })
            .unwrap_or_default();
        Ok(text)
    }

    pub fn save_workplan(&mut self, plan: &WorkPlan) -> Result<()> {
        let file_path = self.plans_dir.join(format!("{}.json", plan.id));
        let content = serde_json::to_string_pretty(plan)?;
        let mut file = File::create(&file_path)?;
        file.write_all(content.as_bytes())?;
        self.plans.insert(plan.id.clone(), plan.clone());
        Ok(())
    }

    pub async fn get_workplan(&self, id: &str) -> Option<WorkPlan> {
        self.plans.get(id).cloned()
    }

    pub async fn list_workplans(&self) -> Vec<WorkPlan> {
        self.plans.values().cloned().collect()
    }

    pub fn delete_workplan(&mut self, id: &str) -> bool {
        if self.plans.remove(id).is_some() {
            let file_path = self.plans_dir.join(format!("{}.json", id));
            let _ = fs::remove_file(&file_path);
            true
        } else {
            false
        }
    }

    pub fn create_sprint(&self, name: &str, start_date: &str, end_date: &str, tasks: Vec<String>) -> Sprint {
        Sprint {
            id: format!("sprint-{}", uuid::Uuid::new_v4()),
            name: name.to_string(),
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
            tasks,
            completed_tasks: Vec::new(),
        }
    }
}
