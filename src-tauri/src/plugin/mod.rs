use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDescriptor {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub homepage: Option<String>,
    pub enabled: bool,
    pub capabilities: Vec<String>,
    pub config: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub homepage: Option<String>,
    pub capabilities: Vec<String>,
    pub default_config: HashMap<String, Value>,
}

pub trait Plugin: Send + Sync {
    fn descriptor(&self) -> &PluginDescriptor;

    fn init(&self) -> Result<()> {
        Ok(())
    }

    fn on_event(&self, _event_type: &str, _payload: &Value) -> Result<Option<Value>> {
        Ok(None)
    }

    fn on_tool_before(&self, _tool_name: &str, _input: &Value) -> Result<Option<Value>> {
        Ok(None)
    }

    fn on_tool_after(&self, _tool_name: &str, _input: &Value, _output: &Value) -> Result<Option<Value>> {
        Ok(None)
    }

    fn on_message_before(&self, _role: &str, _content: &str) -> Result<Option<String>> {
        Ok(None)
    }

    fn on_message_after(&self, _role: &str, _content: &str, _response: &str) -> Result<Option<String>> {
        Ok(None)
    }

    fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

type PluginFactory = Box<dyn Fn(PluginDescriptor) -> Box<dyn Plugin> + Send + Sync>;

pub struct PluginManager {
    plugins: Arc<RwLock<HashMap<String, Box<dyn Plugin>>>>,
    factories: Arc<RwLock<HashMap<String, PluginFactory>>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            factories: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register_factory<F>(&self, id: &str, factory: F)
    where
        F: Fn(PluginDescriptor) -> Box<dyn Plugin> + Send + Sync + 'static,
    {
        let mut factories = self.factories.blocking_write();
        factories.insert(id.to_string(), Box::new(factory));
    }

    pub async fn load_plugin(&self, descriptor: PluginDescriptor) -> Result<()> {
        let factories = self.factories.read().await;
        if let Some(factory) = factories.get(&descriptor.id) {
            let plugin = factory(descriptor.clone());
            plugin.init()?;
            let mut plugins = self.plugins.write().await;
            plugins.insert(descriptor.id.clone(), plugin);
            Ok(())
        } else {
            Err(anyhow::anyhow!("No factory registered for plugin: {}", descriptor.id))
        }
    }

    pub async fn unload_plugin(&self, id: &str) -> Result<()> {
        let mut plugins = self.plugins.write().await;
        if let Some(plugin) = plugins.remove(id) {
            plugin.shutdown()?;
        }
        Ok(())
    }

    pub async fn get_plugin(&self, id: &str) -> Option<Box<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        plugins.get(id).map(|p| {
            let descriptor = p.descriptor().clone();
            // Return a new box; for real use we'd return Arc<dyn Plugin>
            // This is simplified for the framework version
            let factories = self.factories.blocking_read();
            if let Some(factory) = factories.get(&descriptor.id) {
                factory(descriptor)
            } else {
                panic!("Factory not found for plugin: {}", id)
            }
        })
    }

    pub async fn list_plugins(&self) -> Vec<PluginDescriptor> {
        let plugins = self.plugins.read().await;
        plugins.values().map(|p| p.descriptor().clone()).collect()
    }

    pub async fn emit_event(&self, event_type: &str, payload: &Value) -> Vec<(String, Result<Option<Value>>)> {
        let plugins = self.plugins.read().await;
        let mut results = Vec::new();
        for (id, plugin) in plugins.iter() {
            results.push((id.clone(), plugin.on_event(event_type, payload)));
        }
        results
    }

    pub async fn apply_tool_before(&self, tool_name: &str, input: &Value) -> Result<Option<Value>> {
        let plugins = self.plugins.read().await;
        for plugin in plugins.values() {
            if let Some(modified) = plugin.on_tool_before(tool_name, input)? {
                return Ok(Some(modified));
            }
        }
        Ok(None)
    }

    pub async fn apply_tool_after(&self, tool_name: &str, input: &Value, output: &Value) -> Result<Option<Value>> {
        let plugins = self.plugins.read().await;
        for plugin in plugins.values() {
            if let Some(modified) = plugin.on_tool_after(tool_name, input, output)? {
                return Ok(Some(modified));
            }
        }
        Ok(None)
    }

    pub async fn apply_message_before(&self, role: &str, content: &str) -> Result<Option<String>> {
        let plugins = self.plugins.read().await;
        let mut current = content.to_string();
        for plugin in plugins.values() {
            if let Some(modified) = plugin.on_message_before(role, &current)? {
                current = modified;
            }
        }
        Ok(Some(current))
    }

    pub async fn apply_message_after(&self, role: &str, content: &str, response: &str) -> Result<Option<String>> {
        let plugins = self.plugins.read().await;
        let mut current = response.to_string();
        for plugin in plugins.values() {
            if let Some(modified) = plugin.on_message_after(role, content, &current)? {
                current = modified;
            }
        }
        Ok(Some(current))
    }

    pub async fn is_loaded(&self, id: &str) -> bool {
        let plugins = self.plugins.read().await;
        plugins.contains_key(id)
    }
}