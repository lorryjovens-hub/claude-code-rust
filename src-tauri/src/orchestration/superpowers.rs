use crate::native_engine::anthropic_client::{AnthropicClient, AnthropicContent, AnthropicMessage};
use crate::native_engine::openai_client::{OpenAIClient, OpenAIContent, OpenAIMessage};
use crate::native_engine::provider_manager::{ApiFormat, ResolvedProvider};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::TaskDefinition;
use super::WorkPlan;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub message: String,
    pub issues: Vec<ValidationIssue>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub id: String,
    pub severity: Severity,
    pub category: String,
    pub message: String,
    pub task_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Critical,
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineeringRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: Severity,
    pub applies_to: Vec<String>,
    pub check_fn: String,
}

#[derive(Clone)]
pub struct SuperpowersEngine {
    rules: Vec<EngineeringRule>,
}

impl SuperpowersEngine {
    pub fn new() -> Self {
        Self {
            rules: Self::load_default_rules(),
        }
    }

    fn load_default_rules() -> Vec<EngineeringRule> {
        vec![
            EngineeringRule {
                id: "RULE-001".to_string(),
                name: "Task Dependencies Valid".to_string(),
                description: "All task dependencies must reference existing tasks".to_string(),
                severity: Severity::Critical,
                applies_to: vec!["task".to_string()],
                check_fn: "check_dependencies".to_string(),
            },
            EngineeringRule {
                id: "RULE-002".to_string(),
                name: "No Circular Dependencies".to_string(),
                description: "Tasks must not have circular dependencies".to_string(),
                severity: Severity::Critical,
                applies_to: vec!["workflow".to_string()],
                check_fn: "check_circular_dependencies".to_string(),
            },
            EngineeringRule {
                id: "RULE-003".to_string(),
                name: "Task Description Complete".to_string(),
                description: "Each task must have a meaningful description".to_string(),
                severity: Severity::Warning,
                applies_to: vec!["task".to_string()],
                check_fn: "check_description_length".to_string(),
            },
            EngineeringRule {
                id: "RULE-004".to_string(),
                name: "Agent Role Assigned".to_string(),
                description: "Each task must have an assigned agent role".to_string(),
                severity: Severity::Error,
                applies_to: vec!["task".to_string()],
                check_fn: "check_agent_role".to_string(),
            },
            EngineeringRule {
                id: "RULE-005".to_string(),
                name: "Plan Has Clear Goal".to_string(),
                description: "Work plan must have a clear objective".to_string(),
                severity: Severity::Warning,
                applies_to: vec!["workflow".to_string()],
                check_fn: "check_plan_objective".to_string(),
            },
            EngineeringRule {
                id: "RULE-006".to_string(),
                name: "Task Count Reasonable".to_string(),
                description: "Number of tasks should be between 3-15 for optimal execution".to_string(),
                severity: Severity::Info,
                applies_to: vec!["workflow".to_string()],
                check_fn: "check_task_count".to_string(),
            },
        ]
    }

    pub async fn validate_plan(
        &self,
        plan: &WorkPlan,
        provider: &ResolvedProvider,
    ) -> Result<ValidationResult> {
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        for rule in &self.rules {
            let rule_issues = self.apply_rule(rule, plan);
            issues.extend(rule_issues);
        }

        if !issues.is_empty() {
            let suggestions_result = self.generate_suggestions(plan, &issues, provider).await?;
            suggestions.extend(suggestions_result);
        }

        let has_critical = issues.iter().any(|i| i.severity == Severity::Critical);
        let has_error = issues.iter().any(|i| i.severity == Severity::Error);

        Ok(ValidationResult {
            valid: !has_critical && !has_error,
            message: if has_critical {
                "Plan has critical issues that must be fixed".to_string()
            } else if has_error {
                "Plan has errors that need attention".to_string()
            } else if issues.is_empty() {
                "Plan validation passed".to_string()
            } else {
                "Plan has warnings but can proceed".to_string()
            },
            issues,
            suggestions,
        })
    }

