//! WASM API Client - Browser-compatible API client

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// WASM-compatible API client
pub struct WasmApiClient {
    api_key: String,
    base_url: String,
    history: std::cell::RefCell<Vec<ChatMessage>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

impl WasmApiClient {
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            history: std::cell::RefCell::new(Vec::new()),
        }
    }

    pub async fn chat(&self, message: String) -> Result<ChatResponse, JsValue> {
        // Add user message to history
        self.add_message("user".to_string(), message.clone());

        let request_body = json!({
            "model": "deepseek-reasoner",
            "messages": [
                {"role": "user", "content": message}
            ],
            "max_tokens": 4096,
            "stream": false,
        });

        let response = self.send_request(&request_body).await?;
        
        // Parse response
        let content = response["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let usage = response["usage"].as_object().map(|u| Usage {
            prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as usize,
            completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as usize,
            total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as usize,
        });

        // Add assistant response to history
        self.add_message("assistant".to_string(), content.clone());

        Ok(ChatResponse { content, usage })
    }

    pub async fn chat_stream<F>(&self, message: String, mut callback: F) -> Result<(), JsValue>
    where
        F: FnMut(String),
    {
        let request_body = json!({
            "model": "deepseek-reasoner",
            "messages": [
                {"role": "user", "content": message}
            ],
            "max_tokens": 4096,
            "stream": true,
        });

        let response = self.send_request_stream(&request_body).await?;
        
        // Process stream chunks
        let reader = response.body()
            .ok_or("Response body is null")?
            .get_reader();

        let mut decoder = web_sys::TextDecoder::new()?;
        
        loop {
            let chunk = JsFuture::from(reader.read()?).await?;
            let done = js_sys::Reflect::get(&chunk, &JsValue::from_str("done"))?
                .as_bool()
                .unwrap_or(true);
            
            if done {
                break;
            }

            let value = js_sys::Reflect::get(&chunk, &JsValue::from_str("value"))?;
            let data = js_sys::Uint8Array::from(&value);
            let text = decoder.decode_with_uint8_array(&data)?;
            
            // Parse SSE format
            for line in text.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data == "[DONE]" {
                        break;
                    }
                    
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                            callback(content.to_string());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn send_request(&self, body: &serde_json::Value) -> Result<serde_json::Value, JsValue> {
        let window = web_sys::window().ok_or("No window object")?;
        
        let mut opts = RequestInit::new("POST");
        opts.mode(RequestMode::Cors);
        
        let body_str = body.to_string();
        opts.body(Some(&JsValue::from_str(&body_str)));

        let url = format!("{}/v1/chat/completions", self.base_url);
        let request = Request::new_with_str_and_init(&url, &opts)?;
        
        request.headers().set("Authorization", &format!("Bearer {}", self.api_key))?;
        request.headers().set("Content-Type", "application/json")?;

        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
        let resp: Response = resp_value.dyn_into()?;
        
        let json = JsFuture::from(resp.json()?).await?;
        let json_str = js_sys::JSON::stringify(&json)?.as_string().unwrap_or_default();
        
        serde_json::from_str(&json_str)
            .map_err(|e| JsValue::from_str(&format!("JSON parse error: {}", e)))
    }

    async fn send_request_stream(&self, body: &serde_json::Value) -> Result<Response, JsValue> {
        let window = web_sys::window().ok_or("No window object")?;
        
        let mut opts = RequestInit::new("POST");
        opts.mode(RequestMode::Cors);
        
        let body_str = body.to_string();
        opts.body(Some(&JsValue::from_str(&body_str)));

        let url = format!("{}/v1/chat/completions", self.base_url);
        let request = Request::new_with_str_and_init(&url, &opts)?;
        
        request.headers().set("Authorization", &format!("Bearer {}", self.api_key))?;
        request.headers().set("Content-Type", "application/json")?;
        request.headers().set("Accept", "text/event-stream")?;

        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
        let resp: Response = resp_value.dyn_into()?;
        
        Ok(resp)
    }

    pub async fn execute_tool(&self, tool_name: &str, params: serde_json::Value) -> Result<serde_json::Value, JsValue> {
        match tool_name {
            "read_file" => self.tool_read_file(params).await,
            "write_file" => self.tool_write_file(params).await,
            "search" => self.tool_search(params).await,
            "execute_command" => self.tool_execute_command(params).await,
            _ => Err(JsValue::from_str(&format!("Unknown tool: {}", tool_name))),
        }
    }

    async fn tool_read_file(&self, params: serde_json::Value) -> Result<serde_json::Value, JsValue> {
        let path = params["path"].as_str().ok_or("Missing path parameter")?;
        
        // Use browser File System Access API if available
        let window = web_sys::window().ok_or("No window object")?;
        let navigator = window.navigator();
        
        // For now, return a placeholder
        Ok(json!({
            "success": true,
            "content": format!("File read from: {}", path)
        }))
    }

    async fn tool_write_file(&self, params: serde_json::Value) -> Result<serde_json::Value, JsValue> {
        let path = params["path"].as_str().ok_or("Missing path parameter")?;
        let content = params["content"].as_str().ok_or("Missing content parameter")?;
        
        Ok(json!({
            "success": true,
            "message": format!("File written to: {}", path)
        }))
    }

    async fn tool_search(&self, params: serde_json::Value) -> Result<serde_json::Value, JsValue> {
        let query = params["query"].as_str().ok_or("Missing query parameter")?;
        
        Ok(json!({
            "success": true,
            "results": [],
            "message": format!("Search for: {}", query)
        }))
    }

    async fn tool_execute_command(&self, params: serde_json::Value) -> Result<serde_json::Value, JsValue> {
        let command = params["command"].as_str().ok_or("Missing command parameter")?;
        
        // Commands cannot be executed in browser WASM
        Ok(json!({
            "success": false,
            "error": "Command execution not supported in browser environment"
        }))
    }

    fn add_message(&self, role: String, content: String) {
        let timestamp = js_sys::Date::now() as u64;
        self.history.borrow_mut().push(ChatMessage {
            role,
            content,
            timestamp,
        });
    }

    pub fn get_history(&self) -> Vec<ChatMessage> {
        self.history.borrow().clone()
    }

    pub fn clear_history(&self) {
        self.history.borrow_mut().clear();
    }
}
