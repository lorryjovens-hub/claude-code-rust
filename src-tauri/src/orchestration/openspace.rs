use crate::native_engine::anthropic_client::{AnthropicClient, AnthropicContent, AnthropicMessage};
use crate::native_engine::openai_client::{OpenAIClient, OpenAIContent, OpenAIMessage};
use crate::native_engine::provider_manager::{ApiFormat, ResolvedProvider};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub priority: Priority,
    pub acceptance_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementsAnalysis {
    pub goal: String,
    pub requirements: Vec<Requirement>,
    pub user_stories: Vec<UserStory>,
    pub success_metrics: Vec<SuccessMetric>,
    pub risks: Vec<Risk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStory {
    pub id: String,
    pub role: String,
    pub goal: String,
    pub benefit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessMetric {
    pub id: String,
    pub description: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub id: String,
    pub description: String,
    pub impact: String,
    pub mitigation: String,
}

#[derive(Clone)]
pub struct OpenSpaceManager {
    requirements_store: HashMap<String, RequirementsAnalysis>,
}

impl OpenSpaceManager {
    pub fn new(_data_dir: &Path) -> Self {
        Self {
            requirements_store: HashMap::new(),
        }
    }

    pub async fn analyze_requirements(
        &self,
        goal: &str,
        provider: &ResolvedProvider,
    ) -> Result<RequirementsAnalysis> {
        let system_prompt = r#"You are a senior Product Manager AI agent specialized in requirements analysis.

Your task is to analyze the user's goal and generate a comprehensive requirements document.

Output ONLY valid JSON in this exact format:
{
  "requirements": [
    {
      "id": "REQ-001",
      "title": "Requirement title",
      "description": "Detailed description",
      "category": "functional|non-functional|technical|business",
      "priority": "critical|high|medium|low",
      "acceptance_criteria": ["criterion 1", "criterion 2"]
    }
  ],
  "user_stories": [
    {
      "id": "US-001",
      "role": "User role",
      "goal": "What the user wants to achieve",
      "benefit": "How this benefits the user"
    }
  ],
  "success_metrics": [
    {
      "id": "SM-001",
      "description": "Metric description",
      "target": "Target value"
    }
  ],
  "risks": [
    {
      "id": "RISK-001",
      "description": "Risk description",
      "impact": "High/Medium/Low",
      "mitigation": "Mitigation strategy"
    }
  ]
}

Guidelines:
- Generate 5-10 well-structured requirements
- Include both functional and non-functional requirements
- Create 3-5 user stories from different user perspectives
- Define measurable success metrics
- Identify potential risks and mitigation strategies
- Prioritize requirements appropriately"#;

        let user_message = format!(
            "Analyze the following goal and generate comprehensive requirements:\n\nGoal: {}\n\nProvide your analysis in JSON format as specified.",
            goal
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
                let requirements: Vec<Requirement> = serde_json::from_value(
                    parsed.get("requirements").cloned().unwrap_or_default()
                ).unwrap_or_default();
                
                let user_stories: Vec<UserStory> = serde_json::from_value(
                    parsed.get("user_stories").cloned().unwrap_or_default()
                ).unwrap_or_default();
                
                let success_metrics: Vec<SuccessMetric> = serde_json::from_value(
                    parsed.get("success_metrics").cloned().unwrap_or_default()
                ).unwrap_or_default();
                
                let risks: Vec<Risk> = serde_json::from_value(
                    parsed.get("risks").cloned().unwrap_or_default()
                ).unwrap_or_default();

                Ok(RequirementsAnalysis {
                    goal: goal.to_string(),
                    requirements,
                    user_stories,
                    success_metrics,
                    risks,
                })
            }
            Err(e) => {
                tracing::warn!(module = "OpenSpace", "Failed to parse requirements: {}", e);
                Ok(Self::generate_fallback_requirements(goal))
            }
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

    fn generate_fallback_requirements(goal: &str) -> RequirementsAnalysis {
        RequirementsAnalysis {
            goal: goal.to_string(),
            requirements: vec![
                Requirement {
                    id: "REQ-001".to_string(),
                    title: "Core Functionality".to_string(),
                    description: "Implement the main functionality as described in the goal".to_string(),
                    category: "functional".to_string(),
                    priority: Priority::Critical,
                    acceptance_criteria: vec!["Functionality works as expected".to_string()],
                },
                Requirement {
                    id: "REQ-002".to_string(),
                    title: "User Interface".to_string(),
                    description: "Create user-friendly interface".to_string(),
                    category: "non-functional".to_string(),
                    priority: Priority::High,
                    acceptance_criteria: vec!["UI is responsive".to_string()],
                },
            ],
            user_stories: vec![
                UserStory {
                    id: "US-001".to_string(),
                    role: "End User".to_string(),
                    goal: "To use the system to achieve my goal".to_string(),
                    benefit: "Get things done efficiently".to_string(),
                },
            ],
            success_metrics: vec![
                SuccessMetric {
                    id: "SM-001".to_string(),
                    description: "Task completion rate".to_string(),
                    target: "95%".to_string(),
                },
            ],
            risks: vec![
                Risk {
                    id: "RISK-001".to_string(),
                    description: "Technical complexity".to_string(),
                    impact: "High".to_string(),
                    mitigation: "Break down into smaller tasks".to_string(),
                },
            ],
        }
    }

    pub fn save_analysis(&mut self, analysis: RequirementsAnalysis) {
        let key = format!("{}", analysis.goal);
        self.requirements_store.insert(key, analysis);
    }

    pub fn get_analysis(&self, goal: &str) -> Option<&RequirementsAnalysis> {
        self.requirements_store.get(goal)
    }
}
