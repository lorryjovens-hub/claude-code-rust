//! Agents Service - Built-in agent system
//!
//! Built-in agents for various tasks including:
//! - claudeCodeGuideAgent: Claude Code guidance
//! - exploreAgent: Codebase exploration
//! - generalPurposeAgent: General purpose tasks
//! - planAgent: Planning and task breakdown
//! - verificationAgent: Verification and testing

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentType {
    ClaudeCodeGuide,
    Explore,
    GeneralPurpose,
    Plan,
    Verification,
    Custom,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::ClaudeCodeGuide => write!(f, "claude-code-guide"),
            AgentType::Explore => write!(f, "explore"),
            AgentType::GeneralPurpose => write!(f, "general-purpose"),
            AgentType::Plan => write!(f, "plan"),
            AgentType::Verification => write!(f, "verification"),
            AgentType::Custom => write!(f, "custom"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub agent_type: AgentType,
    pub name: String,
    pub description: String,
    pub when_to_use: String,
    pub tools: Vec<String>,
    pub model: String,
    pub system_prompt: String,
    pub source: String,
    pub base_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: String,
    pub agent_type: AgentType,
    pub status: AgentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<AgentMessage>,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentStatus {
    Idle,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentStatusReport {
    pub available_agents: Vec<AgentDefinition>,
    pub active_sessions: usize,
    pub sessions: Vec<AgentSession>,
}

pub struct AgentsService {
    state: Arc<RwLock<AppState>>,
    agents: Arc<RwLock<HashMap<AgentType, AgentDefinition>>>,
    sessions: Arc<RwLock<HashMap<String, AgentSession>>>,
}

impl AgentsService {
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        let agents = Self::load_builtin_agents();
        Self {
            state,
            agents: Arc::new(RwLock::new(agents)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn load_builtin_agents() -> HashMap<AgentType, AgentDefinition> {
        let mut agents = HashMap::new();

        agents.insert(
            AgentType::ClaudeCodeGuide,
            AgentDefinition {
                agent_type: AgentType::ClaudeCodeGuide,
                name: "Claude Code Guide".to_string(),
                description: "Guides users through Claude Code features and best practices".to_string(),
                when_to_use: "When you need help understanding Claude Code features, commands, or workflows".to_string(),
                tools: vec!["file_read".to_string(), "search".to_string()],
                model: "sonnet".to_string(),
                system_prompt: r#"You are a Claude Code Guide agent. Your role is to help users understand and effectively use Claude Code.

Key responsibilities:
1. Explain Claude Code features and capabilities
2. Guide users through common workflows
3. Provide best practices and tips
4. Help troubleshoot issues

Be concise, helpful, and focus on practical guidance."#.to_string(),
                source: "built-in".to_string(),
                base_dir: "built-in".to_string(),
            },
        );

        agents.insert(
            AgentType::Explore,
            AgentDefinition {
                agent_type: AgentType::Explore,
                name: "Explore Agent".to_string(),
                description: "Explores and analyzes codebases to understand structure and patterns".to_string(),
                when_to_use: "When you need to understand a codebase, find specific code, or analyze project structure".to_string(),
                tools: vec!["file_read".to_string(), "search".to_string(), "list_files".to_string()],
                model: "sonnet".to_string(),
                system_prompt: r#"You are an Explore agent. Your role is to analyze and understand codebases.

Key responsibilities:
1. Explore project structure
2. Identify key files and patterns
3. Understand code organization
4. Find relevant code for tasks

Be thorough but efficient. Focus on providing useful insights about the codebase."#.to_string(),
                source: "built-in".to_string(),
                base_dir: "built-in".to_string(),
            },
        );

        agents.insert(
            AgentType::GeneralPurpose,
            AgentDefinition {
                agent_type: AgentType::GeneralPurpose,
                name: "General Purpose Agent".to_string(),
                description: "Handles general tasks and questions".to_string(),
                when_to_use: "For general tasks that don't fit other specialized agents".to_string(),
                tools: vec!["file_read".to_string(), "file_write".to_string(), "file_edit".to_string(), "search".to_string(), "execute_command".to_string()],
                model: "sonnet".to_string(),
                system_prompt: r#"You are a General Purpose agent. Your role is to handle a wide variety of tasks.

Key responsibilities:
1. Execute user requests efficiently
2. Use appropriate tools for tasks
3. Provide clear and helpful responses
4. Handle edge cases gracefully

Be flexible and adaptive to different types of requests."#.to_string(),
                source: "built-in".to_string(),
                base_dir: "built-in".to_string(),
            },
        );

        agents.insert(
            AgentType::Plan,
            AgentDefinition {
                agent_type: AgentType::Plan,
                name: "Plan Agent".to_string(),
                description: "Creates detailed plans and breaks down complex tasks".to_string(),
                when_to_use: "When you need to plan a complex task or break down work into steps".to_string(),
                tools: vec!["file_read".to_string(), "search".to_string()],
                model: "sonnet".to_string(),
                system_prompt: r#"You are a Plan agent. Your role is to create detailed plans for complex tasks.

Key responsibilities:
1. Analyze task requirements
2. Break down complex tasks into steps
3. Identify dependencies and risks
4. Create actionable plans

Be thorough and structured. Focus on creating clear, executable plans."#.to_string(),
                source: "built-in".to_string(),
                base_dir: "built-in".to_string(),
            },
        );

        agents.insert(
            AgentType::Verification,
            AgentDefinition {
                agent_type: AgentType::Verification,
                name: "Verification Agent".to_string(),
                description: "Verifies implementations and runs tests".to_string(),
                when_to_use: "When you need to verify code works correctly or run tests".to_string(),
                tools: vec!["file_read".to_string(), "execute_command".to_string(), "search".to_string()],
                model: "sonnet".to_string(),
                system_prompt: r#"You are a Verification agent. Your role is to verify implementations and ensure quality.

Key responsibilities:
1. Run tests and analyze results
2. Verify code correctness
3. Check for edge cases
4. Report issues clearly

Be thorough and systematic. Focus on finding and reporting issues."#.to_string(),
                source: "built-in".to_string(),
                base_dir: "built-in".to_string(),
            },
        );

        agents
    }

    pub async fn list_agents(&self) -> Vec<AgentDefinition> {
        let agents = self.agents.read().await;
        agents.values().cloned().collect()
    }

    pub async fn get_agent(&self, agent_type: &AgentType) -> Option<AgentDefinition> {
        let agents = self.agents.read().await;
        agents.get(agent_type).cloned()
    }

    pub async fn run_agent(&self, agent_type: &AgentType, prompt: &str) -> anyhow::Result<AgentSession> {
        let agents = self.agents.read().await;
        let agent = agents
            .get(agent_type)
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {:?}", agent_type))?;

        let session_id = uuid::Uuid::new_v4().to_string();
        let session = AgentSession {
            id: session_id.clone(),
            agent_type: agent_type.clone(),
            status: AgentStatus::Running,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            messages: vec![AgentMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
                timestamp: Utc::now(),
            }],
            result: None,
        };

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.clone(), session.clone());
        }

        println!("🤖 Running agent: {} ({})", agent.name, session_id);

        let result = self.execute_agent(&agent, prompt).await?;

        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.status = AgentStatus::Completed;
            session.result = Some(result.clone());
            session.updated_at = Utc::now();
            session.messages.push(AgentMessage {
                role: "assistant".to_string(),
                content: result,
                timestamp: Utc::now(),
            });
            
            return Ok(session.clone());
        }

        Err(anyhow::anyhow!("Session not found after execution"))
    }

    async fn execute_agent(&self, agent: &AgentDefinition, prompt: &str) -> anyhow::Result<String> {
        let state = self.state.read().await;
        let api_client = crate::api::ApiClient::new(state.settings.clone());

        let messages = vec![
            crate::api::ChatMessage {
                role: "system".to_string(),
                content: Some(agent.system_prompt.clone()),
                tool_calls: None,
                tool_call_id: None,
            },
            crate::api::ChatMessage {
                role: "user".to_string(),
                content: Some(prompt.to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        let response = api_client.chat(messages, None).await?;

        if let Some(choice) = response.choices.first() {
            return Ok(choice.message.content.clone().unwrap_or_default());
        }

        Ok(String::new())
    }

    pub async fn get_session(&self, session_id: &str) -> Option<AgentSession> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    pub async fn list_sessions(&self) -> Vec<AgentSession> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    pub async fn cancel_session(&self, session_id: &str) -> anyhow::Result<()> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = AgentStatus::Failed;
            session.updated_at = Utc::now();
            println!("🚫 Session cancelled: {}", session_id);
        }

        Ok(())
    }

    pub async fn get_status(&self) -> AgentStatusReport {
        let agents = self.agents.read().await;
        let sessions = self.sessions.read().await;
        
        let active_sessions = sessions
            .values()
            .filter(|s| s.status == AgentStatus::Running)
            .count();

        AgentStatusReport {
            available_agents: agents.values().cloned().collect(),
            active_sessions,
            sessions: sessions.values().cloned().collect(),
        }
    }

    pub async fn register_custom_agent(&self, definition: AgentDefinition) -> anyhow::Result<()> {
        let mut agents = self.agents.write().await;
        agents.insert(definition.agent_type.clone(), definition);
        println!("✅ Custom agent registered");
        Ok(())
    }

    pub async fn load_agents_from_dir(&self, dir: &PathBuf) -> anyhow::Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        let mut agents = self.agents.write().await;
        
        let entries = std::fs::read_dir(dir)?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(agent) = serde_json::from_str::<AgentDefinition>(&content) {
                        agents.insert(agent.agent_type.clone(), agent);
                    }
                }
            }
        }

        println!("📂 Loaded agents from: {:?}", dir);
        Ok(())
    }
}
