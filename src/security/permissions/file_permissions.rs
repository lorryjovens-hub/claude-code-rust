//! 文件权限管理器

use super::{FileOperation, PermissionDecision, PermissionRule, ApprovalLevel};
use crate::error::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;

/// 目录白名单条目
#[derive(Debug, Clone)]
pub struct DirectoryWhitelistEntry {
    /// 目录路径
    pub path: PathBuf,
    /// 允许的操作
    pub allowed_operations: Vec<FileOperation>,
    /// 是否递归
    pub recursive: bool,
    /// 描述
    pub description: String,
}

/// 文件权限管理器
/// 
/// 管理文件访问权限，实现目录白名单管理
#[derive(Debug)]
pub struct FilePermissionManager {
    /// 目录白名单
    whitelist: RwLock<Vec<DirectoryWhitelistEntry>>,
    /// 黑名单
    blacklist: RwLock<Vec<PathBuf>>,
    /// 权限规则
    rules: RwLock<HashMap<String, PermissionRule>>,
    /// 受保护的文件模式
    protected_patterns: RwLock<Vec<String>>,
}

impl FilePermissionManager {
    /// 创建新的文件权限管理器
    pub fn new() -> Self {
        Self {
            whitelist: RwLock::new(Vec::new()),
            blacklist: RwLock::new(Vec::new()),
            rules: RwLock::new(HashMap::new()),
            protected_patterns: RwLock::new(Vec::new()),
        }
    }
    
    /// 加载默认策略
    pub async fn load_default_policies(&mut self) -> Result<()> {
        let mut whitelist = self.whitelist.write().await;
        
        if let Some(home) = dirs::home_dir() {
            whitelist.push(DirectoryWhitelistEntry {
                path: home.clone(),
                allowed_operations: vec![
                    FileOperation::Read,
                    FileOperation::Write,
                    FileOperation::List,
                ],
                recursive: true,
                description: "Home directory".to_string(),
            });
        }
        
        whitelist.push(DirectoryWhitelistEntry {
            path: PathBuf::from("."),
            allowed_operations: vec![
                FileOperation::Read,
                FileOperation::Write,
                FileOperation::List,
                FileOperation::Delete,
            ],
            recursive: true,
            description: "Current working directory".to_string(),
        });
        
        let mut protected = self.protected_patterns.write().await;
        *protected = vec![
            "**/.env".to_string(),
            "**/.git/**".to_string(),
            "**/id_rsa".to_string(),
            "**/id_ed25519".to_string(),
            "**/*.pem".to_string(),
            "**/*.key".to_string(),
        ];
        
        let mut blacklist = self.blacklist.write().await;
        blacklist.push(PathBuf::from("/etc/shadow"));
        blacklist.push(PathBuf::from("/etc/passwd"));
        
        Ok(())
    }
    
    /// 检查文件权限
    pub async fn check_permission(
        &self,
        path: &Path,
        operation: FileOperation,
        user_id: &str,
    ) -> Result<PermissionDecision> {
        let canonical_path = if path.exists() {
            path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
        } else {
            path.to_path_buf()
        };
        
        let blacklist = self.blacklist.read().await;
        for blacklisted_path in blacklist.iter() {
            if canonical_path.starts_with(blacklisted_path) {
                return Ok(PermissionDecision::Deny(format!(
                    "Path '{}' is blacklisted",
                    path.display()
                )));
            }
        }
        
        let protected = self.protected_patterns.read().await;
        for pattern in protected.iter() {
            if Self::matches_protected_pattern(&canonical_path, pattern) {
                match operation {
                    FileOperation::Read => {
                        return Ok(PermissionDecision::RequireApproval {
                            level: ApprovalLevel::Single,
                            reason: format!(
                                "Path '{}' matches protected pattern '{}'",
                                path.display(),
                                pattern
                            ),
                        });
                    },
                    FileOperation::Write | FileOperation::Delete => {
                        return Ok(PermissionDecision::RequireApproval {
                            level: ApprovalLevel::Double,
                            reason: format!(
                                "Write/Delete access to protected path '{}'",
                                path.display()
                            ),
                        });
                    },
                    _ => {},
                }
            }
        }
        
        let whitelist = self.whitelist.read().await;
        for entry in whitelist.iter() {
            if Self::path_matches_whitelist(&canonical_path, entry) {
                if entry.allowed_operations.contains(&operation) {
                    return Ok(PermissionDecision::Allow);
                } else {
                    return Ok(PermissionDecision::Deny(format!(
                        "Operation '{}' not allowed for path '{}'",
                        operation,
                        path.display()
                    )));
                }
            }
        }
        
        match operation {
            FileOperation::Read | FileOperation::List => {
                Ok(PermissionDecision::Allow)
            },
            FileOperation::Write | FileOperation::Delete => {
                Ok(PermissionDecision::RequireApproval {
                    level: ApprovalLevel::Single,
                    reason: format!(
                        "Path '{}' not in whitelist, requires approval for '{}'",
                        path.display(),
                        operation
                    ),
                })
            },
            FileOperation::Execute => {
                Ok(PermissionDecision::RequireApproval {
                    level: ApprovalLevel::Double,
                    reason: format!(
                        "Execute permission required for '{}'",
                        path.display()
                    ),
                })
            },
        }
    }
    
