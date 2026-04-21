//! WebAssembly Module - Browser-compatible Claude Code
//!
//! This module provides WebAssembly bindings for running Claude Code
//! in web browsers with JavaScript interop.

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

pub mod client;
pub mod storage;
pub mod bridge;

pub use client::WasmApiClient;
pub use storage::BrowserStorage;
pub use bridge::JsBridge;

/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
}

/// Main WASM API for JavaScript interop
#[wasm_bindgen]
pub struct ClaudeCodeWasm {
    client: WasmApiClient,
    storage: BrowserStorage,
}

#[wasm_bindgen]
impl ClaudeCodeWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            client: WasmApiClient::new(api_key, base_url),
            storage: BrowserStorage::new(),
        }
    }

    /// Send a chat message and get response
    pub async fn chat(&self, message: String) -> Result<JsValue, JsValue> {
        let response = self.client
            .chat(message)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        serde_wasm_bindgen::to_value(&response)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Stream chat response
    pub async fn chat_stream(&self, message: String, callback: js_sys::Function) -> Result<(), JsValue> {
        self.client
            .chat_stream(message, |chunk| {
                let _ = callback.call1(&JsValue::NULL, &JsValue::from_str(&chunk));
            })
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Save configuration to browser storage
    pub fn save_config(&self, key: String, value: String) -> Result<(), JsValue> {
        self.storage
            .set(&key, &value)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Load configuration from browser storage
    pub fn load_config(&self, key: String) -> Option<String> {
        self.storage.get(&key).ok().flatten()
    }

    /// Execute a tool in the browser environment
    pub async fn execute_tool(&self, tool_name: String, params: JsValue) -> Result<JsValue, JsValue> {
        let params: serde_json::Value = serde_wasm_bindgen::from_value(params)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        let result = self.client
            .execute_tool(&tool_name, params)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get conversation history
    pub fn get_history(&self) -> Result<JsValue, JsValue> {
        let history = self.client.get_history();
        serde_wasm_bindgen::to_value(&history)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Clear conversation history
    pub fn clear_history(&self) {
        self.client.clear_history();
    }
}

/// Tool execution result for JavaScript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

/// Chat message for JavaScript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageJs {
    pub role: String,
    pub content: String,
    pub timestamp: u64,
}

/// Initialize panic hook for better error messages
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Version information
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
