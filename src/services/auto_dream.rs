//! 自动记忆整合模块
//! 
//! 这个模块实现了后台记忆整合功能，对应 TypeScript 的 services/autoDream/autoDream.ts

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 自动记忆配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoDreamConfig {
    /// 最小小时数（距离上次整合）
    pub min_hours: u64,
    
    /// 最小会话数
    pub min_sessions: usize,
}

impl Default for AutoDreamConfig {
    fn default() -> Self {
        Self {
            min_hours: 24,
            min_sessions: 5,
        }
    }
}

/// 整合锁
pub struct ConsolidationLock {
    /// 锁文件路径
    lock_path: PathBuf,
    
    /// 是否已获取
    acquired: bool,
}

impl ConsolidationLock {
    /// 创建新的整合锁
    pub fn new(lock_path: PathBuf) -> Self {
        Self {
            lock_path,
            acquired: false,
        }
    }
    
    /// 尝试获取锁
    pub async fn try_acquire(&mut self) -> Result<Option<i64>> {
        // 检查锁文件是否存在
        if self.lock_path.exists() {
            // 读取锁文件的修改时间
            let metadata = tokio::fs::metadata(&self.lock_path).await?;
            let modified = metadata.modified()
                .map_err(|e| crate::error::ClaudeError::Other(e.to_string()))?;
            let mtime = modified.duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| crate::error::ClaudeError::Other(e.to_string()))?
                .as_millis() as i64;
            
            // 检查锁是否过期（例如，超过 1 小时）
            let now = chrono::Utc::now().timestamp_millis();
            if now - mtime > 3600 * 1000 {
                // 锁已过期，可以获取
                self.acquired = true;
                return Ok(Some(mtime));
            }
            
            // 锁未过期，无法获取
            return Ok(None);
        }
        
        // 创建锁文件
        if let Some(parent) = self.lock_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::File::create(&self.lock_path).await?;
        
        self.acquired = true;
        Ok(Some(0))
    }
    
    /// 回滚锁
    pub async fn rollback(&self, _prior_mtime: i64) -> Result<()> {
        if !self.acquired {
            return Ok(());
        }
        
        // 简化实现：不设置修改时间，只是确保文件存在
        if !self.lock_path.exists() {
            if let Some(parent) = self.lock_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::File::create(&self.lock_path).await?;
        }
        
        Ok(())
    }
    
    /// 释放锁
    pub async fn release(&self) -> Result<()> {
        if self.acquired && self.lock_path.exists() {
            tokio::fs::remove_file(&self.lock_path).await?;
        }
        Ok(())
    }
}

/// 自动记忆整合器
pub struct AutoDream {
    /// 配置
    config: AutoDreamConfig,
    
    /// 记忆目录
    memory_dir: PathBuf,
    
    /// 会话目录
    session_dir: PathBuf,
    
    /// 上次扫描时间
    last_session_scan_at: Arc<RwLock<i64>>,
    
    /// 是否已初始化
    initialized: Arc<RwLock<bool>>,
}

impl AutoDream {
    /// 创建新的自动记忆整合器
    pub fn new(memory_dir: PathBuf, session_dir: PathBuf) -> Self {
        Self {
            config: AutoDreamConfig::default(),
            memory_dir,
            session_dir,
            last_session_scan_at: Arc::new(RwLock::new(0)),
            initialized: Arc::new(RwLock::new(false)),
        }
    }
    
    /// 使用自定义配置创建
    pub fn with_config(memory_dir: PathBuf, session_dir: PathBuf, config: AutoDreamConfig) -> Self {
        Self {
            config,
            memory_dir,
            session_dir,
            last_session_scan_at: Arc::new(RwLock::new(0)),
            initialized: Arc::new(RwLock::new(false)),
        }
    }
    
    /// 初始化
    pub async fn init(&self) -> Result<()> {
        // 创建必要的目录
        tokio::fs::create_dir_all(&self.memory_dir).await?;
        tokio::fs::create_dir_all(&self.session_dir).await?;
        
        *self.initialized.write().await = true;
        
        tracing::debug!("AutoDream initialized");
        Ok(())
    }
    
