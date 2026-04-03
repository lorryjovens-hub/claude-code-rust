//! 权限系统模块
//! 
//! 实现多层防护权限控制体系，包括：
//! - 工具权限：细粒度工具访问控制
//! - 文件权限：目录白名单管理
//! - 命令权限：危险命令识别与审批
//! - 网络权限：域名访问控制

pub mod tool_permissions;
pub mod file_permissions;
pub mod command_permissions;
pub mod network_permissions;
pub mod role_manager;
pub mod types;

pub use tool_permissions::ToolPermissionManager;
pub use file_permissions::FilePermissionManager;
pub use command_permissions::CommandPermissionManager;
pub use network_permissions::NetworkPermissionManager;
pub use role_manager::RoleManager;
pub use types::*;

use crate::error::{ClaudeError, Result};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 权限决策
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionDecision {
    /// 允许
    Allow,
    /// 拒绝
    Deny(String),
    /// 需要审批
    RequireApproval {
        /// 审批级别
        level: ApprovalLevel,
        /// 原因
        reason: String,
    },
}

/// 审批级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalLevel {
    /// 单级审批
    Single,
    /// 双级审批
    Double,
    /// 三级审批
    Triple,
}

/// 权限管理器
/// 
/// 统一管理所有权限类型
#[derive(Debug)]
pub struct PermissionManager {
    /// 工具权限管理器
    tool_manager: Arc<RwLock<ToolPermissionManager>>,
    /// 文件权限管理器
    file_manager: Arc<RwLock<FilePermissionManager>>,
    /// 命令权限管理器
    command_manager: Arc<RwLock<CommandPermissionManager>>,
    /// 网络权限管理器
    network_manager: Arc<RwLock<NetworkPermissionManager>>,
    /// 角色管理器
    role_manager: Arc<RwLock<RoleManager>>,
}

impl PermissionManager {
    /// 创建新的权限管理器
    pub fn new() -> Result<Self> {
        Ok(Self {
            tool_manager: Arc::new(RwLock::new(ToolPermissionManager::new())),
            file_manager: Arc::new(RwLock::new(FilePermissionManager::new())),
            command_manager: Arc::new(RwLock::new(CommandPermissionManager::new())),
            network_manager: Arc::new(RwLock::new(NetworkPermissionManager::new())),
            role_manager: Arc::new(RwLock::new(RoleManager::new())),
        })
    }
    
    /// 加载默认策略
    pub async fn load_default_policies(&mut self) -> Result<()> {
        tracing::info!("Loading default permission policies");
        
        self.tool_manager.write().await.load_default_policies().await?;
        self.file_manager.write().await.load_default_policies().await?;
        self.command_manager.write().await.load_default_policies().await?;
        self.network_manager.write().await.load_default_policies().await?;
        self.role_manager.write().await.load_default_roles().await?;
        
        Ok(())
    }
    
    /// 检查工具权限
    pub async fn check_tool_permission(
        &self,
        tool_name: &str,
        user_id: &str,
        context: &PermissionContext,
    ) -> Result<PermissionDecision> {
        let manager = self.tool_manager.read().await;
        manager.check_permission(tool_name, user_id, context).await
    }
    
    /// 检查文件权限
    pub async fn check_file_permission(
        &self,
        path: &std::path::Path,
        operation: FileOperation,
        user_id: &str,
    ) -> Result<PermissionDecision> {
        let manager = self.file_manager.read().await;
        manager.check_permission(path, operation, user_id).await
    }
    
    /// 检查命令权限
    pub async fn check_command_permission(
        &self,
        command: &str,
        user_id: &str,
    ) -> Result<PermissionDecision> {
        let manager = self.command_manager.read().await;
        manager.check_permission(command, user_id).await
    }
    
    /// 检查网络权限
    pub async fn check_network_permission(
        &self,
        domain: &str,
        user_id: &str,
    ) -> Result<PermissionDecision> {
        let manager = self.network_manager.read().await;
        manager.check_permission(domain, user_id).await
    }
    
    /// 获取工具权限管理器
    pub fn tool_manager(&self) -> Arc<RwLock<ToolPermissionManager>> {
        Arc::clone(&self.tool_manager)
    }
    
    /// 获取文件权限管理器
    pub fn file_manager(&self) -> Arc<RwLock<FilePermissionManager>> {
        Arc::clone(&self.file_manager)
    }
    
    /// 获取命令权限管理器
    pub fn command_manager(&self) -> Arc<RwLock<CommandPermissionManager>> {
        Arc::clone(&self.command_manager)
    }
    
    /// 获取网络权限管理器
    pub fn network_manager(&self) -> Arc<RwLock<NetworkPermissionManager>> {
        Arc::clone(&self.network_manager)
    }
    
    /// 获取角色管理器
    pub fn role_manager(&self) -> Arc<RwLock<RoleManager>> {
        Arc::clone(&self.role_manager)
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new().expect("Failed to create permission manager")
    }
}
