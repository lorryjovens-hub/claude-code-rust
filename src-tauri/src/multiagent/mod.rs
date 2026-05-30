use crate::native_engine::anthropic_client::{AnthropicClient, AnthropicContent, AnthropicMessage};
use crate::native_engine::openai_client::{OpenAIClient, OpenAIContent, OpenAIMessage};
use crate::native_engine::provider_manager::{ApiFormat, ResolvedProvider};
use crate::agent_bus::{AgentMessageBus, AgentMessageType, AgentMessage};
use anyhow::Result;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex, Semaphore};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentState {
    Idle,
    Planning,
    Executing,
    Synthesizing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    Planner,
    Researcher,
    Writer,
    Reviewer,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_id: String,
    pub agent_type: AgentType,
    pub model_id: Option<String>,
    pub system_prompt: Option<String>,
    pub max_tokens: Option<u32>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub task_id: String,
    pub agent_id: String,
    pub description: String,
    pub input: serde_json::Value,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub task_id: String,
    pub agent_id: String,
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchPlan {
    pub title: String,
    pub sub_questions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestratorEvent {
    PhaseChanged {
        phase: String,
        label: String,
    },
    AgentStarted {
        agent_id: String,
        task_description: String,
    },
    AgentProgress {
        agent_id: String,
        progress: f32,
        message: String,
    },
    AgentCompleted {
        agent_id: String,
        result: AgentResult,
    },
    AgentFailed {
        agent_id: String,
        error: String,
    },
    ResearchPlanGenerated {
        plan: ResearchPlan,
    },
    ResearchSource {
        agent_id: String,
        source: serde_json::Value,
    },
    ResearchFinding {
        agent_id: String,
        markdown: String,
    },
    ReportDelta {
        text: String,
    },
    FinalReport {
        markdown: String,
    },
    Completed {
        total_agents: usize,
        duration_ms: u64,
    },
}

#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    pub max_concurrent_agents: usize,
    pub max_sub_questions: usize,
    pub timeout_ms: u64,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_agents: 5,
            max_sub_questions: 5,
            timeout_ms: 300_000,
        }
    }
}

pub struct MultiAgentOrchestrator {
    config: OrchestratorConfig,
    agents: Arc<Mutex<HashMap<String, AgentConfig>>>,
    state: Arc<Mutex<AgentState>>,
    event_tx: broadcast::Sender<OrchestratorEvent>,
    http_client: reqwest::Client,
    message_bus: Arc<AgentMessageBus>,
}

impl MultiAgentOrchestrator {
    pub fn new(config: OrchestratorConfig) -> Self {
        let event_tx = broadcast::channel(100).0;
        let message_bus = Arc::new(AgentMessageBus::new());
        Self {
            config,
            agents: Arc::new(Mutex::new(HashMap::new())),
            state: Arc::new(Mutex::new(AgentState::Idle)),
            event_tx,
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .build()
                .expect("Failed to create HTTP client"),
            message_bus,
        }
    }

