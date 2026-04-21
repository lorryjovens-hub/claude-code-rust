//! Plugin Loader - Hot loading support

use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

use super::{PluginManifest, PluginStatus};

pub struct PluginLoader {
    loaded: Arc<RwLock<Vec<LoadedPlugin>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedPlugin {
    pub name: String,
    pub manifest: PluginManifest,
    pub status: PluginStatus,
    pub module: Option<PluginModule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginModule {
    Native,
    Wasm { bytes: Vec<u8> },
    Script { path: String },
}

impl PluginLoader {
    pub fn new() -> Self {
        Self {
            loaded: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub async fn load_manifest(&self, plugin_dir: &Path) -> anyhow::Result<PluginManifest> {
        let manifest_path = plugin_dir.join("plugin.json");
        
        if !manifest_path.exists() {
            return Err(anyhow::anyhow!("Plugin manifest not found: {:?}", manifest_path));
        }
        
        let content = tokio::fs::read_to_string(&manifest_path).await?;
        let manifest: PluginManifest = serde_json::from_str(&content)?;
        
        Ok(manifest)
    }
    
    pub async fn load(&self, plugin_dir: &Path, manifest: &PluginManifest) -> anyhow::Result<LoadedPlugin> {
        let main_path = plugin_dir.join(&manifest.main);
        
        let module = if main_path.extension().map(|e| e == "wasm").unwrap_or(false) {
            let bytes = tokio::fs::read(&main_path).await?;
            PluginModule::Wasm { bytes }
        } else if main_path.extension().map(|e| e == "rs").unwrap_or(false) {
            PluginModule::Native
        } else {
            PluginModule::Script { path: main_path.to_string_lossy().to_string() }
        };
        
        let loaded = LoadedPlugin {
            name: manifest.name.clone(),
            manifest: manifest.clone(),
            status: PluginStatus::Loaded,
            module: Some(module),
        };
        
        let mut loaded_list = self.loaded.write().await;
        loaded_list.push(loaded.clone());
        
        Ok(loaded)
    }
    
    pub async fn unload(&self, name: &str) -> anyhow::Result<()> {
        let mut loaded_list = self.loaded.write().await;
        loaded_list.retain(|p| p.name != name);
        Ok(())
    }
    
    pub async fn reload(&self, plugin_dir: &Path, manifest: &PluginManifest) -> anyhow::Result<LoadedPlugin> {
        self.unload(&manifest.name).await?;
        self.load(plugin_dir, manifest).await
    }
    
    pub async fn get(&self, name: &str) -> Option<LoadedPlugin> {
        let loaded_list = self.loaded.read().await;
        loaded_list.iter().find(|p| p.name == name).cloned()
    }
    
    pub async fn list(&self) -> Vec<LoadedPlugin> {
        let loaded_list = self.loaded.read().await;
        loaded_list.clone()
    }
    
    pub async fn is_loaded(&self, name: &str) -> bool {
        let loaded_list = self.loaded.read().await;
        loaded_list.iter().any(|p| p.name == name)
    }
    
    pub async fn hot_reload(&self, plugin_dir: &Path) -> anyhow::Result<Option<LoadedPlugin>> {
        let manifest = self.load_manifest(plugin_dir).await?;
        
        let current = self.get(&manifest.name).await;
        if let Some(current) = current {
            if current.manifest.version != manifest.version {
                println!("🔄 Hot reloading plugin: {} ({} -> {})", 
                    manifest.name, current.manifest.version, manifest.version);
                return Ok(Some(self.reload(plugin_dir, &manifest).await?));
            }
        }
        
        Ok(None)
    }
    
    pub async fn watch(&self, plugin_dir: &Path) -> anyhow::Result<()> {
        let manifest = self.load_manifest(plugin_dir).await?;
        let main_path = plugin_dir.join(&manifest.main);
        
        println!("👁️ Watching plugin: {} at {:?}", manifest.name, main_path);
        
        Ok(())
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}
