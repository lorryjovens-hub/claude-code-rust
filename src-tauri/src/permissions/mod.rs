pub mod manager;
pub mod rules;
pub mod audit;

pub use manager::{PermissionManager, PermissionMode};
pub use rules::{PermissionScope, PermissionLevel};
pub use audit::AuditLogger;

use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionResult {
    Granted,
    Denied(String),
    RequiresConfirmation(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPermission {
    pub tool_name: String,
    pub allowed: bool,
    pub requires_confirmation: bool,
    pub scope: PermissionScope,
    pub level: PermissionLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionContext {
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub conversation_id: String,
    pub user_id: Option<String>,
    pub workspace_path: Option<String>,
}

pub trait PermissionChecker {
    fn check_permission(&self, context: &PermissionContext) -> PermissionResult;
}

pub type PermissionCheckerRef = Arc<dyn PermissionChecker + Send + Sync>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DangerousTool {
    pub tool_name: String,
    pub action: String,
    pub risk_level: String,
    pub description: String,
}

pub fn get_dangerous_tools() -> Vec<DangerousTool> {
    vec![
        DangerousTool {
            tool_name: "Write".to_string(),
            action: "file_delete".to_string(),
            risk_level: "high".to_string(),
            description: "Delete a file from the filesystem".to_string(),
        },
        DangerousTool {
            tool_name: "Bash".to_string(),
            action: "command_execute".to_string(),
            risk_level: "critical".to_string(),
            description: "Execute a shell command on the system".to_string(),
        },
        DangerousTool {
            tool_name: "ComputerUse".to_string(),
            action: "config_modify".to_string(),
            risk_level: "critical".to_string(),
            description: "Modify system-level configuration files".to_string(),
        },
        DangerousTool {
            tool_name: "Edit".to_string(),
            action: "file_write".to_string(),
            risk_level: "medium".to_string(),
            description: "Write or overwrite a file on the filesystem".to_string(),
        },
        DangerousTool {
            tool_name: "Read".to_string(),
            action: "file_read".to_string(),
            risk_level: "medium".to_string(),
            description: "Read file contents, potentially accessing sensitive data".to_string(),
        },
        DangerousTool {
            tool_name: "network_request".to_string(),
            action: "network_outbound".to_string(),
            risk_level: "medium".to_string(),
            description: "Make an outbound network request".to_string(),
        },
        DangerousTool {
            tool_name: "git_push".to_string(),
            action: "git_push".to_string(),
            risk_level: "high".to_string(),
            description: "Push commits to a remote Git repository".to_string(),
        },
        DangerousTool {
            tool_name: "git_force_push".to_string(),
            action: "git_force_push".to_string(),
            risk_level: "critical".to_string(),
            description: "Force push commits, potentially overwriting remote history".to_string(),
        },
        DangerousTool {
            tool_name: "install_package".to_string(),
            action: "package_install".to_string(),
            risk_level: "medium".to_string(),
            description: "Install a software package on the system".to_string(),
        },
        DangerousTool {
            tool_name: "uninstall_package".to_string(),
            action: "package_uninstall".to_string(),
            risk_level: "medium".to_string(),
            description: "Uninstall a software package from the system".to_string(),
        },
        DangerousTool {
            tool_name: "database_migrate".to_string(),
            action: "db_migrate".to_string(),
            risk_level: "critical".to_string(),
            description: "Run database migration that can modify or delete data".to_string(),
        },
        DangerousTool {
            tool_name: "FileDelete".to_string(),
            action: "file_delete".to_string(),
            risk_level: "critical".to_string(),
            description: "Delete a file or directory from the filesystem".to_string(),
        },
        DangerousTool {
            tool_name: "ProcessKill".to_string(),
            action: "process_kill".to_string(),
            risk_level: "critical".to_string(),
            description: "Kill a running process by PID".to_string(),
        },
    ]
}
