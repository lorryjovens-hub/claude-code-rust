//! MCP Server - Server implementation

use std::sync::Arc;

use std::collections::HashMap;

use super::{McpMessage, McpConfig, McpServerInfo};
use super::tools::ToolRegistry;
use super::resources::ResourceManager;
use super::prompts::PromptManager;
use super::sampling::SamplingManager;

pub struct McpServer {
    name: String,
    config: McpConfig,
    tool_registry: Arc<ToolRegistry>,
    resource_manager: Arc<ResourceManager>,
    prompt_manager: Arc<PromptManager>,
    sampling_manager: Arc<SamplingManager>,

}

impl McpServer {
    pub fn new(name: &str, config: McpConfig) -> Self {
        Self {
            name: name.to_string(),
            config,
            tool_registry: Arc::new(ToolRegistry::new()),
            resource_manager: Arc::new(ResourceManager::new()),
            prompt_manager: Arc::new(PromptManager::new()),
            sampling_manager: Arc::new(SamplingManager::new()),
        }
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn config(&self) -> &McpConfig {
        &self.config
    }
    
    pub fn tool_registry(&self) -> Arc<ToolRegistry> {
        self.tool_registry.clone()
    }
    
    pub fn resource_manager(&self) -> Arc<ResourceManager> {
        self.resource_manager.clone()
    }
    
    pub fn prompt_manager(&self) -> Arc<PromptManager> {
        self.prompt_manager.clone()
    }
    
    pub fn sampling_manager(&self) -> Arc<SamplingManager> {
        self.sampling_manager.clone()
    }
    
    pub async fn handle_message(&self, message: McpMessage) -> McpMessage {
        let id = message.id.unwrap_or(0);
        
        match message.method.as_deref() {
            Some("initialize") => self.handle_initialize(id, message.params).await,
            Some("tools/list") => self.handle_tools_list(id).await,
            Some("tools/call") => self.handle_tools_call(id, message.params).await,
            Some("resources/list") => self.handle_resources_list(id).await,
            Some("resources/read") => self.handle_resources_read(id, message.params).await,
            Some("prompts/list") => self.handle_prompts_list(id).await,
            Some("prompts/get") => self.handle_prompts_get(id, message.params).await,
            Some("sampling/createMessage") => self.handle_sampling(id, message.params).await,
            Some("ping") => McpMessage::response(id, serde_json::json!({"status": "ok"})),
            _ => McpMessage::error_response(id, -32601, "Method not found"),
        }
    }
    
    async fn handle_initialize(&self, id: i64, _params: Option<serde_json::Value>) -> McpMessage {
        self.tool_registry.register_builtin_tools().await;
        self.prompt_manager.register_builtin_prompts().await;
        
        McpMessage::response(id, serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {},
                "prompts": {},
                "sampling": {}
            },
            "serverInfo": {
                "name": self.name,
                "version": "0.1.0"
            }
        }))
    }
    
    async fn handle_tools_list(&self, id: i64) -> McpMessage {
        let tools = self.tool_registry.list().await;
        McpMessage::response(id, serde_json::json!({
            "tools": tools
        }))
    }
    
    async fn handle_tools_call(&self, id: i64, params: Option<serde_json::Value>) -> McpMessage {
        if let Some(params) = params {
            if let Some(name) = params["name"].as_str() {
                let arguments = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));
                
                match self.tool_registry.execute(name, arguments).await {
                    Ok(result) => McpMessage::response(id, serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": serde_json::to_string(&result).unwrap_or_default()
                        }]
                    })),
                    Err(e) => McpMessage::error_response(id, -32000, &e.to_string()),
                }
            } else {
                McpMessage::error_response(id, -32602, "Missing tool name")
            }
        } else {
            McpMessage::error_response(id, -32602, "Missing parameters")
        }
    }
    
    async fn handle_resources_list(&self, id: i64) -> McpMessage {
        let resources = self.resource_manager.list().await;
        McpMessage::response(id, serde_json::json!({
            "resources": resources
        }))
    }
    
    async fn handle_resources_read(&self, id: i64, params: Option<serde_json::Value>) -> McpMessage {
        if let Some(params) = params {
            if let Some(uri) = params["uri"].as_str() {
                match self.resource_manager.read(uri).await {
                    Ok(content) => McpMessage::response(id, serde_json::json!({
                        "contents": [content]
                    })),
                    Err(e) => McpMessage::error_response(id, -32000, &e.to_string()),
                }
            } else {
                McpMessage::error_response(id, -32602, "Missing resource URI")
            }
        } else {
            McpMessage::error_response(id, -32602, "Missing parameters")
        }
    }
    
    async fn handle_prompts_list(&self, id: i64) -> McpMessage {
        let prompts = self.prompt_manager.list().await;
        McpMessage::response(id, serde_json::json!({
            "prompts": prompts
        }))
    }
    
    async fn handle_prompts_get(&self, id: i64, params: Option<serde_json::Value>) -> McpMessage {
        if let Some(params) = params {
            if let Some(name) = params["name"].as_str() {
                let args: HashMap<String, String> = params["arguments"]
                    .as_object()
                    .map(|obj| obj.iter()
                        .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                        .collect())
                    .unwrap_or_default();
                
                match self.prompt_manager.render(name, args).await {
                    Ok(rendered) => McpMessage::response(id, serde_json::json!({
                        "messages": [{
                            "role": "user",
                            "content": {
                                "type": "text",
                                "text": rendered
                            }
                        }]
                    })),
                    Err(e) => McpMessage::error_response(id, -32000, &e.to_string()),
                }
            } else {
                McpMessage::error_response(id, -32602, "Missing prompt name")
            }
        } else {
            McpMessage::error_response(id, -32602, "Missing parameters")
        }
    }
    
    async fn handle_sampling(&self, id: i64, _params: Option<serde_json::Value>) -> McpMessage {
        McpMessage::response(id, serde_json::json!({
            "status": "sampling_request_created",
            "message": "Sampling request pending approval"
        }))
    }
    
    pub async fn get_info(&self) -> McpServerInfo {
        McpServerInfo {
            name: self.name.clone(),
            status: self.config.status.clone(),
            tools_count: self.tool_registry.list().await.len(),
            resources_count: self.resource_manager.list().await.len(),
            prompts_count: self.prompt_manager.list().await.len(),
            started_at: None,
            last_error: None,
        }
    }
}