    pub async fn register_agent(&self, config: AgentConfig) {
        self.agents
            .lock()
            .await
            .insert(config.agent_id.clone(), config);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<OrchestratorEvent> {
        self.event_tx.subscribe()
    }

    pub async fn get_state(&self) -> AgentState {
        self.state.lock().await.clone()
    }

    fn emit_event(&self, event: OrchestratorEvent) {
        let _ = self.event_tx.send(event);
    }

    pub async fn execute_research(
        &self,
        query: String,
        provider: &ResolvedProvider,
    ) -> Result<AgentResult> {
        let start_time = std::time::Instant::now();
        self.set_state(AgentState::Planning).await;

        self.emit_event(OrchestratorEvent::PhaseChanged {
            phase: "planning".to_string(),
            label: "Creating research plan...".to_string(),
        });

        let plan = self.generate_research_plan(&query, provider).await?;
        self.emit_event(OrchestratorEvent::ResearchPlanGenerated { plan: plan.clone() });

        self.set_state(AgentState::Executing).await;
        self.emit_event(OrchestratorEvent::PhaseChanged {
            phase: "gathering".to_string(),
            label: "Gathering information...".to_string(),
        });

        let sub_results = self.execute_sub_researchers(&query, &plan.sub_questions, provider).await?;

        self.set_state(AgentState::Synthesizing).await;
        self.emit_event(OrchestratorEvent::PhaseChanged {
            phase: "writing".to_string(),
            label: "Writing final report...".to_string(),
        });

        let final_report = self.synthesize_report(&query, &sub_results, provider).await?;

        self.set_state(AgentState::Completed).await;
        self.emit_event(OrchestratorEvent::Completed {
            total_agents: plan.sub_questions.len() + 2,
            duration_ms: start_time.elapsed().as_millis() as u64,
        });

        self.message_bus.broadcast(
            "orchestrator".to_string(),
            AgentMessageType::StatusUpdate,
            serde_json::json!({
                "event": "research_completed",
                "title": plan.title,
                "duration_ms": start_time.elapsed().as_millis() as u64,
            }),
        ).await;

        Ok(AgentResult {
            task_id: "research_task".to_string(),
            agent_id: "orchestrator".to_string(),
            success: true,
            output: serde_json::json!({
                "title": plan.title,
                "report": final_report,
                "sub_questions": plan.sub_questions,
                "sub_results": sub_results
            }),
            error: None,
            duration_ms: start_time.elapsed().as_millis() as u64,
        })
    }

    async fn set_state(&self, new_state: AgentState) {
        let mut state = self.state.lock().await;
        *state = new_state;
    }

    async fn call_llm_non_streaming(
        &self,
        provider: &ResolvedProvider,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<String> {
        match provider.provider.api_format {
            ApiFormat::Anthropic => {
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
                        crate::native_engine::anthropic_client::ContentBlock::Text { text } => {
                            Some(text.clone())
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("");
                Ok(text)
            }
            ApiFormat::OpenAI => {
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
                                crate::native_engine::openai_client::OpenAIContentPart::Text {
                                    text,
                                } => Some(text.clone()),
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                            .join(""),
                    })
                    .unwrap_or_default();
                Ok(text)
            }
        }
    }

    async fn generate_research_plan(
        &self,
        query: &str,
        provider: &ResolvedProvider,
    ) -> Result<ResearchPlan> {
        let system_prompt = r#"You are a research planning assistant. Given a research query, generate a structured research plan.
Return ONLY valid JSON in this exact format, no other text:
{"title": "Research title", "sub_questions": ["question 1", "question 2", "question 3"]}

Guidelines:
- Title should be concise and descriptive
- Generate 3-5 specific sub-questions that cover different aspects of the topic
- Each sub-question should be answerable through research
- Sub-questions should be complementary, not overlapping"#;

        let user_message = format!("Generate a research plan for the following query:\n\n{}", query);

        match self.call_llm_non_streaming(provider, system_prompt, &user_message).await {
            Ok(response_text) => {
                let json_str = response_text
                    .trim()
                    .trim_start_matches("```json")
                    .trim_start_matches("```")
                    .trim_end_matches("```")
                    .trim();

                match serde_json::from_str::<ResearchPlan>(json_str) {
                    Ok(mut plan) => {
                        plan.sub_questions.truncate(self.config.max_sub_questions);
                        Ok(plan)
                    }
                    Err(e) => {
                        tracing::error!(module = "MultiAgent", "Failed to parse LLM plan response: {}, raw: {}", e, json_str);
                        Ok(fallback_plan(query))
                    }
                }
            }
            Err(e) => {
                tracing::error!(module = "MultiAgent", "LLM call for research plan failed: {}", e);
                Ok(fallback_plan(query))
            }
        }
    }

    async fn execute_sub_researchers(
        &self,
        main_query: &str,
        sub_questions: &[String],
        provider: &ResolvedProvider,
    ) -> Result<Vec<AgentResult>> {
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent_agents));
        let mut handles = Vec::new();

