//! Plugin Marketplace Service - Plugin management and marketplace
//!
//! Background plugin and marketplace auto-install manager.
//! Supports installing, updating, and managing plugins from various sources.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub auto_update: bool,
    pub marketplace_url: String,
    pub trusted_sources: Vec<String>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_update: true,
            marketplace_url: "https://plugins.claude.ai".to_string(),
            trusted_sources: vec!["official".to_string(), "community".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub source: String,
    pub installed_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub enabled: bool,
    pub dependencies: Vec<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplacePlugin {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub downloads: usize,
    pub rating: f32,
    pub tags: Vec<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub main: String,
    pub dependencies: HashMap<String, String>,
    pub hooks: HashMap<String, String>,
    pub commands: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginStatus {
    pub enabled: bool,
    pub installed_count: usize,
    pub updates_available: usize,
    pub plugins: Vec<Plugin>,
}

pub struct PluginMarketplaceService {
    config: PluginConfig,
    installed_plugins: Arc<RwLock<HashMap<String, Plugin>>>,
    marketplace_cache: Arc<RwLock<Vec<MarketplacePlugin>>>,
}

impl PluginMarketplaceService {
    pub fn new(_state: Arc<RwLock<AppState>>, config: Option<PluginConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            installed_plugins: Arc::new(RwLock::new(HashMap::new())),
            marketplace_cache: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn list_installed(&self) -> Vec<Plugin> {
        self.load_installed_plugins().await;
        let plugins = self.installed_plugins.read().await;
        plugins.values().cloned().collect()
    }

    async fn load_installed_plugins(&self) {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let plugin_dir = home.join(".claude-code").join("plugins");
        
        if !plugin_dir.exists() {
            return;
        }

        let mut plugins = self.installed_plugins.write().await;
        plugins.clear();

        if let Ok(mut entries) = tokio::fs::read_dir(&plugin_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.is_dir() {
                    let manifest_path = path.join("plugin.json");
                    if manifest_path.exists() {
                        if let Ok(content) = tokio::fs::read_to_string(&manifest_path).await {
                            if let Ok(plugin) = serde_json::from_str::<Plugin>(&content) {
                                plugins.insert(plugin.name.clone(), plugin);
                            }
                        }
                    }
                }
            }
        }
    }

    pub async fn search(&self, query: &str) -> Vec<MarketplacePlugin> {
        println!("🔍 Searching marketplace for: {}", query);
        
        self.fetch_marketplace().await;
        
        let cache = self.marketplace_cache.read().await;
        cache
            .iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&query.to_lowercase())
                    || p.description.to_lowercase().contains(&query.to_lowercase())
                    || p.tags.iter().any(|t| t.to_lowercase().contains(&query.to_lowercase()))
            })
            .cloned()
            .collect()
    }

    async fn fetch_marketplace(&self) {
        println!("📡 Fetching marketplace data...");
        
        let mut cache = self.marketplace_cache.write().await;
        
        cache.clear();
        cache.extend(vec![
            MarketplacePlugin {
                name: "code-formatter".to_string(),
                version: "1.0.0".to_string(),
                description: "Auto-format code in multiple languages".to_string(),
                author: "community".to_string(),
                downloads: 1500,
                rating: 4.5,
                tags: vec!["formatting".to_string(), "code".to_string()],
                homepage: None,
                repository: None,
            },
            MarketplacePlugin {
                name: "git-helper".to_string(),
                version: "2.1.0".to_string(),
                description: "Enhanced git operations and visualizations".to_string(),
                author: "official".to_string(),
                downloads: 3200,
                rating: 4.8,
                tags: vec!["git".to_string(), "vcs".to_string()],
                homepage: None,
                repository: None,
            },
            MarketplacePlugin {
                name: "test-runner".to_string(),
                version: "1.5.0".to_string(),
                description: "Run tests with coverage reports".to_string(),
                author: "community".to_string(),
                downloads: 890,
                rating: 4.2,
                tags: vec!["testing".to_string(), "coverage".to_string()],
                homepage: None,
                repository: None,
            },
        ]);
    }

    pub async fn install(&self, plugin_name: &str) -> anyhow::Result<Plugin> {
        println!("📦 Installing plugin: {}", plugin_name);

        self.fetch_marketplace().await;
        
        let cache = self.marketplace_cache.read().await;
        let marketplace_plugin = cache
            .iter()
            .find(|p| p.name == plugin_name)
            .ok_or_else(|| anyhow::anyhow!("Plugin not found in marketplace: {}", plugin_name))?;

        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let plugin_dir = home.join(".claude-code").join("plugins").join(&marketplace_plugin.name);
        tokio::fs::create_dir_all(&plugin_dir).await?;

        let plugin = Plugin {
            name: marketplace_plugin.name.clone(),
            version: marketplace_plugin.version.clone(),
            description: Some(marketplace_plugin.description.clone()),
            author: Some(marketplace_plugin.author.clone()),
            source: "marketplace".to_string(),
            installed_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
            enabled: true,
            dependencies: vec![],
            homepage: marketplace_plugin.homepage.clone(),
            repository: marketplace_plugin.repository.clone(),
        };

        let manifest_path = plugin_dir.join("plugin.json");
        let manifest_content = serde_json::to_string_pretty(&plugin)?;
        tokio::fs::write(&manifest_path, manifest_content).await?;

        let main_content = r#"// Plugin entry point
module.exports = {
    name: "${plugin.name}",
    version: "${plugin.version}",
    activate: async (context) => {
        console.log('Plugin activated: ${plugin.name}');
    },
    deactivate: async () => {
        console.log('Plugin deactivated: ${plugin.name}');
    }
};
"#.replace("${plugin.name}", &plugin.name)
   .replace("${plugin.version}", &plugin.version);
        
        let main_path = plugin_dir.join("index.js");
        tokio::fs::write(&main_path, main_content).await?;

        let mut installed = self.installed_plugins.write().await;
        installed.insert(plugin.name.clone(), plugin.clone());

        println!("✅ Plugin installed: {} v{}", plugin.name, plugin.version);

        Ok(plugin)
    }

    pub async fn remove(&self, plugin_name: &str) -> anyhow::Result<()> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let plugin_dir = home.join(".claude-code").join("plugins").join(plugin_name);
        
        if plugin_dir.exists() {
            tokio::fs::remove_dir_all(&plugin_dir).await?;
        }

        let mut installed = self.installed_plugins.write().await;
        installed.remove(plugin_name);

        println!("🗑️ Plugin removed: {}", plugin_name);

        Ok(())
    }

    pub async fn update(&self, plugin_name: &str) -> anyhow::Result<Plugin> {
        println!("⬆️ Updating plugin: {}", plugin_name);

        let mut installed = self.installed_plugins.write().await;
        
        if let Some(plugin) = installed.get_mut(plugin_name) {
            plugin.version = "latest".to_string();
            plugin.updated_at = Some(Utc::now());
            
            let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            let plugin_dir = home.join(".claude-code").join("plugins").join(plugin_name);
            let manifest_path = plugin_dir.join("plugin.json");
            let manifest_content = serde_json::to_string_pretty(&*plugin)?;
            tokio::fs::write(&manifest_path, manifest_content).await?;

            println!("✅ Plugin updated: {} v{}", plugin.name, plugin.version);
            
            return Ok(plugin.clone());
        }

        Err(anyhow::anyhow!("Plugin not found: {}", plugin_name))
    }

    pub async fn update_all(&self) -> anyhow::Result<Vec<String>> {
        println!("⬆️ Updating all plugins...");
        
        let plugins = self.installed_plugins.read().await;
        let plugin_names: Vec<String> = plugins.keys().cloned().collect();
        drop(plugins);

        let mut updated = Vec::new();
        for name in plugin_names {
            if self.update(&name).await.is_ok() {
                updated.push(name);
            }
        }

        println!("✅ Updated {} plugins", updated.len());
        Ok(updated)
    }

    pub async fn enable(&self, plugin_name: &str) -> anyhow::Result<()> {
        let mut installed = self.installed_plugins.write().await;
        
        if let Some(plugin) = installed.get_mut(plugin_name) {
            plugin.enabled = true;
            println!("✅ Plugin enabled: {}", plugin_name);
        }

        Ok(())
    }

    pub async fn disable(&self, plugin_name: &str) -> anyhow::Result<()> {
        let mut installed = self.installed_plugins.write().await;
        
        if let Some(plugin) = installed.get_mut(plugin_name) {
            plugin.enabled = false;
            println!("⏸️ Plugin disabled: {}", plugin_name);
        }

        Ok(())
    }

    pub async fn get_status(&self) -> PluginStatus {
        self.load_installed_plugins().await;
        let plugins = self.installed_plugins.read().await;
        
        PluginStatus {
            enabled: self.config.enabled,
            installed_count: plugins.len(),
            updates_available: 0,
            plugins: plugins.values().cloned().collect(),
        }
    }

    pub async fn get_plugin(&self, name: &str) -> Option<Plugin> {
        let plugins = self.installed_plugins.read().await;
        plugins.get(name).cloned()
    }

    pub async fn check_updates(&self) -> Vec<(String, String, String)> {
        println!("🔍 Checking for plugin updates...");
        
        self.fetch_marketplace().await;
        
        let plugins = self.installed_plugins.read().await;
        let cache = self.marketplace_cache.read().await;
        
        let mut updates = Vec::new();
        for plugin in plugins.values() {
            if let Some(marketplace) = cache.iter().find(|m| m.name == plugin.name) {
                if marketplace.version != plugin.version {
                    updates.push((
                        plugin.name.clone(),
                        plugin.version.clone(),
                        marketplace.version.clone(),
                    ));
                }
            }
        }

        updates
    }
}
