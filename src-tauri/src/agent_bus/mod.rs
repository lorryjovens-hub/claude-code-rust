pub mod workspace;
pub mod health;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub sender_id: String,
    pub recipient_id: String,
    pub message_type: AgentMessageType,
    pub payload: serde_json::Value,
    pub timestamp: u64,
    pub correlation_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentMessageType {
    TaskAssignment,
    TaskResult,
    DataRequest,
    DataResponse,
    StatusUpdate,
    Error,
    Heartbeat,
}

pub struct AgentMessageBus {
    tx: broadcast::Sender<AgentMessage>,
    subscribers: Arc<RwLock<HashMap<String, broadcast::Sender<AgentMessage>>>>,
}

impl AgentMessageBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self {
            tx,
            subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_agent(&self, agent_id: String) -> broadcast::Receiver<AgentMessage> {
        let mut subs = self.subscribers.write().await;
        let (agent_tx, agent_rx) = broadcast::channel(100);
        subs.insert(agent_id, agent_tx);
        agent_rx
    }

    pub async fn send(&self, message: AgentMessage) {
        let _ = self.tx.send(message.clone());
        let subs = self.subscribers.read().await;
        if let Some(agent_tx) = subs.get(&message.recipient_id) {
            let _ = agent_tx.send(message);
        }
    }

    pub async fn broadcast(&self, sender_id: String, message_type: AgentMessageType, payload: serde_json::Value) {
        let message = AgentMessage {
            sender_id,
            recipient_id: "all".to_string(),
            message_type,
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            correlation_id: uuid::Uuid::new_v4().to_string(),
        };
        let _ = self.tx.send(message);
    }

    pub fn subscribe_global(&self) -> broadcast::Receiver<AgentMessage> {
        self.tx.subscribe()
    }
}