        for (idx, sub_question) in sub_questions.iter().enumerate() {
            let agent_id = format!("researcher_{}", idx);
            let sub_question = sub_question.clone();
            let main_query = main_query.to_string();
            let provider = provider.clone();
            let semaphore = semaphore.clone();
            let event_tx = self.event_tx.clone();
            let max_tokens = provider.model.max_tokens.unwrap_or(4096);
            let message_bus = self.message_bus.clone();

            self.emit_event(OrchestratorEvent::AgentStarted {
                agent_id: agent_id.clone(),
                task_description: sub_question.clone(),
            });

            let handle = tokio::spawn(async move {
                let _permit = match semaphore.acquire().await {
                    Ok(p) => p,
                    Err(_) => {
                        let _ = event_tx.send(OrchestratorEvent::AgentFailed {
                            agent_id: agent_id.clone(),
                            error: "Semaphore closed".to_string(),
                        });
                        return AgentResult {
                            task_id: format!("task_{}", idx),
                            agent_id: agent_id.clone(),
                            success: false,
                            output: serde_json::json!({ "error": "Semaphore closed" }),
                            error: Some("Semaphore closed".to_string()),
                            duration_ms: 0,
                        };
                    }
                };
                let start = std::time::Instant::now();

                let system_prompt = "You are a research assistant. Provide detailed, well-structured findings for the given research question.\nInclude relevant facts, analysis, and insights. Write in clear markdown format.";

                let user_message = format!(
                    "Research context: {}\n\nResearch question: {}\n\nProvide your detailed findings:",
                    main_query, sub_question
                );

                let findings_result: Result<String, String> = match provider.provider.api_format {
                    ApiFormat::Anthropic => {
                        let client = crate::native_engine::anthropic_client::AnthropicClient::new();
                        let messages = vec![crate::native_engine::anthropic_client::AnthropicMessage {
                            role: "user".to_string(),
                            content: crate::native_engine::anthropic_client::AnthropicContent::Text(user_message),
                        }];
                        match client.send_message(&provider, messages, Some(system_prompt), vec![], max_tokens).await {
                            Ok(response) => {
                                let text = response.content.iter().filter_map(|block| match block {
                                    crate::native_engine::anthropic_client::ContentBlock::Text { text } => Some(text.clone()),
                                    _ => None,
                                }).collect::<Vec<_>>().join("");
                                Ok(text)
                            }
                            Err(e) => Err(e.to_string()),
                        }
                    }
                    ApiFormat::OpenAI => {
                        let client = crate::native_engine::openai_client::OpenAIClient::new();
                        let messages = vec![crate::native_engine::openai_client::OpenAIMessage {
                            role: "user".to_string(),
                            content: crate::native_engine::openai_client::OpenAIContent::Text(user_message),
                            tool_calls: None,
                            tool_call_id: None,
                            reasoning_content: None,
                        }];
                        match client.send_message(&provider, messages, Some(system_prompt), vec![], max_tokens).await {
                            Ok(response) => {
                                let text = response.choices.first().map(|c| match &c.message.content {
                                    crate::native_engine::openai_client::OpenAIContent::Text(t) => t.clone(),
                                    crate::native_engine::openai_client::OpenAIContent::Multi(parts) => parts.iter().filter_map(|p| match p {
                                        crate::native_engine::openai_client::OpenAIContentPart::Text { text } => Some(text.clone()),
                                        _ => None,
                                    }).collect::<Vec<_>>().join(""),
                                }).unwrap_or_default();
                                Ok(text)
                            }
                            Err(e) => Err(e.to_string()),
                        }
                    }
                };

                let duration_ms = start.elapsed().as_millis() as u64;

                match findings_result {
                    Ok(findings) => {
                        let agent_result = AgentResult {
                            task_id: format!("task_{}", idx),
                            agent_id: agent_id.clone(),
                            success: true,
                            output: serde_json::json!({
                                "sub_question": sub_question,
                                "findings": findings,
                                "sources": []
                            }),
                            error: None,
                            duration_ms,
                        };

                        let _ = event_tx.send(OrchestratorEvent::AgentCompleted {
                            agent_id: agent_id.clone(),
                            result: agent_result.clone(),
                        });

                        message_bus.send(AgentMessage {
                            sender_id: agent_id.clone(),
                            recipient_id: "orchestrator".to_string(),
                            message_type: AgentMessageType::TaskResult,
                            payload: serde_json::json!({
                                "task_id": format!("task_{}", idx),
                                "status": "completed",
                                "duration_ms": duration_ms,
                            }),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis() as u64,
                            correlation_id: uuid::Uuid::new_v4().to_string(),
                        }).await;

                        agent_result
                    }
                    Err(e) => {
                        let agent_result = AgentResult {
                            task_id: format!("task_{}", idx),
                            agent_id: agent_id.clone(),
                            success: false,
                            output: serde_json::json!({
                                "sub_question": sub_question,
                                "findings": serde_json::Value::Null,
                                "error": e.clone()
                            }),
                            error: Some(e.clone()),
                            duration_ms,
                        };

                        let _ = event_tx.send(OrchestratorEvent::AgentFailed {
                            agent_id: agent_id.clone(),
                            error: e.clone(),
                        });

                        message_bus.send(AgentMessage {
                            sender_id: agent_id.clone(),
                            recipient_id: "orchestrator".to_string(),
                            message_type: AgentMessageType::Error,
                            payload: serde_json::json!({
                                "task_id": format!("task_{}", idx),
                                "error": e,
                            }),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis() as u64,
                            correlation_id: uuid::Uuid::new_v4().to_string(),
                        }).await;

                        agent_result
                    }
                }
            });

            handles.push(handle);
        }

