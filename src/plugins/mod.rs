//! Plugins Module - Complete plugin system
//!
//! Features:
//! - Custom commands
//! - Hook system
//! - Hot loading support
//! - Plugin isolation

pub mod commands;
pub mod hooks;
pub mod loader;
pub mod isolation;
pub mod registry;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};

pub use commands::{PluginCommand, CommandRegistry};
pub use hooks::{Hook, HookManager, HookPoint};
pub use loader::{PluginLoader, LoadedPlugin};
pub use isolation::{PluginSandbox, IsolationConfig};
pub use registry::PluginRegistry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub main: String,
    pub commands: Vec<PluginCommandDef>,
    pub hooks: Vec<String>,
    pub dependencies: HashMap<String, String>,
    pub permissions: Vec<String>,
    pub enabled: bool,
}

impl PluginManifest {
    pub fn new(name: &str, version: &str, main: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            description: None,
            author: None,
            license: None,
            repository: None,
            main: main.to_string(),
            commands: Vec::new(),
            hooks: Vec::new(),
            dependencies: HashMap::new(),
            permissions: Vec::new(),
            enabled: true,
        }
    }
    
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    
    pub fn with_author(mut self, author: &str) -> Self {
        self.author = Some(author.to_string());
        self
    }
    
    pub fn with_command(mut self, command: PluginCommandDef) -> Self {
        self.commands.push(command);
        self
    }
    
    pub fn with_hook(mut self, hook: &str) -> Self {
        self.hooks.push(hook.to_string());
        self
    }
    
    pub fn with_permission(mut self, permission: &str) -> Self {
        self.permissions.push(permission.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommandDef {
    pub name: String,
    pub description: String,
    pub usage: Option<String>,
    pub examples: Vec<String>,
}

impl PluginCommandDef {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            usage: None,
            examples: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub status: PluginStatus,
    pub enabled: bool,
    pub loaded_at: Option<DateTime<Utc>>,
    pub commands_count: usize,
    pub hooks_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginStatus {
    Installed,
    Loaded,
    Error,
    Disabled,
}

pub struct PluginManager {
    registry: Arc<PluginRegistry>,
    loader: Arc<PluginLoader>,
    sandbox: Arc<PluginSandbox>,
    hook_manager: Arc<HookManager>,
    command_registry: Arc<CommandRegistry>,
    plugins_dir: PathBuf,
}

impl PluginManager {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let plugins_dir = home.join(".claude-code").join("plugins");
        
        Self {
            registry: Arc::new(PluginRegistry::new()),
            loader: Arc::new(PluginLoader::new()),
            sandbox: Arc::new(PluginSandbox::new(Default::default())),
            hook_manager: Arc::new(HookManager::new()),
            command_registry: Arc::new(CommandRegistry::new()),
            plugins_dir,
        }
    }
    
    pub fn with_plugins_dir(mut self, dir: PathBuf) -> Self {
        self.plugins_dir = dir;
        self
    }
    
    pub async fn list(&self) -> anyhow::Result<Vec<PluginInfo>> {
        let plugins = self.registry.list().await;
        Ok(plugins)
    }
    
    pub async fn install(&self, source: &str) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.plugins_dir)?;
        
        let plugin_name = source.rsplit('/').next().unwrap_or(source);
        let plugin_dir = self.plugins_dir.join(plugin_name);
        
        if source.starts_with("http") || source.starts_with("git") {
            println!("📥 Cloning plugin from {}...", source);
            let output = tokio::process::Command::new("git")
                .args(["clone", source, &plugin_dir.to_string_lossy()])
                .output()
                .await?;
            
            if !output.status.success() {
                return Err(anyhow::anyhow!("Failed to clone plugin: {}", 
                    String::from_utf8_lossy(&output.stderr)));
            }
        } else if std::path::Path::new(source).exists() {
            println!("📁 Copying plugin from {}...", source);
            fs_extra::dir::copy(source, &self.plugins_dir, &fs_extra::dir::CopyOptions::new())?;
        } else {
            return Err(anyhow::anyhow!("Plugin source not found: {}", source));
        }
        
        let manifest = self.loader.load_manifest(&plugin_dir).await?;
        self.registry.register(manifest).await?;
        
        println!("✅ Plugin installed: {}", plugin_name);
        Ok(())
    }
    
    pub async fn remove(&self, name: &str) -> anyhow::Result<()> {
        self.registry.unregister(name).await?;
        
        let plugin_dir = self.plugins_dir.join(name);
        if plugin_dir.exists() {
            std::fs::remove_dir_all(&plugin_dir)?;
        }
        
        println!("🗑️ Plugin removed: {}", name);
        Ok(())
    }
    
    pub async fn load(&self, name: &str) -> anyhow::Result<()> {
        let manifest = self.registry.get(name).await?
            .ok_or_else(|| anyhow::anyhow!("Plugin not found: {}", name))?;
        
        let plugin_dir = self.plugins_dir.join(name);
        let loaded = self.loader.load(&plugin_dir, &manifest).await?;
        
        for cmd in &manifest.commands {
            self.command_registry.register(cmd.clone()).await;
        }
        
        for hook in &manifest.hooks {
            self.hook_manager.register(hook.parse()?, name).await;
        }
        
        self.registry.set_loaded(name, loaded).await?;
        println!("✅ Plugin loaded: {}", name);
        Ok(())
    }
    
    pub async fn unload(&self, name: &str) -> anyhow::Result<()> {
        let manifest = self.registry.get(name).await?
            .ok_or_else(|| anyhow::anyhow!("Plugin not found: {}", name))?;
        
        for cmd in &manifest.commands {
            self.command_registry.unregister(&cmd.name).await;
        }
        
        for hook in &manifest.hooks {
            self.hook_manager.unregister(&hook.parse()?, name).await;
        }
        
        self.registry.set_unloaded(name).await?;
        println!("⏹️ Plugin unloaded: {}", name);
        Ok(())
    }
    
    pub async fn reload(&self, name: &str) -> anyhow::Result<()> {
        self.unload(name).await?;
        self.load(name).await
    }
    
    pub async fn enable(&self, name: &str) -> anyhow::Result<()> {
        self.registry.set_enabled(name, true).await?;
        self.load(name).await
    }
    
    pub async fn disable(&self, name: &str) -> anyhow::Result<()> {
        self.unload(name).await?;
        self.registry.set_enabled(name, false).await
    }
    
    pub async fn update(&self, name: &str) -> anyhow::Result<()> {
        let plugin_dir = self.plugins_dir.join(name);
        
        if plugin_dir.join(".git").exists() {
            println!("⬆️ Updating plugin: {}", name);
            let output = tokio::process::Command::new("git")
                .args(["pull"])
                .current_dir(&plugin_dir)
                .output()
                .await?;
            
            if output.status.success() {
                self.reload(name).await?;
                println!("✅ Plugin updated: {}", name);
            } else {
                return Err(anyhow::anyhow!("Failed to update plugin: {}",
                    String::from_utf8_lossy(&output.stderr)));
            }
        } else {
            println!("⚠️ Plugin is not a git repository, skipping update");
        }
        
        Ok(())
    }
    
    pub async fn update_all(&self) -> anyhow::Result<()> {
        let plugins = self.registry.list().await;
        for plugin in plugins {
            if plugin.enabled {
                let _ = self.update(&plugin.name).await;
            }
        }
        Ok(())
    }
    
    pub async fn load_all(&self) -> anyhow::Result<()> {
        if !self.plugins_dir.exists() {
            return Ok(());
        }
        
        for entry in std::fs::read_dir(&self.plugins_dir)? {
            let entry = entry?;
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Err(e) = self.load(&name).await {
                    println!("⚠️ Failed to load plugin {}: {}", name, e);
                }
            }
        }
        
        Ok(())
    }
    
    pub fn registry(&self) -> Arc<PluginRegistry> {
        self.registry.clone()
    }
    
    pub fn hook_manager(&self) -> Arc<HookManager> {
        self.hook_manager.clone()
    }
    
    pub fn command_registry(&self) -> Arc<CommandRegistry> {
        self.command_registry.clone()
    }
    
    pub fn sandbox(&self) -> Arc<PluginSandbox> {
        self.sandbox.clone()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