    /// 添加白名单条目
    pub async fn add_whitelist_entry(&self, entry: DirectoryWhitelistEntry) -> Result<()> {
        let mut whitelist = self.whitelist.write().await;
        whitelist.push(entry);
        Ok(())
    }
    
    /// 移除白名单条目
    pub async fn remove_whitelist_entry(&self, path: &Path) -> Result<()> {
        let mut whitelist = self.whitelist.write().await;
        whitelist.retain(|entry| entry.path != path);
        Ok(())
    }
    
    /// 添加黑名单路径
    pub async fn add_blacklist_entry(&self, path: PathBuf) -> Result<()> {
        let mut blacklist = self.blacklist.write().await;
        if !blacklist.contains(&path) {
            blacklist.push(path);
        }
        Ok(())
    }
    
    /// 添加受保护模式
    pub async fn add_protected_pattern(&self, pattern: String) -> Result<()> {
        let mut protected = self.protected_patterns.write().await;
        if !protected.contains(&pattern) {
            protected.push(pattern);
        }
        Ok(())
    }
    
    /// 检查路径是否匹配白名单
    fn path_matches_whitelist(path: &Path, entry: &DirectoryWhitelistEntry) -> bool {
        if entry.recursive {
            path.starts_with(&entry.path)
        } else {
            path == entry.path
        }
    }
    
    /// 检查路径是否匹配受保护模式
    fn matches_protected_pattern(path: &Path, pattern: &str) -> bool {
        let path_str = path.to_string_lossy();
        
        if pattern.starts_with("**/") {
            let suffix = &pattern[3..];
            return path_str.ends_with(suffix) || 
                   path_str.contains(&format!("/{}", suffix));
        }
        
        if pattern.ends_with("/**") {
            let prefix = &pattern[..pattern.len() - 3];
            return path_str.starts_with(prefix);
        }
        
        if pattern.contains("**") {
            let parts: Vec<&str> = pattern.split("**").collect();
            if parts.len() == 2 {
                return path_str.starts_with(parts[0]) && path_str.ends_with(parts[1]);
            }
        }
        
        glob_match::glob_match(pattern, &path_str)
    }
    
    /// 验证路径安全性
    pub async fn validate_path(&self, path: &Path) -> Result<bool> {
        let canonical = if path.exists() {
            path.canonicalize()?
        } else {
            path.to_path_buf()
        };
        
        let blacklist = self.blacklist.read().await;
        for blacklisted in blacklist.iter() {
            if canonical.starts_with(blacklisted) {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}

impl Default for FilePermissionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_file_permission_manager() {
        let mut manager = FilePermissionManager::new();
        manager.load_default_policies().await.unwrap();
        
        let decision = manager.check_permission(
            Path::new("."),
            FileOperation::Read,
            "user1"
        ).await.unwrap();
        
        match decision {
            PermissionDecision::Allow => (),
            _ => panic!("Expected Allow for current directory"),
        }
    }
    
    #[tokio::test]
    async fn test_blacklist() {
        let mut manager = FilePermissionManager::new();
        manager.load_default_policies().await.unwrap();
        
        let decision = manager.check_permission(
            Path::new("/etc/shadow"),
            FileOperation::Read,
            "user1"
        ).await.unwrap();
        
        match decision {
            PermissionDecision::Deny(_) => (),
            _ => panic!("Expected Deny for blacklisted path"),
        }
    }
}
