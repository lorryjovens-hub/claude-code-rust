//! Plugin Isolation - Plugin sandbox and isolation

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationConfig {
    pub allowed_paths: Vec<PathBuf>,
    pub denied_paths: Vec<PathBuf>,
    pub allowed_commands: HashSet<String>,
    pub denied_commands: HashSet<String>,
    pub network_access: bool,
    pub max_memory_mb: u32,
    pub max_cpu_percent: u32,
    pub timeout_secs: u32,
    pub env_whitelist: Vec<String>,
}

impl Default for IsolationConfig {
    fn default() -> Self {
        Self {
            allowed_paths: vec![PathBuf::from(".")],
            denied_paths: vec![
                PathBuf::from("/etc/passwd"),
                PathBuf::from("/etc/shadow"),
                PathBuf::from("~/.ssh"),
            ],
            allowed_commands: ["git", "npm", "cargo", "rustc", "python", "node"]
                .iter().map(|s| s.to_string()).collect(),
            denied_commands: ["rm", "sudo", "su", "chmod", "chown"]
                .iter().map(|s| s.to_string()).collect(),
            network_access: true,
            max_memory_mb: 512,
            max_cpu_percent: 50,
            timeout_secs: 30,
            env_whitelist: ["PATH", "HOME", "USER", "TEMP", "TMP"]
                .iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl IsolationConfig {
    pub fn strict() -> Self {
        Self {
            allowed_paths: vec![PathBuf::from(".")],
            denied_paths: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/root"),
                PathBuf::from("~"),
            ],
            allowed_commands: HashSet::new(),
            denied_commands: HashSet::new(),
            network_access: false,
            max_memory_mb: 128,
            max_cpu_percent: 25,
            timeout_secs: 10,
            env_whitelist: vec![],
        }
    }
    
    pub fn permissive() -> Self {
        Self {
            allowed_paths: vec![PathBuf::from("/")],
            denied_paths: vec![],
            allowed_commands: ["*"].iter().map(|s| s.to_string()).collect(),
            denied_commands: HashSet::new(),
            network_access: true,
            max_memory_mb: 1024,
            max_cpu_percent: 100,
            timeout_secs: 300,
            env_whitelist: ["*"].iter().map(|s| s.to_string()).collect(),
        }
    }
    
    pub fn with_allowed_path(mut self, path: PathBuf) -> Self {
        self.allowed_paths.push(path);
        self
    }
    
    pub fn with_denied_path(mut self, path: PathBuf) -> Self {
        self.denied_paths.push(path);
        self
    }
    
    pub fn with_allowed_command(mut self, cmd: &str) -> Self {
        self.allowed_commands.insert(cmd.to_string());
        self
    }
    
    pub fn with_network_access(mut self, allowed: bool) -> Self {
        self.network_access = allowed;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxViolation {
    pub plugin_name: String,
    pub violation_type: ViolationType,
    pub details: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    PathAccess,
    CommandExecution,
    NetworkAccess,
    MemoryLimit,
    CpuLimit,
    Timeout,
    EnvAccess,
}

pub struct PluginSandbox {
    config: IsolationConfig,
    violations: Arc<RwLock<Vec<SandboxViolation>>>,
}

impl PluginSandbox {
    pub fn new(config: IsolationConfig) -> Self {
        Self {
            config,
            violations: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub fn config(&self) -> &IsolationConfig {
        &self.config
    }
    
    pub async fn check_path_access(&self, plugin_name: &str, path: &PathBuf) -> anyhow::Result<bool> {
        let canonical_path = path.canonicalize().unwrap_or_else(|_| path.clone());
        
        for denied in &self.config.denied_paths {
            if canonical_path.starts_with(denied) {
                self.record_violation(plugin_name, ViolationType::PathAccess, 
                    &format!("Attempted access to denied path: {:?}", path)).await;
                return Ok(false);
            }
        }
        
        let allowed = self.config.allowed_paths.iter().any(|allowed| {
            canonical_path.starts_with(allowed) || path.starts_with(allowed)
        });
        
        if !allowed {
            self.record_violation(plugin_name, ViolationType::PathAccess,
                &format!("Attempted access to non-allowed path: {:?}", path)).await;
        }
        
        Ok(allowed)
    }
    
    pub async fn check_command(&self, plugin_name: &str, command: &str) -> anyhow::Result<bool> {
        let cmd_name = command.split_whitespace().next().unwrap_or(command);
        
        if self.config.denied_commands.contains(cmd_name) {
            self.record_violation(plugin_name, ViolationType::CommandExecution,
                &format!("Attempted to execute denied command: {}", cmd_name)).await;
            return Ok(false);
        }
        
        if !self.config.allowed_commands.is_empty() && 
           !self.config.allowed_commands.contains("*") &&
           !self.config.allowed_commands.contains(cmd_name) {
            self.record_violation(plugin_name, ViolationType::CommandExecution,
                &format!("Attempted to execute non-allowed command: {}", cmd_name)).await;
            return Ok(false);
        }
        
        Ok(true)
    }
    
    pub async fn check_network(&self, plugin_name: &str) -> anyhow::Result<bool> {
        if !self.config.network_access {
            self.record_violation(plugin_name, ViolationType::NetworkAccess,
                "Attempted network access").await;
        }
        Ok(self.config.network_access)
    }
    
    pub async fn check_env(&self, plugin_name: &str, env_var: &str) -> anyhow::Result<bool> {
        if self.config.env_whitelist.contains(&"*".to_string()) {
            return Ok(true);
        }
        
        let allowed = self.config.env_whitelist.iter().any(|allowed| {
            env_var == allowed || env_var.starts_with(&format!("{}_", allowed))
        });
        
        if !allowed {
            self.record_violation(plugin_name, ViolationType::EnvAccess,
                &format!("Attempted to access env var: {}", env_var)).await;
        }
        
        Ok(allowed)
    }
    
    pub async fn get_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.config.timeout_secs as u64)
    }
    
    pub async fn get_memory_limit(&self) -> u64 {
        (self.config.max_memory_mb as u64) * 1024 * 1024
    }
    
    async fn record_violation(&self, plugin_name: &str, violation_type: ViolationType, details: &str) {
        let violation = SandboxViolation {
            plugin_name: plugin_name.to_string(),
            violation_type,
            details: details.to_string(),
            timestamp: chrono::Utc::now(),
        };
        
        println!("⚠️ Sandbox violation: {:?}", violation);
        
        let mut violations = self.violations.write().await;
        violations.push(violation);
    }
    
    pub async fn get_violations(&self, plugin_name: Option<&str>) -> Vec<SandboxViolation> {
        let violations = self.violations.read().await;
        match plugin_name {
            Some(name) => violations.iter().filter(|v| v.plugin_name == name).cloned().collect(),
            None => violations.clone(),
        }
    }
    
    pub async fn clear_violations(&self, plugin_name: Option<&str>) {
        let mut violations = self.violations.write().await;
        match plugin_name {
            Some(name) => violations.retain(|v| v.plugin_name != name),
            None => violations.clear(),
        }
    }
}

impl Default for PluginSandbox {
    fn default() -> Self {
        Self::new(Default::default())
    }
}
