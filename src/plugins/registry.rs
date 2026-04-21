//! Plugin Registry - Plugin registration and management

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;


use super::{PluginManifest, PluginInfo, PluginStatus, LoadedPlugin};

pub struct PluginRegistry {
    manifests: Arc<RwLock<HashMap<String, PluginManifest>>>,
    loaded: Arc<RwLock<HashMap<String, LoadedPlugin>>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            manifests: Arc::new(RwLock::new(HashMap::new())),
            loaded: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn register(&self, manifest: PluginManifest) -> anyhow::Result<()> {
        let name = manifest.name.clone();
        let mut manifests = self.manifests.write().await;
        
        if manifests.contains_key(&name) {
            return Err(anyhow::anyhow!("Plugin already registered: {}", name));
        }
        
        manifests.insert(name, manifest);
        Ok(())
    }
    
    pub async fn unregister(&self, name: &str) -> anyhow::Result<()> {
        let mut manifests = self.manifests.write().await;
        let mut loaded = self.loaded.write().await;
        
        manifests.remove(name);
        loaded.remove(name);
        
        Ok(())
    }
    
    pub async fn get(&self, name: &str) -> anyhow::Result<Option<PluginManifest>> {
        let manifests = self.manifests.read().await;
        Ok(manifests.get(name).cloned())
    }
    
    pub async fn list(&self) -> Vec<PluginInfo> {
        let manifests = self.manifests.read().await;
        let loaded = self.loaded.read().await;
        
        manifests.values().map(|m| {
            let loaded_plugin = loaded.get(&m.name);
            PluginInfo {
                name: m.name.clone(),
                version: m.version.clone(),
                description: m.description.clone(),
                author: m.author.clone(),
                status: if loaded_plugin.is_some() {
                    PluginStatus::Loaded
                } else if m.enabled {
                    PluginStatus::Installed
                } else {
                    PluginStatus::Disabled
                },
                enabled: m.enabled,
                loaded_at: loaded_plugin.map(|_p| {
                    chrono::Utc::now()
                }),
                commands_count: m.commands.len(),
                hooks_count: m.hooks.len(),
            }
        }).collect()
    }
    
    pub async fn set_loaded(&self, name: &str, plugin: LoadedPlugin) -> anyhow::Result<()> {
        let mut loaded = self.loaded.write().await;
        loaded.insert(name.to_string(), plugin);
        Ok(())
    }
    
    pub async fn set_unloaded(&self, name: &str) -> anyhow::Result<()> {
        let mut loaded = self.loaded.write().await;
        loaded.remove(name);
        Ok(())
    }
    
    pub async fn set_enabled(&self, name: &str, enabled: bool) -> anyhow::Result<()> {
        let mut manifests = self.manifests.write().await;
        if let Some(manifest) = manifests.get_mut(name) {
            manifest.enabled = enabled;
        }
        Ok(())
    }
    
    pub async fn is_loaded(&self, name: &str) -> bool {
        let loaded = self.loaded.read().await;
        loaded.contains_key(name)
    }
    
    pub async fn is_enabled(&self, name: &str) -> bool {
        let manifests = self.manifests.read().await;
        manifests.get(name).map(|m| m.enabled).unwrap_or(false)
    }
    
    pub async fn get_loaded(&self, name: &str) -> Option<LoadedPlugin> {
        let loaded = self.loaded.read().await;
        loaded.get(name).cloned()
    }
    
    pub async fn count(&self) -> (usize, usize) {
        let manifests = self.manifests.read().await;
        let loaded = self.loaded.read().await;
        (manifests.len(), loaded.len())
    }
    
    pub async fn update_manifest(&self, name: &str, manifest: PluginManifest) -> anyhow::Result<()> {
        let mut manifests = self.manifests.write().await;
        manifests.insert(name.to_string(), manifest);
        Ok(())
    }
    
    pub async fn search(&self, query: &str) -> Vec<PluginInfo> {
        let query_lower = query.to_lowercase();
        let manifests = self.manifests.read().await;
        
        manifests.values()
            .filter(|m| {
                m.name.to_lowercase().contains(&query_lower) ||
                m.description.as_ref().map(|d| d.to_lowercase().contains(&query_lower)).unwrap_or(false) ||
                m.author.as_ref().map(|a| a.to_lowercase().contains(&query_lower)).unwrap_or(false)
            })
            .map(|m| PluginInfo {
                name: m.name.clone(),
                version: m.version.clone(),
                description: m.description.clone(),
                author: m.author.clone(),
                status: PluginStatus::Installed,
                enabled: m.enabled,
                loaded_at: None,
                commands_count: m.commands.len(),
                hooks_count: m.hooks.len(),
            })
            .collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
