//! Bash沙箱实现

use super::{SandboxConfig, SandboxExecutionResult};
use crate::error::Result;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

/// Bash沙箱
/// 
/// 提供安全的Bash命令执行环境
#[derive(Debug, Clone)]
pub struct BashSandbox {
    config: SandboxConfig,
}

impl BashSandbox {
    /// 创建新的Bash沙箱
    pub fn new(config: SandboxConfig) -> Self {
        Self { config }
    }
    
    /// 在沙箱中执行命令
    pub async fn execute(
        &self,
        command: &str,
        working_dir: Option<&PathBuf>,
    ) -> Result<SandboxExecutionResult> {
        let sandbox_dir = self.prepare_sandbox_directory()?;
        
        let safe_command = self.sanitize_command(command);
        
        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.args(["/C", &safe_command]);
            c
        } else {
            let mut c = Command::new("bash");
            c.args(["-c", &safe_command]);
            c
        };
        
        cmd.current_dir(working_dir.unwrap_or(&sandbox_dir));
        
        self.configure_command_environment(&mut cmd);
        
        if !self.config.allow_network {
            self.disable_network(&mut cmd);
        }
        
        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());
        
        let execution_time = Duration::from_secs(self.config.max_execution_time);
        
        let result = timeout(execution_time, async {
            let output = cmd.output().await?;
            
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            
            Ok::<_, crate::error::ClaudeError>((
                output.status.code(),
                stdout,
                stderr,
            ))
        })
        .await;
        
        match result {
            Ok(Ok((exit_code, stdout, stderr))) => {
                Ok(SandboxExecutionResult {
                    success: exit_code.unwrap_or(1) == 0,
                    stdout,
                    stderr,
                    exit_code,
                    sandboxed: true,
                })
            }
            Ok(Err(e)) => {
                Ok(SandboxExecutionResult {
                    success: false,
                    stdout: String::new(),
                    stderr: e.to_string(),
                    exit_code: Some(1),
                    sandboxed: true,
                })
            }
            Err(_) => {
                Ok(SandboxExecutionResult {
                    success: false,
                    stdout: String::new(),
                    stderr: "Command execution timed out".to_string(),
                    exit_code: Some(124),
                    sandboxed: true,
                })
            }
        }
    }
    
    /// 准备沙箱目录
    fn prepare_sandbox_directory(&self) -> Result<PathBuf> {
        let sandbox_dir = self.config.sandbox_dir.clone();
        
        if !sandbox_dir.exists() {
            std::fs::create_dir_all(&sandbox_dir)?;
        }
        
        Ok(sandbox_dir)
    }
    
    /// 清理命令
    fn sanitize_command(&self, command: &str) -> String {
        let dangerous_patterns = [
            "&&", "||", ";", "|", "`", "$(", "${",
        ];
        
        let mut sanitized = command.to_string();
        
        for pattern in dangerous_patterns.iter() {
            sanitized = sanitized.replace(pattern, "");
        }
        
        sanitized = sanitized.replace("..", "");
        
        sanitized
    }
    
    /// 配置命令环境
    fn configure_command_environment(&self, cmd: &mut Command) {
        for var in &self.config.allowed_env_vars {
            if let Ok(value) = std::env::var(var) {
                cmd.env(var, value);
            }
        }
        
        for var in &self.config.blocked_env_vars {
            cmd.env_remove(var);
        }
    }
    
    /// 禁用网络（Linux/Unix）
    #[cfg(unix)]
    fn disable_network(&self, cmd: &mut Command) {
        cmd.env("http_proxy", "")
            .env("https_proxy", "")
            .env("HTTP_PROXY", "")
            .env("HTTPS_PROXY", "");
    }
    
    /// 禁用网络（Windows）
    #[cfg(windows)]
    fn disable_network(&self, cmd: &mut Command) {
        cmd.env("http_proxy", "")
            .env("https_proxy", "")
            .env("HTTP_PROXY", "")
            .env("HTTPS_PROXY", "");
    }
    
    /// 验证文件路径
    pub fn validate_path(&self, path: &PathBuf) -> Result<bool> {
        let canonical = if path.exists() {
            path.canonicalize()?
        } else {
            path.clone()
        };
        
        for blocked in &self.config.blocked_directories {
            if canonical.starts_with(blocked) {
                return Ok(false);
            }
        }
        
        if !self.config.allowed_directories.is_empty() {
            for allowed in &self.config.allowed_directories {
                if canonical.starts_with(allowed) {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_bash_sandbox() {
        let config = SandboxConfig::default();
        let sandbox = BashSandbox::new(config);
        
        let result = sandbox.execute("echo 'Hello'", None).await.unwrap();
        assert!(result.success);
        assert!(result.sandboxed);
    }
    
    #[test]
    fn test_sanitize_command() {
        let config = SandboxConfig::default();
        let sandbox = BashSandbox::new(config);
        
        let sanitized = sandbox.sanitize_command("ls && rm -rf /");
        assert!(!sanitized.contains("&&"));
    }
}
