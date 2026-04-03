//! 审计日志记录器

use super::{AuditConfig, AuditLevel, AuditEvent};
use crate::error::Result;
use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 审计日志记录器
/// 
/// 负责记录所有审计事件
#[derive(Debug)]
pub struct AuditLogger {
    /// 配置
    config: AuditConfig,
    /// 事件队列
    event_queue: Arc<RwLock<VecDeque<AuditEvent>>>,
    /// 日志文件路径
    log_file: Arc<RwLock<Option<PathBuf>>>,
    /// 是否已初始化
    initialized: bool,
}

impl AuditLogger {
    /// 创建新的审计日志记录器
    pub fn new() -> Result<Self> {
        let config = AuditConfig::default();
        
        Ok(Self {
            config,
            event_queue: Arc::new(RwLock::new(VecDeque::new())),
            log_file: Arc::new(RwLock::new(None)),
            initialized: false,
        })
    }
    
    /// 创建带配置的审计日志记录器
    pub fn with_config(config: AuditConfig) -> Result<Self> {
        Ok(Self {
            config,
            event_queue: Arc::new(RwLock::new(VecDeque::new())),
            log_file: Arc::new(RwLock::new(None)),
            initialized: false,
        })
    }
    
    /// 初始化审计日志系统
    pub async fn initialize(&mut self) -> Result<()> {
        if !self.config.enabled {
            tracing::info!("Audit logging is disabled");
            return Ok(());
        }
        
        tracing::info!("Initializing audit logger");
        
        std::fs::create_dir_all(&self.config.log_dir)?;
        
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let log_file_path = self.config.log_dir.join(format!("audit_{}.log", timestamp));
        
        let _file = File::create(&log_file_path)?;
        
        let mut log_file = self.log_file.write().await;
        *log_file = Some(log_file_path);
        
        self.initialized = true;
        
        tracing::info!("Audit logger initialized");
        Ok(())
    }
    
    /// 记录事件
    pub async fn log(&self, event: AuditEvent) {
        if !self.config.enabled {
            return;
        }
        
        if event.level < self.config.min_level {
            return;
        }
        
        let mut queue = self.event_queue.write().await;
        queue.push_back(event);
        
        if queue.len() > 1000 {
            queue.pop_front();
        }
    }
    
    /// 记录工具调用
    pub async fn log_tool_call(
        &self,
        tool_name: &str,
        user_id: &str,
        input: &serde_json::Value,
        result: &serde_json::Value,
        duration_ms: u64,
    ) {
        if !self.config.log_tool_calls {
            return;
        }
        
        let event = AuditEvent::tool_call(
            tool_name,
            user_id,
            input,
            result,
            duration_ms,
        );
        
        self.log(event).await;
    }
    
    /// 记录权限决策
    pub async fn log_permission_decision(
        &self,
        resource_type: &str,
        resource_id: &str,
        operation: &str,
        user_id: &str,
        decision: &str,
        reason: Option<&str>,
    ) {
        if !self.config.log_permission_decisions {
            return;
        }
        
        let event = AuditEvent::permission_decision(
            resource_type,
            resource_id,
            operation,
            user_id,
            decision,
            reason,
        );
        
        self.log(event).await;
    }
    
    /// 记录文件操作
    pub async fn log_file_operation(
        &self,
        operation: &str,
        path: &std::path::Path,
        user_id: &str,
        success: bool,
    ) {
        if !self.config.log_file_operations {
            return;
        }
        
        let event = AuditEvent::file_operation(
            operation,
            path,
            user_id,
            success,
        );
        
        self.log(event).await;
    }
    
    /// 记录网络请求
    pub async fn log_network_request(
        &self,
        url: &str,
        method: &str,
        user_id: &str,
        status_code: Option<u16>,
        response_size: Option<u64>,
    ) {
        if !self.config.log_network_requests {
            return;
        }
        
        let event = AuditEvent::network_request(
            url,
            method,
            user_id,
            status_code,
            response_size,
        );
        
        self.log(event).await;
    }
    
    /// 记录危险命令检测
    pub async fn log_dangerous_command(
        &self,
        command: &str,
        user_id: &str,
        danger_level: &str,
    ) {
        let event = AuditEvent::dangerous_command(command, user_id, danger_level);
        self.log(event).await;
    }
    
    /// 记录沙箱执行
    pub async fn log_sandbox_execution(
        &self,
        command: &str,
        sandboxed: bool,
        success: bool,
        reason: Option<&str>,
    ) {
        let event = AuditEvent::sandbox_execution(command, sandboxed, success, reason);
        self.log(event).await;
    }
    
    /// 记录认证事件
    pub async fn log_authentication(
        &self,
        user_id: &str,
        success: bool,
        method: &str,
        source_ip: Option<&str>,
    ) {
        let event = AuditEvent::authentication(user_id, success, method, source_ip);
        self.log(event).await;
    }
    
    /// 刷新日志到磁盘
    pub async fn flush(&self) -> Result<()> {
        if !self.config.enabled || !self.initialized {
            return Ok(());
        }
        
        let queue = self.event_queue.read().await;
        
        if queue.is_empty() {
            return Ok(());
        }
        
        let log_file = self.log_file.read().await;
        
        if let Some(ref path) = *log_file {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;
            
            for event in queue.iter() {
                let json = serde_json::to_string(event)?;
                writeln!(file, "{}", json)?;
            }
            
            file.flush()?;
        }
        
        Ok(())
    }
    
    /// 获取待处理的事件数量
    pub async fn pending_events(&self) -> usize {
        let queue = self.event_queue.read().await;
        queue.len()
    }
    
    /// 获取配置
    pub fn config(&self) -> &AuditConfig {
        &self.config
    }
    
    /// 导出事件（用于调试）
    pub async fn export_events(&self) -> Vec<AuditEvent> {
        let queue = self.event_queue.read().await;
        queue.iter().cloned().collect()
    }
    
    /// 清空事件
    pub async fn clear(&self) {
        let mut queue = self.event_queue.write().await;
        queue.clear();
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new().expect("Failed to create audit logger")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_audit_logger() {
        let logger = AuditLogger::new().unwrap();
        
        logger.log_tool_call(
            "Read",
            "user1",
            &serde_json::json!({"path": "/test"}),
            &serde_json::json!({"content": "test"}),
            100,
        ).await;
        
        assert_eq!(logger.pending_events().await, 1);
    }
    
    #[tokio::test]
    async fn test_permission_decision_logging() {
        let logger = AuditLogger::new().unwrap();
        
        logger.log_permission_decision(
            "file",
            "/etc/passwd",
            "read",
            "user1",
            "deny",
            Some("Path is blacklisted"),
        ).await;
        
        assert_eq!(logger.pending_events().await, 1);
    }
}
