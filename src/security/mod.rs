//! 安全机制模块
//! 
//! 这个模块实现了完整的安全机制，包括：
//! - 多层防护权限控制体系
//! - 沙箱隔离执行环境
//! - 全面的审计日志系统

pub mod permissions;
pub mod sandbox;
pub mod audit;

pub use permissions::{PermissionManager, PermissionDecision};
pub use sandbox::{SandboxManager, SandboxConfig};
pub use audit::{AuditLogger, AuditEvent};

use crate::error::Result;

/// 安全管理器
/// 
/// 统一管理权限、沙箱和审计日志
#[derive(Debug)]
pub struct SecurityManager {
    /// 权限管理器
    permission_manager: PermissionManager,
    /// 沙箱管理器
    sandbox_manager: SandboxManager,
    /// 审计日志记录器
    audit_logger: AuditLogger,
}

impl SecurityManager {
    /// 创建新的安全管理器
    pub fn new() -> Result<Self> {
        let permission_manager = PermissionManager::new()?;
        let sandbox_manager = SandboxManager::new(SandboxConfig::default())?;
        let audit_logger = AuditLogger::new()?;
        
        Ok(Self {
            permission_manager,
            sandbox_manager,
            audit_logger,
        })
    }
    
    /// 获取权限管理器
    pub fn permissions(&self) -> &PermissionManager {
        &self.permission_manager
    }
    
    /// 获取权限管理器（可变引用）
    pub fn permissions_mut(&mut self) -> &mut PermissionManager {
        &mut self.permission_manager
    }
    
    /// 获取沙箱管理器
    pub fn sandbox(&self) -> &SandboxManager {
        &self.sandbox_manager
    }
    
    /// 获取沙箱管理器（可变引用）
    pub fn sandbox_mut(&mut self) -> &mut SandboxManager {
        &mut self.sandbox_manager
    }
    
    /// 获取审计日志记录器
    pub fn audit(&self) -> &AuditLogger {
        &self.audit_logger
    }
    
    /// 获取审计日志记录器（可变引用）
    pub fn audit_mut(&mut self) -> &mut AuditLogger {
        &mut self.audit_logger
    }
    
    /// 初始化安全系统
    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing security system");
        
        self.permission_manager.load_default_policies().await?;
        self.sandbox_manager.initialize().await?;
        self.audit_logger.initialize().await?;
        
        tracing::info!("Security system initialized successfully");
        Ok(())
    }
    
    /// 关闭安全系统
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down security system");
        
        self.audit_logger.flush().await?;
        self.sandbox_manager.cleanup().await?;
        
        tracing::info!("Security system shutdown complete");
        Ok(())
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new().expect("Failed to create security manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_security_manager_creation() {
        let manager = SecurityManager::new();
        assert!(manager.is_ok());
    }
}
