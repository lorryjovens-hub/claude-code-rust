use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfigFile {
    pub orchestrator: OrchestratorSection,
    pub priority_scheduling: PrioritySchedulingSection,
    pub logging: LoggingSection,
    pub queue: QueueSection,
    pub workflow: WorkflowSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorSection {
    pub max_concurrent_agents: usize,
    pub default_model: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritySchedulingSection {
    pub enabled: bool,
    pub priority_adjust_interval_ms: u64,
    pub aging_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSection {
    pub log_level: String,
    pub task_logging_enabled: bool,
    pub dependency_tracking_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueSection {
    pub queue_check_interval_ms: u64,
    pub max_tasks_per_cycle: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSection {
    pub openspace_enabled: bool,
    pub superpowers_enabled: bool,
    pub gstack_enabled: bool,
    pub auto_validate_plan: bool,
    pub auto_execute: bool,
}

impl Default for OrchestratorConfigFile {
    fn default() -> Self {
        Self {
            orchestrator: OrchestratorSection {
                max_concurrent_agents: 8,
                default_model: "claude-sonnet-4-6".to_string(),
                timeout_ms: 300000,
            },
            priority_scheduling: PrioritySchedulingSection {
                enabled: true,
                priority_adjust_interval_ms: 10000,
                aging_factor: 0.1,
            },
            logging: LoggingSection {
                log_level: "trace".to_string(),
                task_logging_enabled: true,
                dependency_tracking_enabled: true,
            },
            queue: QueueSection {
                queue_check_interval_ms: 100,
                max_tasks_per_cycle: 5,
            },
            workflow: WorkflowSection {
                openspace_enabled: true,
                superpowers_enabled: true,
                gstack_enabled: true,
                auto_validate_plan: true,
                auto_execute: false,
            },
        }
    }
}

impl OrchestratorConfigFile {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read config file: {}", e))?;
        let config: OrchestratorConfigFile = toml::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))?;
        Ok(config)
    }

    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load_from_file(path) {
            Ok(config) => config,
            Err(e) => {
                tracing::warn!(module = "OrchestratorConfig", "Failed to load config: {}, using defaults", e);
                Self::default()
            }
        }
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let content = toml::to_string_pretty(self).map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(&path, content).map_err(|e| format!("Failed to write config file: {}", e))?;
        Ok(())
    }
}

impl From<&OrchestratorConfigFile> for super::OrchestratorConfig {
    fn from(file_config: &OrchestratorConfigFile) -> Self {
        Self {
            max_concurrent_agents: file_config.orchestrator.max_concurrent_agents,
            default_model: file_config.orchestrator.default_model.clone(),
            timeout_ms: file_config.orchestrator.timeout_ms,
            enable_priority_scheduling: file_config.priority_scheduling.enabled,
            priority_adjust_interval_ms: file_config.priority_scheduling.priority_adjust_interval_ms,
            aging_factor: file_config.priority_scheduling.aging_factor,
        }
    }
}