    fn apply_rule(&self, rule: &EngineeringRule, plan: &WorkPlan) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        match rule.check_fn.as_str() {
            "check_dependencies" => {
                let task_ids: Vec<String> = plan.tasks.iter().map(|t| t.id.clone()).collect();
                for task in &plan.tasks {
                    for dep in &task.dependencies {
                        if !task_ids.contains(dep) {
                            issues.push(ValidationIssue {
                                id: format!("{}-{}", rule.id, task.id),
                                severity: rule.severity.clone(),
                                category: "dependency".to_string(),
                                message: format!("Task {} has invalid dependency {}", task.id, dep),
                                task_id: Some(task.id.clone()),
                            });
                        }
                    }
                }
            }
            "check_circular_dependencies" => {
                if self.has_circular_dependency(plan) {
                    issues.push(ValidationIssue {
                        id: format!("{}-circular", rule.id),
                        severity: rule.severity.clone(),
                        category: "dependency".to_string(),
                        message: "Circular dependency detected in workflow".to_string(),
                        task_id: None,
                    });
                }
            }
            "check_description_length" => {
                for task in &plan.tasks {
                    if task.description.len() < 10 {
                        issues.push(ValidationIssue {
                            id: format!("{}-{}", rule.id, task.id),
                            severity: rule.severity.clone(),
                            category: "quality".to_string(),
                            message: format!("Task {} has insufficient description", task.id),
                            task_id: Some(task.id.clone()),
                        });
                    }
                }
            }
            "check_agent_role" => {
                for task in &plan.tasks {
                    if matches!(&task.agent_role, super::AgentRole::Custom(s) if s.is_empty()) {
                        issues.push(ValidationIssue {
                            id: format!("{}-{}", rule.id, task.id),
                            severity: rule.severity.clone(),
                            category: "configuration".to_string(),
                            message: format!("Task {} has no agent role assigned", task.id),
                            task_id: Some(task.id.clone()),
                        });
                    }
                }
            }
            "check_plan_objective" => {
                if plan.objective.len() < 10 {
                    issues.push(ValidationIssue {
                        id: format!("{}-objective", rule.id),
                        severity: rule.severity.clone(),
                        category: "quality".to_string(),
                        message: "Work plan objective is too brief".to_string(),
                        task_id: None,
                    });
                }
            }
            "check_task_count" => {
                if plan.tasks.len() < 3 {
                    issues.push(ValidationIssue {
                        id: format!("{}-count-low", rule.id),
                        severity: rule.severity.clone(),
                        category: "optimization".to_string(),
                        message: format!("Only {} tasks in plan - consider adding more detail", plan.tasks.len()),
                        task_id: None,
                    });
                } else if plan.tasks.len() > 15 {
                    issues.push(ValidationIssue {
                        id: format!("{}-count-high", rule.id),
                        severity: rule.severity.clone(),
                        category: "optimization".to_string(),
                        message: format!("{} tasks in plan - consider splitting into smaller workflows", plan.tasks.len()),
                        task_id: None,
                    });
                }
            }
            _ => {}
        }

        issues
    }

    fn has_circular_dependency(&self, plan: &WorkPlan) -> bool {
        let mut visited = HashMap::new();
        
        for task in &plan.tasks {
            if self.detect_cycle(task.id.clone(), &plan.tasks, &mut visited) {
                return true;
            }
        }
        
        false
    }

    fn detect_cycle(
        &self,
        task_id: String,
        tasks: &[TaskDefinition],
        visited: &mut HashMap<String, bool>,
    ) -> bool {
        if visited.get(&task_id) == Some(&true) {
            return true;
        }
        
        if visited.contains_key(&task_id) {
            return false;
        }
        
        visited.insert(task_id.clone(), true);
        
        if let Some(task) = tasks.iter().find(|t| t.id == task_id) {
            for dep in &task.dependencies {
                if self.detect_cycle(dep.clone(), tasks, visited) {
                    return true;
                }
            }
        }
        
        visited.insert(task_id, false);
        false
    }

    async fn generate_suggestions(
        &self,
        plan: &WorkPlan,
        issues: &[ValidationIssue],
        provider: &ResolvedProvider,
    ) -> Result<Vec<String>> {
        if issues.is_empty() {
            return Ok(Vec::new());
        }

        let system_prompt = r#"You are a senior engineering advisor AI agent.

Given a work plan and validation issues, provide actionable suggestions to fix the issues.

Output ONLY a JSON array of strings with your suggestions.

Example output:
["Fix task dependency X", "Add description to task Y"]

Focus on practical, actionable advice."#;

        let issues_json = serde_json::to_string(issues)?;
        let user_message = format!(
            "Work plan objective: {}\n\nValidation issues:\n{}\n\nProvide actionable suggestions to fix these issues.",
            plan.objective, issues_json
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

        match serde_json::from_str::<Vec<String>>(json_str) {
            Ok(suggestions) => Ok(suggestions),
            Err(_) => Ok(vec![
                "Review the validation issues and make necessary corrections".to_string(),
                "Ensure all dependencies reference valid tasks".to_string(),
                "Add detailed descriptions to all tasks".to_string(),
            ]),
        }
    }

    async fn call_anthropic(&self, provider: &ResolvedProvider, system_prompt: &str, user_message: &str) -> Result<String> {
        let client = AnthropicClient::new();
        let messages = vec![AnthropicMessage {
            role: "user".to_string(),
            content: AnthropicContent::Text(user_message.to_string()),
        }];
        let response = client
            .send_message(provider, messages, Some(system_prompt), vec![], 4096)
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
            .send_message(provider, messages, Some(system_prompt), vec![], 4096)
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

    pub fn get_rules(&self) -> &[EngineeringRule] {
        &self.rules
    }
}
