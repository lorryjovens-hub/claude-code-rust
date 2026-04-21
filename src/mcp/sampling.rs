//! MCP Sampling - LLM sampling support

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingRequest {
    pub messages: Vec<SamplingMessage>,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
    pub system_prompt: Option<String>,
}

impl SamplingRequest {
    pub fn new(messages: Vec<SamplingMessage>) -> Self {
        Self {
            messages,
            model: None,
            max_tokens: None,
            temperature: None,
            stop_sequences: None,
            system_prompt: None,
        }
    }
    
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = Some(model.to_string());
        self
    }
    
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
    
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
    
    pub fn with_system_prompt(mut self, prompt: &str) -> Self {
        self.system_prompt = Some(prompt.to_string());
        self
    }
    
    pub fn add_message(mut self, role: &str, content: &str) -> Self {
        self.messages.push(SamplingMessage {
            role: role.to_string(),
            content: SamplingContent {
                content_type: "text".to_string(),
                text: content.to_string(),
            },
        });
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingMessage {
    pub role: String,
    pub content: SamplingContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingResponse {
    pub model: String,
    pub content: SamplingContent,
    pub stop_reason: Option<String>,
    pub usage: Option<SamplingUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

pub struct SamplingManager {
    pending_requests: Arc<RwLock<Vec<PendingRequest>>>,
}

struct PendingRequest {
    id: String,
    request: SamplingRequest,
    response: Option<SamplingResponse>,
}

impl SamplingManager {
    pub fn new() -> Self {
        Self {
            pending_requests: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub async fn create_request(&self, request: SamplingRequest) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let mut pending = self.pending_requests.write().await;
        pending.push(PendingRequest {
            id: id.clone(),
            request,
            response: None,
        });
        id
    }
    
    pub async fn get_request(&self, id: &str) -> Option<SamplingRequest> {
        let pending = self.pending_requests.read().await;
        pending.iter().find(|r| r.id == id).map(|r| r.request.clone())
    }
    
    pub async fn submit_response(&self, id: &str, response: SamplingResponse) -> anyhow::Result<()> {
        let mut pending = self.pending_requests.write().await;
        if let Some(req) = pending.iter_mut().find(|r| r.id == id) {
            req.response = Some(response);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Request not found: {}", id))
        }
    }
    
    pub async fn get_response(&self, id: &str) -> Option<SamplingResponse> {
        let pending = self.pending_requests.read().await;
        pending.iter().find(|r| r.id == id).and_then(|r| r.response.clone())
    }
    
    pub async fn list_pending(&self) -> Vec<(String, SamplingRequest)> {
        let pending = self.pending_requests.read().await;
        pending.iter()
            .filter(|r| r.response.is_none())
            .map(|r| (r.id.clone(), r.request.clone()))
            .collect()
    }
    
    pub async fn clear_completed(&self) {
        let mut pending = self.pending_requests.write().await;
        pending.retain(|r| r.response.is_none());
    }
    
    pub async fn execute_with_api(&self, request: SamplingRequest, api_client: &crate::api::ApiClient) -> anyhow::Result<SamplingResponse> {
        let messages: Vec<crate::api::ChatMessage> = request.messages.iter()
            .map(|m| crate::api::ChatMessage {
                role: m.role.clone(),
                content: Some(m.content.text.clone()),
                tool_calls: None,
                tool_call_id: None,
            })
            .collect();

        let response = api_client.chat(messages, None).await?;

        if let Some(choice) = response.choices.first() {
            let usage = response.usage.as_ref();
            Ok(SamplingResponse {
                model: response.model.clone(),
                content: SamplingContent {
                    content_type: "text".to_string(),
                    text: choice.message.content.clone().unwrap_or_default(),
                },
                stop_reason: Some(choice.finish_reason.clone().unwrap_or_default()),
                usage: Some(SamplingUsage {
                    input_tokens: usage.map(|u| u.prompt_tokens as u32).unwrap_or(0),
                    output_tokens: usage.map(|u| u.completion_tokens as u32).unwrap_or(0),
                }),
            })
        } else {
            Err(anyhow::anyhow!("No response from API"))
        }
    }
}

impl Default for SamplingManager {
    fn default() -> Self {
        Self::new()
    }
}