    /// 检查门是否开启
    pub fn is_gate_open(&self) -> bool {
        // 检查各种状态
        // 1. 不在 KAIROS 模式
        // 2. 不在远程模式
        // 3. 自动记忆已启用
        // 4. AutoDream 已启用
        
        // 简化实现：检查环境变量
        std::env::var("CLAUDE_CODE_AUTO_DREAM")
            .map(|v| v == "1" || v == "true")
            .unwrap_or(false)
    }
    
    /// 读取上次整合时间
    pub async fn read_last_consolidated_at(&self) -> Result<i64> {
        let lock_path = self.memory_dir.join(".consolidation_lock");
        
        if lock_path.exists() {
            let metadata = tokio::fs::metadata(&lock_path).await?;
            let modified = metadata.modified()
                .map_err(|e| crate::error::ClaudeError::Other(e.to_string()))?;
            Ok(modified.duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| crate::error::ClaudeError::Other(e.to_string()))?
                .as_millis() as i64)
        } else {
            Ok(0)
        }
    }
    
    /// 列出自上次整合以来的会话
    pub async fn list_sessions_touched_since(&self, since: i64) -> Result<Vec<String>> {
        let mut sessions = Vec::new();
        
        if !self.session_dir.exists() {
            return Ok(sessions);
        }
        
        let mut entries = tokio::fs::read_dir(&self.session_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                let metadata = entry.metadata().await?;
                let modified = metadata.modified()
                    .map_err(|e| crate::error::ClaudeError::Other(e.to_string()))?;
                let mtime = modified.duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| crate::error::ClaudeError::Other(e.to_string()))?
                    .as_millis() as i64;
                
                if mtime > since {
                    if let Some(name) = path.file_name() {
                        if let Some(name_str) = name.to_str() {
                            sessions.push(name_str.to_string());
                        }
                    }
                }
            }
        }
        
        Ok(sessions)
    }
    
    /// 执行自动记忆整合
    pub async fn execute(&self) -> Result<()> {
        if !*self.initialized.read().await {
            return Ok(());
        }
        
        if !self.is_gate_open() {
            return Ok(());
        }
        
        // 1. 时间检查
        let last_at = self.read_last_consolidated_at().await?;
        let now = chrono::Utc::now().timestamp_millis();
        let hours_since = (now - last_at) as f64 / 3_600_000.0;
        
        if hours_since < self.config.min_hours as f64 {
            tracing::debug!("AutoDream: time gate not passed");
            return Ok(());
        }
        
        // 2. 扫描节流
        let since_scan_ms = now - *self.last_session_scan_at.read().await;
        if since_scan_ms < 10 * 60 * 1000 { // 10 分钟
            tracing::debug!("AutoDream: scan throttle active");
            return Ok(());
        }
        
        *self.last_session_scan_at.write().await = now;
        
        // 3. 会话计数
        let sessions = self.list_sessions_touched_since(last_at).await?;
        if sessions.len() < self.config.min_sessions {
            tracing::debug!(
                "AutoDream: not enough sessions ({} < {})",
                sessions.len(),
                self.config.min_sessions
            );
            return Ok(());
        }
        
        // 4. 获取锁
        let lock_path = self.memory_dir.join(".consolidation_lock");
        let mut lock = ConsolidationLock::new(lock_path);
        
        let prior_mtime = match lock.try_acquire().await? {
            Some(mtime) => mtime,
            None => {
                tracing::debug!("AutoDream: lock acquisition failed");
                return Ok(());
            }
        };
        
        tracing::info!(
            "AutoDream: firing - {:.1}h since last, {} sessions to review",
            hours_since,
            sessions.len()
        );
        
        // 5. 执行记忆整合
        match self.run_consolidation(&sessions).await {
            Ok(_) => {
                tracing::info!("AutoDream: completed successfully");
                lock.release().await?;
            }
            Err(e) => {
                tracing::error!("AutoDream: failed - {}", e);
                lock.rollback(prior_mtime).await?;
            }
        }
        
        Ok(())
    }
    
    /// 运行整合过程
    async fn run_consolidation(&self, sessions: &[String]) -> Result<()> {
        // TODO: 实现实际的记忆整合逻辑
        // 1. 提取关键信息
        // 2. 合并记忆
        // 3. 更新记忆文件
        
        tracing::info!("Consolidating {} sessions", sessions.len());
        
        // 模拟整合过程
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        Ok(())
    }
    
    /// 重置状态
    pub async fn reset(&self) {
        *self.last_session_scan_at.write().await = 0;
        *self.initialized.write().await = false;
    }
}

