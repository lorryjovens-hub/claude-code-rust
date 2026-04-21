//! Plugin Hooks - Hook system for plugin events

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HookPoint {
    PreCommand,
    PostCommand,
    PreQuery,
    PostQuery,
    PreFileRead,
    PostFileRead,
    PreFileWrite,
    PostFileWrite,
    PreToolExecution,
    PostToolExecution,
    OnSessionStart,
    OnSessionEnd,
    OnError,
    OnMemoryConsolidation,
    OnPluginLoad,
    OnPluginUnload,
    Custom(String),
}

impl FromStr for HookPoint {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pre_command" => Ok(HookPoint::PreCommand),
            "post_command" => Ok(HookPoint::PostCommand),
            "pre_query" => Ok(HookPoint::PreQuery),
            "post_query" => Ok(HookPoint::PostQuery),
            "pre_file_read" => Ok(HookPoint::PreFileRead),
            "post_file_read" => Ok(HookPoint::PostFileRead),
            "pre_file_write" => Ok(HookPoint::PreFileWrite),
            "post_file_write" => Ok(HookPoint::PostFileWrite),
            "pre_tool_execution" => Ok(HookPoint::PreToolExecution),
            "post_tool_execution" => Ok(HookPoint::PostToolExecution),
            "on_session_start" => Ok(HookPoint::OnSessionStart),
            "on_session_end" => Ok(HookPoint::OnSessionEnd),
            "on_error" => Ok(HookPoint::OnError),
            "on_memory_consolidation" => Ok(HookPoint::OnMemoryConsolidation),
            "on_plugin_load" => Ok(HookPoint::OnPluginLoad),
            "on_plugin_unload" => Ok(HookPoint::OnPluginUnload),
            s if s.starts_with("custom:") => Ok(HookPoint::Custom(s[7..].to_string())),
            _ => Err(anyhow::anyhow!("Unknown hook point: {}", s)),
        }
    }
}

