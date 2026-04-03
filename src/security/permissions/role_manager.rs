//! 角色管理器

use super::{Role, UserPermissionConfig, PermissionRule};
use crate::error::Result;
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use tokio::sync::RwLock;

/// 角色管理器
/// 
/// 管理角色和用户权限分配
#[derive(Debug)]
pub struct RoleManager {
    /// 角色定义
    roles: RwLock<HashMap<String, Role>>,
    /// 用户权限配置
    user_configs: RwLock<HashMap<String, UserPermissionConfig>>,
    /// 角色继承缓存
    inheritance_cache: RwLock<HashMap<String, Vec<String>>>,
}

impl RoleManager {
    /// 创建新的角色管理器
    pub fn new() -> Self {
        Self {
            roles: RwLock::new(HashMap::new()),
            user_configs: RwLock::new(HashMap::new()),
            inheritance_cache: RwLock::new(HashMap::new()),
        }
    }
    
    /// 加载默认角色
    pub async fn load_default_roles(&mut self) -> Result<()> {
        let mut roles = self.roles.write().await;
        
        roles.insert("admin".to_string(), Role {
            id: "admin".to_string(),
            name: "Administrator".to_string(),
            description: "Full system access".to_string(),
            permissions: vec![
                "tool:*".to_string(),
                "file:*".to_string(),
                "command:*".to_string(),
                "network:*".to_string(),
            ],
            inherits: vec![],
            is_system: true,
        });
        
        roles.insert("developer".to_string(), Role {
            id: "developer".to_string(),
            name: "Developer".to_string(),
            description: "Development access".to_string(),
            permissions: vec![
                "tool:read".to_string(),
                "tool:write".to_string(),
                "tool:edit".to_string(),
                "tool:bash".to_string(),
                "file:read".to_string(),
                "file:write".to_string(),
                "command:execute".to_string(),
                "network:api".to_string(),
            ],
            inherits: vec!["viewer".to_string()],
            is_system: true,
        });
        
        roles.insert("viewer".to_string(), Role {
            id: "viewer".to_string(),
            name: "Viewer".to_string(),
            description: "Read-only access".to_string(),
            permissions: vec![
                "tool:read".to_string(),
                "file:read".to_string(),
            ],
            inherits: vec![],
            is_system: true,
        });
        
        roles.insert("restricted".to_string(), Role {
            id: "restricted".to_string(),
            name: "Restricted".to_string(),
            description: "Limited access with approval".to_string(),
            permissions: vec![
                "tool:read".to_string(),
            ],
            inherits: vec![],
            is_system: true,
        });
        
        roles.insert("guest".to_string(), Role {
            id: "guest".to_string(),
            name: "Guest".to_string(),
            description: "Minimal access".to_string(),
            permissions: vec![],
            inherits: vec![],
            is_system: true,
        });
        
        Ok(())
    }
    
    /// 获取用户的所有权限
    pub async fn get_user_permissions(&self, user_id: &str) -> Result<Vec<String>> {
        let user_configs = self.user_configs.read().await;
        let roles_map = self.roles.read().await;
        
        let mut all_permissions = HashSet::new();
        
        if let Some(config) = user_configs.get(user_id) {
            for role_id in &config.roles {
                if let Some(role) = roles_map.get(role_id) {
                    let role_permissions = self.get_role_permissions(role, &roles_map).await;
                    all_permissions.extend(role_permissions);
                }
            }
            
            all_permissions.extend(config.permissions.clone());
        }
        
        Ok(all_permissions.into_iter().collect())
    }
    
    /// 获取角色的所有权限（包括继承的）
    async fn get_role_permissions(
        &self,
        role: &Role,
        roles_map: &HashMap<String, Role>,
    ) -> Vec<String> {
        let mut permissions = role.permissions.clone();
        
        for inherited_role_id in &role.inherits {
            if let Some(inherited_role) = roles_map.get(inherited_role_id) {
                let inherited_permissions = Box::pin(
                    self.get_role_permissions(inherited_role, roles_map)
                ).await;
                permissions.extend(inherited_permissions);
            }
        }
        
        permissions
    }
    