/// 构建整合提示
pub fn build_consolidation_prompt(memory_root: &str, transcript_dir: &str, extra: &str) -> String {
    format!(
        r#"# Memory Consolidation

You are performing a memory consolidation task. Your goal is to:

1. Review recent sessions and extract key information
2. Identify patterns, decisions, and important context
3. Update the memory files with consolidated knowledge

## Memory Root
{}

## Transcript Directory
{}

## Additional Context
{}

## Instructions

1. Read the memory files to understand existing knowledge
2. Review the session transcripts listed in the context
3. Extract new information that should be remembered
4. Update the memory files with consolidated knowledge
5. Remove redundant or outdated information
6. Ensure the memory remains concise and useful

Focus on:
- Important decisions and their rationale
- Key patterns and conventions
- Useful context for future sessions
- Critical errors and their solutions

Do not include:
- Temporary or transient information
- Session-specific details without general value
- Redundant information already in memory"#,
        memory_root, transcript_dir, extra
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_auto_dream_config_default() {
        let config = AutoDreamConfig::default();
        assert_eq!(config.min_hours, 24);
        assert_eq!(config.min_sessions, 5);
    }
    
    #[tokio::test]
    async fn test_auto_dream_creation() {
        let temp_dir = TempDir::new().unwrap();
        let memory_dir = temp_dir.path().join("memory");
        let session_dir = temp_dir.path().join("sessions");
        
        let auto_dream = AutoDream::new(memory_dir, session_dir);
        auto_dream.init().await.unwrap();
        
        assert!(*auto_dream.initialized.read().await);
    }
    
    #[tokio::test]
    async fn test_read_last_consolidated_at() {
        let temp_dir = TempDir::new().unwrap();
        let memory_dir = temp_dir.path().join("memory");
        let session_dir = temp_dir.path().join("sessions");
        
        let auto_dream = AutoDream::new(memory_dir, session_dir);
        auto_dream.init().await.unwrap();
        
        let last_at = auto_dream.read_last_consolidated_at().await.unwrap();
        assert_eq!(last_at, 0);
    }
    
    #[tokio::test]
    async fn test_list_sessions() {
        let temp_dir = TempDir::new().unwrap();
        let memory_dir = temp_dir.path().join("memory");
        let session_dir = temp_dir.path().join("sessions");
        
        tokio::fs::create_dir_all(&session_dir).await.unwrap();
        
        // 创建一些测试会话文件
        tokio::fs::write(session_dir.join("session1.json"), "{}").await.unwrap();
        tokio::fs::write(session_dir.join("session2.json"), "{}").await.unwrap();
        
        let auto_dream = AutoDream::new(memory_dir.clone(), session_dir.clone());
        auto_dream.init().await.unwrap();
        
        let sessions = auto_dream.list_sessions_touched_since(0).await.unwrap();
        assert_eq!(sessions.len(), 2);
    }
    
    #[test]
    fn test_build_consolidation_prompt() {
        let prompt = build_consolidation_prompt(
            "/path/to/memory",
            "/path/to/sessions",
            "Extra context"
        );
        
        assert!(prompt.contains("/path/to/memory"));
        assert!(prompt.contains("/path/to/sessions"));
        assert!(prompt.contains("Extra context"));
    }
}