impl std::fmt::Display for HookPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookPoint::PreCommand => write!(f, "pre_command"),
            HookPoint::PostCommand => write!(f, "post_command"),
            HookPoint::PreQuery => write!(f, "pre_query"),
            HookPoint::PostQuery => write!(f, "post_query"),
            HookPoint::PreFileRead => write!(f, "pre_file_read"),
            HookPoint::PostFileRead => write!(f, "post_file_read"),
            HookPoint::PreFileWrite => write!(f, "pre_file_write"),
            HookPoint::PostFileWrite => write!(f, "post_file_write"),
            HookPoint::PreToolExecution => write!(f, "pre_tool_execution"),
            HookPoint::PostToolExecution => write!(f, "post_tool_execution"),
            HookPoint::OnSessionStart => write!(f, "on_session_start"),
            HookPoint::OnSessionEnd => write!(f, "on_session_end"),
            HookPoint::OnError => write!(f, "on_error"),
            HookPoint::OnMemoryConsolidation => write!(f, "on_memory_consolidation"),
            HookPoint::OnPluginLoad => write!(f, "on_plugin_load"),
            HookPoint::OnPluginUnload => write!(f, "on_plugin_unload"),
            HookPoint::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {
    pub point: HookPoint,
    pub data: HashMap<String, serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl HookContext {
    pub fn new(point: HookPoint) -> Self {
        Self {
            point,
            data: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }
    
    pub fn with_data(mut self, key: &str, value: serde_json::Value) -> Self {
        self.data.insert(key.to_string(), value);
        self
    }
    
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    pub proceed: bool,
    pub modified_data: Option<HashMap<String, serde_json::Value>>,
    pub error: Option<String>,
}

impl HookResult {
    pub fn proceed() -> Self {
        Self {
            proceed: true,
            modified_data: None,
            error: None,
        }
    }
    
    pub fn stop() -> Self {
        Self {
            proceed: false,
            modified_data: None,
            error: None,
        }
    }
    
    pub fn with_error(error: &str) -> Self {
        Self {
            proceed: false,
            modified_data: None,
            error: Some(error.to_string()),
        }
    }
    
    pub fn with_modified_data(mut self, data: HashMap<String, serde_json::Value>) -> Self {
        self.modified_data = Some(data);
        self
    }
}

#[derive(Debug, Clone)]
pub struct Hook {
    pub plugin_name: String,
    pub point: HookPoint,
    pub priority: i32,
    pub handler_type: HookHandlerType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookHandlerType {
    Script { path: String },
    BuiltIn { name: String },
    Async { name: String },
}

pub struct HookManager {
    hooks: Arc<RwLock<HashMap<HookPoint, Vec<Hook>>>>,
}

impl HookManager {
    pub fn new() -> Self {
        Self {
            hooks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn register(&self, point: HookPoint, plugin_name: &str) {
        let mut hooks = self.hooks.write().await;
        hooks.entry(point.clone())
            .or_insert_with(Vec::new)
            .push(Hook {
                plugin_name: plugin_name.to_string(),
                point,
                priority: 0,
                handler_type: HookHandlerType::BuiltIn { name: String::new() },
            });
    }
    
    pub async fn register_hook(&self, hook: Hook) {
        let mut hooks = self.hooks.write().await;
        let point = hook.point.clone();
        hooks.entry(point.clone())
            .or_insert_with(Vec::new)
            .push(hook);
        
        if let Some(hook_list) = hooks.get_mut(&point) {
            hook_list.sort_by_key(|h| h.priority);
        }
    }
    
    pub async fn unregister(&self, point: &HookPoint, plugin_name: &str) {
        let mut hooks = self.hooks.write().await;
        if let Some(hook_list) = hooks.get_mut(point) {
            hook_list.retain(|h| h.plugin_name != plugin_name);
        }
    }
    
    pub async fn unregister_all(&self, plugin_name: &str) {
        let mut hooks = self.hooks.write().await;
        for hook_list in hooks.values_mut() {
            hook_list.retain(|h| h.plugin_name != plugin_name);
        }
    }
    
    pub async fn get_hooks(&self, point: &HookPoint) -> Vec<Hook> {
        let hooks = self.hooks.read().await;
        hooks.get(point).cloned().unwrap_or_default()
    }
    
    pub async fn execute(&self, context: HookContext) -> HookResult {
        let hooks = self.hooks.read().await;
        let hook_list = hooks.get(&context.point).cloned().unwrap_or_default();
        drop(hooks);
        
        let mut current_context = context;
        
        for hook in hook_list {
            let result = self.execute_hook(&hook, &current_context).await;
            
            if !result.proceed {
                return result;
            }
            
            if let Some(modified_data) = result.modified_data {
                current_context.data = modified_data;
            }
        }
        
        HookResult::proceed()
    }
    
    async fn execute_hook(&self, hook: &Hook, context: &HookContext) -> HookResult {
        match &hook.handler_type {
            HookHandlerType::Script { path } => {
                let json_context = serde_json::to_string(&context).unwrap_or_default();
                let output = tokio::process::Command::new(path)
                    .arg(&json_context)
                    .output()
                    .await;
                
                match output {
                    Ok(output) if output.status.success() => {
                        match serde_json::from_slice(&output.stdout) {
                            Ok(result) => result,
                            Err(_) => HookResult::proceed(),
                        }
                    }
                    Ok(output) => HookResult::with_error(&String::from_utf8_lossy(&output.stderr)),
                    Err(e) => HookResult::with_error(&e.to_string()),
                }
            }
            HookHandlerType::BuiltIn { name } => {
                self.execute_builtin_hook(name, context).await
            }
            HookHandlerType::Async { name } => {
                self.execute_builtin_hook(name, context).await
            }
        }
    }
    
    async fn execute_builtin_hook(&self, name: &str, context: &HookContext) -> HookResult {
        match name {
            "log" => {
                println!("[Hook] {:?}: {:?}", context.point, context.data);
                HookResult::proceed()
            }
            "validate" => {
                HookResult::proceed()
            }
            _ => HookResult::proceed(),
        }
    }
    
    pub async fn list_all(&self) -> HashMap<HookPoint, Vec<Hook>> {
        let hooks = self.hooks.read().await;
        hooks.clone()
    }
}

impl Default for HookManager {
    fn default() -> Self {
        Self::new()
    }
}
