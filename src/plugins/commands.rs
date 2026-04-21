//! Plugin Commands - Custom command system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;

use super::PluginCommandDef;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    pub definition: PluginCommandDef,
    pub plugin_name: String,
    pub handler_type: CommandHandlerType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandHandlerType {
    Script { path: String },
    BuiltIn { name: String },
    Wasm { module: String, function: String },
}

impl PluginCommand {
    pub fn new(definition: PluginCommandDef, plugin_name: &str, handler_type: CommandHandlerType) -> Self {
        Self {
            definition,
            plugin_name: plugin_name.to_string(),
            handler_type,
        }
    }
}

#[async_trait]
pub trait CommandHandler: Send + Sync {
    async fn execute(&self, args: HashMap<String, String>) -> anyhow::Result<String>;
}

pub struct CommandRegistry {
    commands: Arc<RwLock<HashMap<String, PluginCommand>>>,
    handlers: Arc<RwLock<HashMap<String, Arc<dyn CommandHandler>>>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: Arc::new(RwLock::new(HashMap::new())),
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn register(&self, definition: PluginCommandDef) {
        let name = definition.name.clone();
        let mut commands = self.commands.write().await;
        commands.insert(name, PluginCommand {
            definition,
            plugin_name: String::new(),
            handler_type: CommandHandlerType::BuiltIn { name: String::new() },
        });
    }
    
    pub async fn register_with_handler(&self, command: PluginCommand, handler: Arc<dyn CommandHandler>) {
        let name = command.definition.name.clone();
        let mut commands = self.commands.write().await;
        let mut handlers = self.handlers.write().await;
        
        handlers.insert(name.clone(), handler);
        commands.insert(name, command);
    }
    
    pub async fn unregister(&self, name: &str) {
        let mut commands = self.commands.write().await;
        let mut handlers = self.handlers.write().await;
        commands.remove(name);
        handlers.remove(name);
    }
    
    pub async fn get(&self, name: &str) -> Option<PluginCommand> {
        let commands = self.commands.read().await;
        commands.get(name).cloned()
    }
    
    pub async fn list(&self) -> Vec<PluginCommand> {
        let commands = self.commands.read().await;
        commands.values().cloned().collect()
    }
    
    pub async fn execute(&self, name: &str, args: HashMap<String, String>) -> anyhow::Result<String> {
        let commands = self.commands.read().await;
        let command = commands.get(name)
            .ok_or_else(|| anyhow::anyhow!("Command not found: {}", name))?;
        
        match &command.handler_type {
            CommandHandlerType::BuiltIn { .. } => {
                let handlers = self.handlers.read().await;
                let handler = handlers.get(name)
                    .ok_or_else(|| anyhow::anyhow!("Handler not found: {}", name))?;
                handler.execute(args).await
            }
            CommandHandlerType::Script { path } => {
                self.execute_script(path, args).await
            }
            CommandHandlerType::Wasm { .. } => {
                Err(anyhow::anyhow!("WASM execution not yet implemented"))
            }
        }
    }
    
    async fn execute_script(&self, path: &str, args: HashMap<String, String>) -> anyhow::Result<String> {
        let mut cmd = tokio::process::Command::new(path);
        
        for (key, value) in args {
            cmd.arg(format!("--{}={}", key, value));
        }
        
        let output = cmd.output().await?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow::anyhow!("Script failed: {}", 
                String::from_utf8_lossy(&output.stderr)))
        }
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BuiltinCommandHandler {
    pub name: String,
    pub description: String,
    pub handler: fn(HashMap<String, String>) -> anyhow::Result<String>,
}

#[async_trait]
impl CommandHandler for BuiltinCommandHandler {
    async fn execute(&self, args: HashMap<String, String>) -> anyhow::Result<String> {
        (self.handler)(args)
    }
}
