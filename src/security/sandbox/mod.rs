//! 沙箱机制模块
//! 
//! 实现隔离执行环境，确保危险命令安全执行

pub mod bash_sandbox;
pub mod command_checker;
pub mod environment;

pub use bash_sandbox::BashSandbox;
pub use command_checker::CommandChecker;
pub use environment::EnvironmentChecker;

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 沙箱配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// 是否启用沙箱
    pub enabled: bool,
    /// 沙箱目录
    pub sandbox_dir: PathBuf,
    /// 最大执行时间（秒）
    pub max_execution_time: u64,
    /// 最大内存使用（MB）
    pub max_memory_mb: u64,
    /// 允许的环境变量
    pub allowed_env_vars: Vec<String>,
    /// 禁止的环境变量
    pub blocked_env_vars: Vec<String>,
    /// 是否允许网络访问
    pub allow_network: bool,
    /// 是否允许文件系统访问
    pub allow_filesystem: bool,
    /// 允许的目录列表
    pub allowed_directories: Vec<PathBuf>,
    /// 禁止的目录列表
    pub blocked_directories: Vec<PathBuf>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        let temp_dir = std::env::temp_dir().join("claude-code-sandbox");
        
        Self {
            enabled: true,
            sandbox_dir: temp_dir,
            max_execution_time: 300,
            max_memory_mb: 512,
            allowed_env_vars: vec![
                "PATH".to_string(),
                "HOME".to_string(),
                "USER".to_string(),
                "LANG".to_string(),
                "TERM".to_string(),
            ],
            blocked_env_vars: vec![
                "API_KEY".to_string(),
                "SECRET".to_string(),
                "PASSWORD".to_string(),
                "TOKEN".to_string(),
            ],
            allow_network: false,
            allow_filesystem: true,
            allowed_directories: vec![],
            blocked_directories: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/root"),
                PathBuf::from("/sys"),
                PathBuf::from("/proc"),
            ],
        }
    }
}

/// 沙箱管理器
#[derive(Debug)]
pub struct SandboxManager {
    /// 沙箱配置
    config: SandboxConfig,
    /// Bash沙箱
    bash_sandbox: BashSandbox,
    /// 命令检查器
    command_checker: CommandChecker,
    /// 环境检查器
    environment_checker: EnvironmentChecker,
}

impl SandboxManager {
    /// 创建新的沙箱管理器
    pub fn new(config: SandboxConfig) -> Result<Self> {
        let bash_sandbox = BashSandbox::new(config.clone());
        let command_checker = CommandChecker::new();
        let environment_checker = EnvironmentChecker::new(
            config.allowed_env_vars.clone(),
            config.blocked_env_vars.clone(),
        );
        
        Ok(Self {
            config,
            bash_sandbox,
            command_checker,
            environment_checker,
        })
    }
    
    /// 初始化沙箱
    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing sandbox system");
        
        std::fs::create_dir_all(&self.config.sandbox_dir)?;
        
        self.command_checker.load_default_rules().await?;
        
        tracing::info!("Sandbox system initialized");
        Ok(())
    }
    
    /// 清理沙箱
    pub async fn cleanup(&mut self) -> Result<()> {
        tracing::info!("Cleaning up sandbox");
        
        if self.config.sandbox_dir.exists() {
            std::fs::remove_dir_all(&self.config.sandbox_dir)?;
        }
        
        Ok(())
    }
    
    /// 判断是否需要使用沙箱
    pub fn should_use_sandbox(&self, command: &str) -> bool {
        if !self.config.enabled {
            return false;
        }
        
        self.command_checker.is_dangerous_command(command)
    }
    
    /// 在沙箱中执行命令
    pub async fn execute_in_sandbox(
        &self,
        command: &str,
        working_dir: Option<&PathBuf>,
    ) -> Result<SandboxExecutionResult> {
        tracing::info!("Executing command in sandbox: {}", command);
        
        let danger_level = self.command_checker.check_command(command).await;
        tracing::debug!("Command danger level: {:?}", danger_level);
        
        let env_check = self.environment_checker.check_environment();
        if !env_check.safe {
            return Ok(SandboxExecutionResult {
                success: false,
                stdout: String::new(),
                stderr: format!("Environment check failed: {}", env_check.reason),
                exit_code: Some(1),
                sandboxed: true,
            });
        }
        
        if let Some(dir) = working_dir {
            if !self.is_directory_allowed(dir) {
                return Ok(SandboxExecutionResult {
                    success: false,
                    stdout: String::new(),
                    stderr: format!("Directory '{}' is not allowed in sandbox", dir.display()),
                    exit_code: Some(1),
                    sandboxed: true,
                });
            }
        }
        
        let result = self.bash_sandbox.execute(command, working_dir).await?;
        
        Ok(result)
    }
    
    /// 检查目录是否允许
    fn is_directory_allowed(&self, path: &PathBuf) -> bool {
        for blocked in &self.config.blocked_directories {
            if path.starts_with(blocked) {
                return false;
            }
        }
        
        if self.config.allowed_directories.is_empty() {
            return true;
        }
        
        for allowed in &self.config.allowed_directories {
            if path.starts_with(allowed) {
                return true;
            }
        }
        
        false
    }
    
    /// 获取配置
    pub fn config(&self) -> &SandboxConfig {
        &self.config
    }
    
    /// 获取命令检查器
    pub fn command_checker(&self) -> &CommandChecker {
        &self.command_checker
    }
    
    /// 获取环境检查器
    pub fn environment_checker(&self) -> &EnvironmentChecker {
        &self.environment_checker
    }
}

/// 沙箱执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxExecutionResult {
    /// 是否成功
    pub success: bool,
    /// 标准输出
    pub stdout: String,
    /// 标准错误
    pub stderr: String,
    /// 退出码
    pub exit_code: Option<i32>,
    /// 是否在沙箱中执行
    pub sandboxed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sandbox_config() {
        let config = SandboxConfig::default();
        assert!(config.enabled);
        assert!(!config.allow_network);
    }
    
    #[test]
    fn test_should_use_sandbox() {
        let config = SandboxConfig::default();
        let manager = SandboxManager::new(config).unwrap();
        
        assert!(manager.should_use_sandbox("rm -rf /"));
        assert!(!manager.should_use_sandbox("ls -la"));
    }
}
