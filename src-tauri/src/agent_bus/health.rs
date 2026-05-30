use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq)]
pub enum AgentHealth {
    Healthy,
    Unhealthy(String),
    Unknown,
}

pub struct AgentHealthMonitor {
    agent_states: Arc<RwLock<HashMap<String, AgentHealth>>>,
    agent_heartbeats: Arc<RwLock<HashMap<String, Instant>>>,
    timeout: Duration,
    max_retries: u32,
    retry_counts: Arc<RwLock<HashMap<String, u32>>>,
}

impl AgentHealthMonitor {
    pub fn new(timeout_secs: u64) -> Self {
        Self {
            agent_states: Arc::new(RwLock::new(HashMap::new())),
            agent_heartbeats: Arc::new(RwLock::new(HashMap::new())),
            timeout: Duration::from_secs(timeout_secs),
            max_retries: 3,
            retry_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_agent(&self, agent_id: String) {
        let mut states = self.agent_states.write().await;
        states.insert(agent_id.clone(), AgentHealth::Healthy);
        let mut heartbeats = self.agent_heartbeats.write().await;
        heartbeats.insert(agent_id, Instant::now());
    }

    pub async fn record_heartbeat(&self, agent_id: &str) {
        let mut heartbeats = self.agent_heartbeats.write().await;
        heartbeats.insert(agent_id.to_string(), Instant::now());
        let mut states = self.agent_states.write().await;
        states.insert(agent_id.to_string(), AgentHealth::Healthy);
    }

    pub async fn check_health(&self) -> HashMap<String, AgentHealth> {
        let mut states = self.agent_states.write().await;
        let heartbeats = self.agent_heartbeats.read().await;
        let now = Instant::now();

        for (agent_id, last_heartbeat) in heartbeats.iter() {
            if now.duration_since(*last_heartbeat) > self.timeout {
                states.insert(agent_id.clone(), AgentHealth::Unhealthy("Heartbeat timeout".to_string()));
            }
        }

        states.clone()
    }

    pub async fn mark_failed(&self, agent_id: &str) {
        let mut states = self.agent_states.write().await;
        states.insert(agent_id.to_string(), AgentHealth::Unhealthy("Task failed".to_string()));
    }

    pub async fn should_retry(&self, agent_id: &str) -> bool {
        let mut retries = self.retry_counts.write().await;
        let count = retries.entry(agent_id.to_string()).or_insert(0);
        if *count < self.max_retries {
            *count += 1;
            true
        } else {
            false
        }
    }

    pub async fn reset_retry_count(&self, agent_id: &str) {
        let mut retries = self.retry_counts.write().await;
        retries.insert(agent_id.to_string(), 0);
    }
}
