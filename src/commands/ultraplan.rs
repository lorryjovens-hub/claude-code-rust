//! Ultraplan 超级规划系统
//! 
//! 这个模块实现了多代理并行探索的超级规划功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

const ULTRAPLAN_TIMEOUT_MS: u64 = 30 * 60 * 1000;
pub const CCR_TERMS_URL: &str = "https://code.claude.com/docs/en/claude-code-on-the-web";

const DEFAULT_INSTRUCTIONS: &str = r#"
You are an expert planning assistant. Your task is to create a comprehensive, actionable plan.

## Instructions

1. Analyze the user's request thoroughly
2. Break down the task into clear, sequential steps
3. Identify all files that need to be created or modified
4. Consider edge cases and potential issues
5. Provide clear acceptance criteria

## Output Format

Structure your plan as follows:

### ## Goal
Brief description of what we're trying to achieve

### ## Steps
Numbered list of concrete steps to take

### ## Files
List of files to create or modify with brief descriptions

### ## Notes
Any additional context, considerations, or warnings

## Guidelines

- Be specific and actionable
- Include code snippets where helpful
- Consider dependencies between steps
- Identify potential risks or challenges
- Suggest testing approaches
"#;

/// 构建 Ultraplan 提示
pub fn build_ultraplan_prompt(blurb: &str, seed_plan: Option<&str>) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(seed) = seed_plan {
        parts.push("Here is a draft plan to refine:".to_string());
        parts.push(String::new());
        parts.push(seed.to_string());
        parts.push(String::new());
    }

    parts.push(DEFAULT_INSTRUCTIONS.to_string());

    if !blurb.is_empty() {
        parts.push(String::new());
        parts.push(blurb.to_string());
    }

    parts.join("\n")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UltraplanPhase {
    Launching,
    Running,
    NeedsInput,
    Approved,
    Failed,
}

impl std::fmt::Display for UltraplanPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UltraplanPhase::Launching => write!(f, "Launching"),
            UltraplanPhase::Running => write!(f, "Running"),
            UltraplanPhase::NeedsInput => write!(f, "Needs Input"),
            UltraplanPhase::Approved => write!(f, "Approved"),
            UltraplanPhase::Failed => write!(f, "Failed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UltraplanConfig {
    pub model: String,
    pub timeout_ms: u64,
    pub max_parallel_agents: usize,
}

impl Default for UltraplanConfig {
    fn default() -> Self {
        Self {
            model: "opus".to_string(),
            timeout_ms: ULTRAPLAN_TIMEOUT_MS,
            max_parallel_agents: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UltraplanSession {
    pub id: String,
    pub url: String,
    pub phase: UltraplanPhase,
    pub plan: Option<String>,
    pub created_at: i64,
    pub seed_plan: Option<String>,
    pub blurb: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UltraplanResult {
    pub session_id: String,
    pub plan: String,
    pub reject_count: u32,
    pub execution_target: ExecutionTarget,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionTarget {
    Local,
    Remote,
}

impl std::fmt::Display for ExecutionTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionTarget::Local => write!(f, "Local"),
            ExecutionTarget::Remote => write!(f, "Remote"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanEvaluation {
    pub completeness: f32,
    pub clarity: f32,
    pub feasibility: f32,
    pub coverage: f32,
    pub overall_score: f32,
}

pub struct UltraplanService {
    config: UltraplanConfig,
    sessions: Arc<RwLock<HashMap<String, UltraplanSession>>>,
    active_session: Arc<RwLock<Option<String>>>,
}

impl UltraplanService {
    pub fn new(config: Option<UltraplanConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            active_session: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn launch(
        &self,
        blurb: String,
        seed_plan: Option<String>,
    ) -> crate::error::Result<UltraplanSession> {
        let active = self.active_session.read().await;
        if active.is_some() {
            return Err(crate::error::ClaudeError::Other(
                "Ultraplan already active".to_string(),
            ));
        }

        let session_id = uuid::Uuid::new_v4().to_string();
        let url = format!("https://code.claude.com/session/{}", session_id);

        let session = UltraplanSession {
            id: session_id.clone(),
            url: url.clone(),
            phase: UltraplanPhase::Launching,
            plan: None,
            created_at: chrono::Utc::now().timestamp_millis(),
            seed_plan: seed_plan.clone(),
            blurb: blurb.clone(),
        };

        self.sessions.write().await.insert(session_id.clone(), session.clone());
        *self.active_session.write().await = Some(session_id.clone());

        tracing::info!("Ultraplan launched: {}", session_id);

        Ok(session)
    }

    pub async fn poll(&self, session_id: &str) -> crate::error::Result<UltraplanPhase> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| crate::error::ClaudeError::Other("Session not found".to_string()))?;

        Ok(session.phase.clone())
    }

    pub async fn update_phase(
        &self,
        session_id: &str,
        phase: UltraplanPhase,
    ) -> crate::error::Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.phase = phase;
        }
        Ok(())
    }

    pub async fn set_plan(&self, session_id: &str, plan: String) -> crate::error::Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.plan = Some(plan);
            session.phase = UltraplanPhase::Approved;
        }
        Ok(())
    }

    pub async fn stop(&self, session_id: &str) -> crate::error::Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.remove(session_id) {
            tracing::info!("Ultraplan stopped: {}", session.id);
        }

        let mut active = self.active_session.write().await;
        if active.as_ref() == Some(&session_id.to_string()) {
            *active = None;
        }

        Ok(())
    }