        let results: Vec<AgentResult> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| match r {
                Ok(result) => result,
                Err(join_error) => AgentResult {
                    task_id: "unknown".to_string(),
                    agent_id: "unknown".to_string(),
                    success: false,
                    output: serde_json::json!({ "error": format!("Task failed: {:?}", join_error) }),
                    error: Some(format!("Task failed: {:?}", join_error)),
                    duration_ms: 0,
                },
            })
            .collect();

        Ok(results)
    }

    async fn synthesize_report(
        &self,
        main_query: &str,
        sub_results: &[AgentResult],
        provider: &ResolvedProvider,
    ) -> Result<String> {
        let mut findings_text = String::new();
        for result in sub_results {
            if result.success {
                let sub_q = result.output.get("sub_question").and_then(|v| v.as_str()).unwrap_or("");
                let findings = result.output.get("findings").and_then(|v| v.as_str()).unwrap_or("");
                findings_text.push_str(&format!("### Sub-question: {}\n\n{}\n\n---\n\n", sub_q, findings));
            }
        }

        let system_prompt = r#"You are a research report writer. Synthesize the provided research findings into a comprehensive, well-structured markdown report.
The report should include:
- A clear title
- An executive summary
- Detailed sections covering each research aspect
- A conclusion with key takeaways

Write in professional, clear markdown format."#;

        let user_message = format!(
            "Original research query: {}\n\nResearch findings:\n\n{}\n\nPlease write a comprehensive research report synthesizing these findings.",
            main_query, findings_text
        );

        match provider.provider.api_format {
            ApiFormat::Anthropic => {
                let client = AnthropicClient::new();
                let messages = vec![AnthropicMessage {
                    role: "user".to_string(),
                    content: AnthropicContent::Text(user_message),
                }];
                let max_tokens = provider.model.max_tokens.unwrap_or(8192);
                let mut stream = client
                    .send_message_stream(provider, messages, Some(system_prompt), vec![], max_tokens)
                    .await?;

                let mut report = String::new();
                let mut sse_buffer = String::new();

                while let Some(chunk_result) = stream.next().await {
                    let chunk = match chunk_result {
                        Ok(c) => c,
                        Err(e) => {
                            tracing::error!(module = "MultiAgent", "Stream error in synthesize: {}", e);
                            break;
                        }
                    };

                    sse_buffer.push_str(&chunk);
                    let consumed = crate::streaming::sse_parser::consume_sse_payloads(&sse_buffer);
                    sse_buffer = consumed.remainder;

                    for payload in &consumed.payloads {
                        if payload == "[DONE]" {
                            continue;
                        }
                        let parsed: serde_json::Value = match serde_json::from_str(payload) {
                            Ok(v) => v,
                            Err(_) => continue,
                        };
                        let event_type = parsed.get("type").and_then(|v| v.as_str()).unwrap_or("");
                        if event_type == "content_block_delta" {
                            if let Some(delta) = parsed.get("delta") {
                                if delta.get("type").and_then(|v| v.as_str()) == Some("text_delta") {
                                    if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                                        report.push_str(text);
                                        self.emit_event(OrchestratorEvent::ReportDelta {
                                            text: text.to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }

                self.emit_event(OrchestratorEvent::FinalReport {
                    markdown: report.clone(),
                });

                Ok(report)
            }
            ApiFormat::OpenAI => {
                let client = OpenAIClient::new();
                let messages = vec![OpenAIMessage {
                    role: "user".to_string(),
                    content: OpenAIContent::Text(user_message),
                    tool_calls: None,
                    tool_call_id: None,
                    reasoning_content: None,
                }];
                let max_tokens = provider.model.max_tokens.unwrap_or(8192);
                let mut stream = client
                    .send_message_stream(provider, messages, Some(system_prompt), vec![], max_tokens)
                    .await?;

                let mut report = String::new();
                let mut sse_buffer = String::new();

                while let Some(chunk_result) = stream.next().await {
                    let chunk = match chunk_result {
                        Ok(c) => c,
                        Err(e) => {
                            tracing::error!(module = "MultiAgent", "Stream error in synthesize: {}", e);
                            break;
                        }
                    };

                    sse_buffer.push_str(&chunk);
                    let consumed = crate::streaming::sse_parser::consume_sse_payloads(&sse_buffer);
                    sse_buffer = consumed.remainder;

                    for payload in &consumed.payloads {
                        if payload == "[DONE]" {
                            continue;
                        }
                        let parsed: serde_json::Value = match serde_json::from_str(payload) {
                            Ok(v) => v,
                            Err(_) => continue,
                        };
                        if let Some(choices) = parsed.get("choices") {
                            if let Some(choice) = choices.as_array().and_then(|a| a.first()) {
                                if let Some(delta) = choice.get("delta") {
                                    if let Some(content) = delta.get("content").and_then(|v| v.as_str()) {
                                        report.push_str(content);
                                        self.emit_event(OrchestratorEvent::ReportDelta {
                                            text: content.to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }

                self.emit_event(OrchestratorEvent::FinalReport {
                    markdown: report.clone(),
                });

                Ok(report)
            }
        }
    }
}

fn fallback_plan(query: &str) -> ResearchPlan {
    ResearchPlan {
        title: format!("Research: {}", &query[..usize::min(40, query.len())]),
        sub_questions: vec![
            format!("What is the current state of {}", query),
            format!("What are the key challenges in {}", query),
            format!("What are the future prospects for {}", query),
        ],
    }
}