    /// 检查用户是否有特定权限
    pub async fn has_permission(&self, user_id: &str, permission: &str) -> Result<bool> {
        let permissions = self.get_user_permissions(user_id).await?;
        
        if permissions.contains(&"*".to_string()) {
            return Ok(true);
        }
        
        if permissions.contains(&permission.to_string()) {
            return Ok(true);
        }
        
        let parts: Vec<&str> = permission.split(':').collect();
        if parts.len() == 2 {
            let wildcard = format!("{}:*", parts[0]);
            if permissions.contains(&wildcard) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// 为用户分配角色
    pub async fn assign_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        let mut user_configs = self.user_configs.write().await;
        
        let config = user_configs
            .entry(user_id.to_string())
            .or_insert_with(|| UserPermissionConfig {
                user_id: user_id.to_string(),
                roles: Vec::new(),
                permissions: Vec::new(),
                custom_rules: Vec::new(),
            });
        
        if !config.roles.contains(&role_id.to_string()) {
            config.roles.push(role_id.to_string());
        }
        
        Ok(())
    }
    
    /// 移除用户的角色
    pub async fn remove_role(&self, user_id: &str, role_id: &str) -> Result<()> {
        let mut user_configs = self.user_configs.write().await;
        
        if let Some(config) = user_configs.get_mut(user_id) {
            config.roles.retain(|r| r != role_id);
        }
        
        Ok(())
    }
    
    /// 为用户添加权限
    pub async fn grant_permission(&self, user_id: &str, permission: &str) -> Result<()> {
        let mut user_configs = self.user_configs.write().await;
        
        let config = user_configs
            .entry(user_id.to_string())
            .or_insert_with(|| UserPermissionConfig {
                user_id: user_id.to_string(),
                roles: Vec::new(),
                permissions: Vec::new(),
                custom_rules: Vec::new(),
            });
        
        if !config.permissions.contains(&permission.to_string()) {
            config.permissions.push(permission.to_string());
        }
        
        Ok(())
    }
    
    /// 移除用户的权限
    pub async fn revoke_permission(&self, user_id: &str, permission: &str) -> Result<()> {
        let mut user_configs = self.user_configs.write().await;
        
        if let Some(config) = user_configs.get_mut(user_id) {
            config.permissions.retain(|p| p != permission);
        }
        
        Ok(())
    }
    
    /// 创建角色
    pub async fn create_role(&self, role: Role) -> Result<()> {
        let mut roles = self.roles.write().await;
        roles.insert(role.id.clone(), role);
        
        let mut cache = self.inheritance_cache.write().await;
        cache.clear();
        
        Ok(())
    }
    
    /// 删除角色
    pub async fn delete_role(&self, role_id: &str) -> Result<()> {
        let mut roles = self.roles.write().await;
        
        if let Some(role) = roles.get(role_id) {
            if role.is_system {
                return Err(crate::error::ClaudeError::Permission(
                    "Cannot delete system role".to_string()
                ));
            }
        }
        
        roles.remove(role_id);
        
        let mut cache = self.inheritance_cache.write().await;
        cache.clear();
        
        Ok(())
    }
    
    /// 获取角色信息
    pub async fn get_role(&self, role_id: &str) -> Option<Role> {
        let roles = self.roles.read().await;
        roles.get(role_id).cloned()
    }
    
    /// 获取所有角色
    pub async fn get_all_roles(&self) -> Vec<Role> {
        let roles = self.roles.read().await;
        roles.values().cloned().collect()
    }
    
    /// 获取用户的角色列表
    pub async fn get_user_roles(&self, user_id: &str) -> Result<Vec<String>> {
        let user_configs = self.user_configs.read().await;
        
        if let Some(config) = user_configs.get(user_id) {
            Ok(config.roles.clone())
        } else {
            Ok(Vec::new())
        }
    }
    
    /// 添加自定义规则
    pub async fn add_custom_rule(
        &self,
        user_id: &str,
        rule: PermissionRule,
    ) -> Result<()> {
        let mut user_configs = self.user_configs.write().await;
        
        let config = user_configs
            .entry(user_id.to_string())
            .or_insert_with(|| UserPermissionConfig {
                user_id: user_id.to_string(),
                roles: Vec::new(),
                permissions: Vec::new(),
                custom_rules: Vec::new(),
            });
        
        config.custom_rules.push(rule);
        
        Ok(())
    }
}

impl Default for RoleManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_role_manager() {
        let mut manager = RoleManager::new();
        manager.load_default_roles().await.unwrap();
        
        let role = manager.get_role("admin").await;
        assert!(role.is_some());
    }
    
    #[tokio::test]
    async fn test_assign_role() {
        let mut manager = RoleManager::new();
        manager.load_default_roles().await.unwrap();
        
        manager.assign_role("user1", "developer").await.unwrap();
        
        let roles = manager.get_user_roles("user1").await.unwrap();
        assert!(roles.contains(&"developer".to_string()));
    }
    
    #[tokio::test]
    async fn test_has_permission() {
        let mut manager = RoleManager::new();
        manager.load_default_roles().await.unwrap();
        
        manager.assign_role("user1", "viewer").await.unwrap();
        
        let has = manager.has_permission("user1", "tool:read").await.unwrap();
        assert!(has);
        
        let has = manager.has_permission("user1", "tool:write").await.unwrap();
        assert!(!has);
    }
}