    pub async fn get_session(&self, session_id: &str) -> Option<UltraplanSession> {
        self.sessions.read().await.get(session_id).cloned()
    }

    pub async fn get_active_session(&self) -> Option<UltraplanSession> {
        let active_id = self.active_session.read().await.clone()?;
        self.get_session(&active_id).await
    }

    pub async fn evaluate_plan(&self, plan: &str) -> PlanEvaluation {
        let completeness = self.evaluate_completeness(plan);
        let clarity = self.evaluate_clarity(plan);
        let feasibility = self.evaluate_feasibility(plan);
        let coverage = self.evaluate_coverage(plan);

        let overall_score = (completeness + clarity + feasibility + coverage) / 4.0;

        PlanEvaluation {
            completeness,
            clarity,
            feasibility,
            coverage,
            overall_score,
        }
    }

    fn evaluate_completeness(&self, plan: &str) -> f32 {
        let sections = ["## Goal", "## Steps", "## Files", "## Notes"];
        let found = sections.iter().filter(|s| plan.contains(*s)).count();
        found as f32 / sections.len() as f32
    }

    fn evaluate_clarity(&self, plan: &str) -> f32 {
        let lines: Vec<&str> = plan.lines().collect();
        let code_blocks = lines.iter().filter(|l| l.starts_with("```")).count();
        let headers = lines.iter().filter(|l| l.starts_with("#")).count();

        if code_blocks > 0 && headers > 0 {
            0.9
        } else if headers > 0 {
            0.7
        } else {
            0.5
        }
    }

    fn evaluate_feasibility(&self, plan: &str) -> f32 {
        let risky_keywords = ["impossible", "unfeasible", "cannot", "won't work"];
        let has_risks = risky_keywords.iter().any(|k| plan.to_lowercase().contains(k));

        if has_risks {
            0.6
        } else {
            0.85
        }
    }

    fn evaluate_coverage(&self, plan: &str) -> f32 {
        let words = plan.split_whitespace().count();
        if words > 500 {
            0.9
        } else if words > 200 {
            0.7
        } else {
            0.5
        }
    }

    pub fn config(&self) -> &UltraplanConfig {
        &self.config
    }
}

impl std::fmt::Debug for UltraplanService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UltraplanService")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ultraplan_launch() {
        let service = UltraplanService::new(None);

        let session = service
            .launch("Test task".to_string(), None)
            .await
            .unwrap();

        assert_eq!(session.phase, UltraplanPhase::Launching);
        assert!(session.plan.is_none());
    }

    #[tokio::test]
    async fn test_ultraplan_phase_update() {
        let service = UltraplanService::new(None);

        let session = service
            .launch("Test task".to_string(), None)
            .await
            .unwrap();

        service
            .update_phase(&session.id, UltraplanPhase::Running)
            .await
            .unwrap();

        let updated = service.get_session(&session.id).await.unwrap();
        assert_eq!(updated.phase, UltraplanPhase::Running);
    }

    #[tokio::test]
    async fn test_ultraplan_stop() {
        let service = UltraplanService::new(None);

        let session = service
            .launch("Test task".to_string(), None)
            .await
            .unwrap();

        service.stop(&session.id).await.unwrap();

        assert!(service.get_session(&session.id).await.is_none());
    }

    #[test]
    fn test_build_ultraplan_prompt_with_seed() {
        let prompt = build_ultraplan_prompt("Test task", Some("Draft plan content"));

        assert!(prompt.contains("Test task"));
        assert!(prompt.contains("Draft plan content"));
        assert!(prompt.contains("draft plan to refine"));
    }

    #[test]
    fn test_build_ultraplan_prompt_without_seed() {
        let prompt = build_ultraplan_prompt("Test task", None);

        assert!(prompt.contains("Test task"));
        assert!(!prompt.contains("draft plan to refine"));
    }

    #[tokio::test]
    async fn test_plan_evaluation() {
        let service = UltraplanService::new(None);

        let plan = r#"
## Goal
Implement user authentication

## Steps
1. Create login form
2. Add password hashing
3. Implement session management

## Files
- src/auth.rs
- src/session.rs

## Notes
Use bcrypt for password hashing
"#;

        let evaluation = service.evaluate_plan(plan).await;

        assert!(evaluation.completeness > 0.7);
        assert!(evaluation.clarity > 0.6);
        assert!(evaluation.overall_score > 0.5);
    }
}
