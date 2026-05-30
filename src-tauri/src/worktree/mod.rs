use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub id: String,
    pub path: String,
    pub branch: String,
    pub agent_id: Option<String>,
    pub status: WorktreeStatus,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorktreeStatus {
    Active,
    Idle,
    Merging,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub worktree_id: String,
    pub task: String,
    pub status: AgentStatus,
    pub model: String,
    pub created_at: String,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    Starting,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorktreeRequest {
    pub branch_prefix: Option<String>,
    pub agent_name: Option<String>,
    pub task: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeWorktreeRequest {
    pub worktree_id: String,
    pub strategy: Option<String>,
}

pub struct WorktreeManager {
    repo_root: PathBuf,
    worktrees: Arc<Mutex<HashMap<String, WorktreeInfo>>>,
    agents: Arc<Mutex<HashMap<String, AgentInfo>>>,
}

impl WorktreeManager {
    pub fn new(repo_root: PathBuf) -> Self {
        Self {
            repo_root,
            worktrees: Arc::new(Mutex::new(HashMap::new())),
            agents: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_cwd(cwd: Option<String>) -> Self {
        let repo_root = cwd
            .map(|p| Path::new(&p).to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        Self {
            repo_root,
            worktrees: Arc::new(Mutex::new(HashMap::new())),
            agents: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn run_git(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.repo_root)
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("Git worktree error: {}", stderr))
        }
    }

    fn worktrees_dir(&self) -> PathBuf {
        self.repo_root.join(".git-worktrees")
    }

    pub async fn create_worktree(&self, req: CreateWorktreeRequest) -> Result<WorktreeInfo> {
        let id = Uuid::new_v4().to_string()[..8].to_string();
        let branch_prefix = req.branch_prefix.unwrap_or_else(|| "agent".to_string());
        let branch_name = format!("{}/{}", branch_prefix, id);

        self.run_git(&["branch", &branch_name])?;

        let worktrees_dir = self.worktrees_dir();
        std::fs::create_dir_all(&worktrees_dir)?;
        let worktree_path = worktrees_dir.join(&id);

        self.run_git(&[
            "worktree", "add",
            worktree_path.to_str().ok_or_else(|| anyhow!("Invalid path"))?,
            &branch_name,
        ])?;

        let mut info = WorktreeInfo {
            id: id.clone(),
            path: worktree_path.to_string_lossy().to_string(),
            branch: branch_name,
            agent_id: None,
            status: WorktreeStatus::Active,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        if let Some(agent_name) = req.agent_name {
            let agent_id = Uuid::new_v4().to_string()[..8].to_string();
            let agent = AgentInfo {
                id: agent_id.clone(),
                name: agent_name,
                worktree_id: id.clone(),
                task: req.task.unwrap_or_default(),
                status: AgentStatus::Starting,
                model: req.model.unwrap_or_else(|| "claude-sonnet-4-6".to_string()),
                created_at: chrono::Utc::now().to_rfc3339(),
                result: None,
            };
            info.agent_id = Some(agent.id.clone());
            self.agents.lock().await.insert(agent_id, agent);
        }

        self.worktrees.lock().await.insert(id.clone(), info.clone());
        Ok(info)
    }

    pub async fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        let worktrees = self.worktrees.lock().await;
        Ok(worktrees.values().cloned().collect())
    }

    pub async fn get_worktree(&self, id: &str) -> Option<WorktreeInfo> {
        let worktrees = self.worktrees.lock().await;
        worktrees.get(id).cloned()
    }

    pub async fn remove_worktree(&self, id: &str) -> Result<()> {
        let info = {
            let worktrees = self.worktrees.lock().await;
            worktrees.get(id).cloned().ok_or_else(|| anyhow!("Worktree not found"))?
        };

        self.run_git(&[
            "worktree", "remove",
            &info.path,
            "--force",
        ])?;

        self.run_git(&["branch", "-D", &info.branch])?;

        let mut worktrees = self.worktrees.lock().await;
        worktrees.remove(id);

        if let Some(agent_id) = &info.agent_id {
            self.agents.lock().await.remove(agent_id);
        }

        Ok(())
    }

    pub async fn merge_worktree(&self, req: MergeWorktreeRequest) -> Result<String> {
        let info = {
            let worktrees = self.worktrees.lock().await;
            worktrees.get(&req.worktree_id).cloned().ok_or_else(|| anyhow!("Worktree not found"))?
        };

        {
            let mut worktrees = self.worktrees.lock().await;
            if let Some(wt) = worktrees.get_mut(&req.worktree_id) {
                wt.status = WorktreeStatus::Merging;
            }
        }

        let strategy = req.strategy.as_deref().unwrap_or("ort");
        let output = self.run_git(&["merge", &info.branch, "--strategy", strategy])?;

        {
            let mut worktrees = self.worktrees.lock().await;
            if let Some(wt) = worktrees.get_mut(&req.worktree_id) {
                wt.status = WorktreeStatus::Idle;
            }
        }

        Ok(output)
    }

    pub async fn list_agents(&self) -> Vec<AgentInfo> {
        let agents = self.agents.lock().await;
        agents.values().cloned().collect()
    }

    pub async fn get_agent(&self, id: &str) -> Option<AgentInfo> {
        let agents = self.agents.lock().await;
        agents.get(id).cloned()
    }

    pub async fn update_agent_status(&self, id: &str, status: AgentStatus) -> Result<()> {
        let mut agents = self.agents.lock().await;
        if let Some(agent) = agents.get_mut(id) {
            agent.status = status;
            Ok(())
        } else {
            Err(anyhow!("Agent not found"))
        }
    }

    pub async fn set_agent_result(&self, id: &str, result: String) -> Result<()> {
        let mut agents = self.agents.lock().await;
        if let Some(agent) = agents.get_mut(id) {
            agent.result = Some(result);
            agent.status = AgentStatus::Completed;
            Ok(())
        } else {
            Err(anyhow!("Agent not found"))
        }
    }

    pub async fn cancel_agent(&self, id: &str) -> Result<()> {
        let mut agents = self.agents.lock().await;
        if let Some(agent) = agents.get_mut(id) {
            agent.status = AgentStatus::Cancelled;
            Ok(())
        } else {
            Err(anyhow!("Agent not found"))
        }
    }

    pub async fn sync_from_git(&self) -> Result<Vec<WorktreeInfo>> {
        let output = self.run_git(&["worktree", "list", "--porcelain"])?;

        let mut existing = self.worktrees.lock().await;
        let mut parsed: Vec<WorktreeInfo> = Vec::new();
        let mut current_path = String::new();

        for line in output.lines() {
            if line.starts_with("worktree ") {
                current_path = line[9..].to_string();
            } else if line.starts_with("branch ") {
                let current_branch = line[7..].to_string();
                if current_path.contains(".git-worktrees") {
                    let path = Path::new(&current_path);
                    let id = path.file_name()
                        .map(|f| f.to_string_lossy().to_string())
                        .unwrap_or_default();

                    if let Some(existing_wt) = existing.get(&id) {
                        parsed.push(existing_wt.clone());
                    } else {
                        let info = WorktreeInfo {
                            id: id.clone(),
                            path: current_path.clone(),
                            branch: current_branch.clone(),
                            agent_id: None,
                            status: WorktreeStatus::Active,
                            created_at: chrono::Utc::now().to_rfc3339(),
                        };
                        parsed.push(info.clone());
                    }
                }
                current_path = String::new();
            }
        }

        for wt in &parsed {
            if !existing.contains_key(&wt.id) {
                existing.insert(wt.id.clone(), wt.clone());
            }
        }

        Ok(parsed)
    }
}
