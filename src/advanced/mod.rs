//! Advanced Features Module
//!
//! Features:
//! - SSH connection support
//! - Remote execution
//! - Project initialization

pub mod ssh;
pub mod remote;
pub mod project_init;

use serde::{Deserialize, Serialize};

pub use ssh::{SshClient, SshConfig, SshSession};
pub use remote::{RemoteExecutor, RemoteConfig, RemoteResult};
pub use project_init::{ProjectInitializer, ProjectConfig, ProjectTemplate};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    pub ssh: SshConfig,
    pub remote: RemoteConfig,
    pub project: ProjectConfig,
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            ssh: SshConfig::default(),
            remote: RemoteConfig::default(),
            project: ProjectConfig::default(),
        }
    }
}
