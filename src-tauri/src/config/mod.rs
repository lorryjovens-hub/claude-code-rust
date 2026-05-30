//! Global orchestration configuration, loaded once from
//! `config/orchestration.toml` at startup.  All previously hard-coded
//! magic numbers live here now.

pub mod api_endpoints;

use anyhow::Result;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

static CONFIG: OnceCell<OrchestrationConfig> = OnceCell::new();

#[derive(Debug, Clone, Deserialize)]
pub struct OrchestrationConfig {
    pub tool_loop: ToolLoopConfig,
    pub tools: ToolsConfig,
    pub memory: MemoryConfig,
    pub bridge: BridgeConfig,
    pub http: HttpConfig,
    pub logging: LoggingConfig,
    pub telemetry: TelemetryConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolLoopConfig {
    #[serde(default = "default_max_tool_iterations")]
    pub max_tool_iterations: usize,
    #[serde(default = "default_token_threshold")]
    pub token_threshold: usize,
    #[serde(default = "default_recent_messages_to_keep")]
    pub recent_messages_to_keep: usize,
    #[serde(default = "default_max_tokens")]
    pub default_max_tokens: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolsConfig {
    #[serde(default = "default_max_output_bytes")]
    pub max_output_bytes: usize,
    #[serde(default = "default_bash_timeout_secs")]
    pub default_bash_timeout_secs: u64,
    #[serde(default)]
    pub computer_use: ComputerUseConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComputerUseConfig {
    #[serde(default = "default_display_width")]
    pub display_width: u32,
    #[serde(default = "default_display_height")]
    pub display_height: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MemoryConfig {
    #[serde(default = "default_memex_url")]
    pub memex_backend_url: String,
    #[serde(default = "default_importance_threshold")]
    pub auto_ingest_importance_threshold: f64,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub auto_search: bool,
    #[serde(default = "default_true")]
    pub auto_ingest: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BridgeConfig {
    #[serde(default = "default_bridge_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HttpConfig {
    #[serde(default = "default_request_timeout_secs")]
    pub request_timeout_secs: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_retry_base_backoff_ms")]
    pub retry_base_backoff_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub log_dir: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelemetryConfig {
    #[serde(default)]
    pub otlp_endpoint: String,
}

// ---- Default value factories ----

fn default_max_tool_iterations() -> usize { 50 }
fn default_token_threshold() -> usize { 600_000 }
fn default_recent_messages_to_keep() -> usize { 10 }
fn default_max_tokens() -> u32 { 1000000 }
fn default_max_output_bytes() -> usize { 100 * 1024 }
fn default_bash_timeout_secs() -> u64 { 120 }
fn default_display_width() -> u32 { 1920 }
fn default_display_height() -> u32 { 1080 }
fn default_memex_url() -> String { "http://127.0.0.1:8765".to_string() }
fn default_importance_threshold() -> f64 { 0.5 }
fn default_true() -> bool { true }
fn default_bridge_port() -> u16 { 30085 }
fn default_request_timeout_secs() -> u64 { 300 }
fn default_max_retries() -> u32 { 3 }
fn default_retry_base_backoff_ms() -> u64 { 500 }
fn default_log_level() -> String { "info".to_string() }

impl Default for OrchestrationConfig {
    fn default() -> Self {
        Self {
            tool_loop: ToolLoopConfig {
                max_tool_iterations: default_max_tool_iterations(),
                token_threshold: default_token_threshold(),
                recent_messages_to_keep: default_recent_messages_to_keep(),
                default_max_tokens: default_max_tokens(),
            },
            tools: ToolsConfig {
                max_output_bytes: default_max_output_bytes(),
                default_bash_timeout_secs: default_bash_timeout_secs(),
                computer_use: ComputerUseConfig::default(),
            },
            memory: MemoryConfig {
                memex_backend_url: default_memex_url(),
                auto_ingest_importance_threshold: default_importance_threshold(),
                enabled: true,
                auto_search: true,
                auto_ingest: true,
            },
            bridge: BridgeConfig {
                port: default_bridge_port(),
            },
            http: HttpConfig {
                request_timeout_secs: default_request_timeout_secs(),
                max_retries: default_max_retries(),
                retry_base_backoff_ms: default_retry_base_backoff_ms(),
            },
            logging: LoggingConfig {
                level: default_log_level(),
                log_dir: String::new(),
            },
            telemetry: TelemetryConfig {
                otlp_endpoint: String::new(),
            },
        }
    }
}

impl Default for ComputerUseConfig {
    fn default() -> Self {
        Self {
            display_width: default_display_width(),
            display_height: default_display_height(),
        }
    }
}

impl OrchestrationConfig {
    /// Load configuration from the default path or return built-in defaults.
    pub fn load() -> Result<Self> {
        // Try several candidate paths
        let candidates = [
            "config/orchestration.toml",
            "src-tauri/config/orchestration.toml",
            "../config/orchestration.toml",
        ];
        for path in &candidates {
            if Path::new(path).exists() {
                let content = std::fs::read_to_string(path)?;
                return Ok(toml::from_str(&content)?);
            }
        }
        tracing::info!("orchestration.toml not found, using built-in defaults");
        Ok(Self::default())
    }

    /// Initialise the global singleton. Must be called once at startup.
    pub fn init_global() -> Result<()> {
        let cfg = Self::load()?;
        CONFIG.set(cfg).map_err(|_| anyhow::anyhow!("OrchestrationConfig already initialised"))?;
        tracing::info!("Orchestration configuration loaded (global singleton)");
        Ok(())
    }

    /// Borrow the global config. Panics if not yet initialised.
    pub fn get() -> &'static Self {
        CONFIG.get().expect("OrchestrationConfig not initialised — call init_global() first")
    }

    // ---- Convenience accessors (avoids deep .tool_loop.xxx chains everywhere) ----

    pub fn max_tool_iterations() -> usize { Self::get().tool_loop.max_tool_iterations }
    pub fn token_threshold() -> usize { Self::get().tool_loop.token_threshold }
    pub fn recent_messages_to_keep() -> usize { Self::get().tool_loop.recent_messages_to_keep }
    pub fn default_max_tokens() -> u32 { Self::get().tool_loop.default_max_tokens }
    pub fn max_output_bytes() -> usize { Self::get().tools.max_output_bytes }
    pub fn default_bash_timeout_secs() -> u64 { Self::get().tools.default_bash_timeout_secs }
    pub fn max_retries() -> u32 { Self::get().http.max_retries }
    pub fn retry_base_backoff_ms() -> u64 { Self::get().http.retry_base_backoff_ms }
    pub fn request_timeout_secs() -> u64 { Self::get().http.request_timeout_secs }
    pub fn bridge_port() -> u16 { Self::get().bridge.port }
    pub fn memex_backend_url() -> &'static str { &Self::get().memory.memex_backend_url }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub max_tokens: Option<u32>,
    pub supports_vision: bool,
    pub supports_tools: bool,
    pub supports_streaming: bool,
    pub context_window: Option<u32>,
    pub cost_per_1k_input: Option<f64>,
    pub cost_per_1k_output: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub api_key: Option<String>,
    pub base_url: String,
    pub models: Vec<ModelConfig>,
    pub enabled: bool,
    pub is_default: bool,
    pub settings: HashMap<String, serde_json::Value>,
    pub supports_web_search: bool,
    pub web_search_strategy: Option<String>,
    pub web_search_tested_at: Option<u64>,
    pub web_search_test_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub providers: Vec<ProviderConfig>,
    #[serde(default)]
    pub default_provider_id: Option<String>,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            providers: Vec::new(),
            default_provider_id: None,
            theme: None,
            language: None,
        }
    }
}

pub struct ConfigManager {
    config: AppConfig,
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new(config_dir: PathBuf) -> Self {
        let config_path = config_dir.join("config.json");
        let config = if config_path.exists() {
            std::fs::read_to_string(&config_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            AppConfig::default()
        };
        Self { config, config_path }
    }

    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }

    pub fn update_config<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce(&mut AppConfig),
    {
        f(&mut self.config);
        self.save()
    }

    pub fn add_provider(&mut self, provider: ProviderConfig) -> Result<()> {
        self.config.providers.push(provider);
        self.save()
    }

    pub fn remove_provider(&mut self, id: &str) -> Result<()> {
        self.config.providers.retain(|p| p.id != id);
        self.save()
    }

    pub fn get_provider(&self, id: &str) -> Option<&ProviderConfig> {
        self.config.providers.iter().find(|p| p.id == id)
    }

    pub fn get_provider_mut(&mut self, id: &str) -> Option<&mut ProviderConfig> {
        self.config.providers.iter_mut().find(|p| p.id == id)
    }

    pub fn get_default_provider(&self) -> Option<&ProviderConfig> {
        if let Some(ref id) = self.config.default_provider_id {
            self.get_provider(id)
        } else {
            self.config.providers.iter().find(|p| p.is_default).or_else(|| self.config.providers.first())
        }
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self.config)?;
        std::fs::write(&self.config_path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let config = OrchestrationConfig::default();

        // ToolLoopConfig
        assert_eq!(config.tool_loop.max_tool_iterations, 50);
        assert_eq!(config.tool_loop.token_threshold, 600_000);
        assert_eq!(config.tool_loop.recent_messages_to_keep, 10);
        assert_eq!(config.tool_loop.default_max_tokens, 1000000);

        // ToolsConfig
        assert_eq!(config.tools.max_output_bytes, 102_400);
        assert_eq!(config.tools.default_bash_timeout_secs, 120);

        // ComputerUseConfig (nested under tools)
        assert_eq!(config.tools.computer_use.display_width, 1920);
        assert_eq!(config.tools.computer_use.display_height, 1080);

        // MemoryConfig
        assert_eq!(config.memory.memex_backend_url, "http://127.0.0.1:8765");
        assert!((config.memory.auto_ingest_importance_threshold - 0.5).abs() < f64::EPSILON);
        assert!(config.memory.enabled);
        assert!(config.memory.auto_search);
        assert!(config.memory.auto_ingest);

        // BridgeConfig
        assert_eq!(config.bridge.port, 30085);

        // HttpConfig
        assert_eq!(config.http.request_timeout_secs, 300);
        assert_eq!(config.http.max_retries, 3);
        assert_eq!(config.http.retry_base_backoff_ms, 500);

        // LoggingConfig
        assert_eq!(config.logging.level, "info");
        assert!(config.logging.log_dir.is_empty());

        // TelemetryConfig
        assert!(config.telemetry.otlp_endpoint.is_empty());
    }

    #[test]
    fn test_computer_use_default() {
        let config = ComputerUseConfig::default();
        assert_eq!(config.display_width, 1920);
        assert_eq!(config.display_height, 1080);
    }

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert!(config.providers.is_empty());
        assert!(config.default_provider_id.is_none());
        assert!(config.theme.is_none());
        assert!(config.language.is_none());
    }

    #[test]
    fn test_init_global_returns_ok() {
        // init_global uses OnceCell and can only be called once per process.
        // The first call returns Ok(()); subsequent calls return an error
        // indicating the config is already initialised.
        let result = OrchestrationConfig::init_global();
        match result {
            Ok(()) => {
                // First call — verify get() works and values match defaults
                let config = OrchestrationConfig::get();
                assert_eq!(config.tool_loop.max_tool_iterations, 50);
                assert_eq!(config.tools.max_output_bytes, 102_400);
                assert_eq!(config.bridge.port, 30085);
                assert_eq!(config.memory.memex_backend_url, "http://127.0.0.1:8765");
            }
            Err(e) => {
                let msg = e.to_string();
                assert!(
                    msg.contains("already initialised"),
                    "Expected 'already initialised' error, got: {}",
                    msg
                );
            }
        }
    }
}
