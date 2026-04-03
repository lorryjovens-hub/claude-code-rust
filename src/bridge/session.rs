//! 会话管理模块

use super::types::*;
use crate::error::Result;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, RwLock};

/// 会话句柄
pub struct SessionHandle {
    /// 会话ID
    pub session_id: String,
    /// 完成状态
    pub done: tokio::sync::oneshot::Receiver<SessionDoneStatus>,
    /// 杀死信号
    kill_sender: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    /// 强制杀死信号
    force_kill_sender: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    /// 活动列表（环形缓冲区）
    activities: Arc<Mutex<VecDeque<SessionActivity>>>,
    /// 当前活动
    current_activity: Arc<Mutex<Option<SessionActivity>>>,
    /// 访问令牌
    access_token: Arc<RwLock<String>>,
    /// 最后的标准错误输出
    last_stderr: Arc<Mutex<VecDeque<String>>>,
    /// 标准输入写入器
    stdin_writer: Arc<Mutex<Option<tokio::process::ChildStdin>>>,
}

impl SessionHandle {
    /// 创建新的会话句柄
    pub fn new(session_id: String, access_token: String) -> Self {
        let (_done_tx, done_rx) = tokio::sync::oneshot::channel();
        let (kill_tx, _kill_rx) = tokio::sync::oneshot::channel();
        let (force_kill_tx, _force_kill_rx) = tokio::sync::oneshot::channel();
        
        Self {
            session_id,
            done: done_rx,
            kill_sender: Arc::new(Mutex::new(Some(kill_tx))),
            force_kill_sender: Arc::new(Mutex::new(Some(force_kill_tx))),
            activities: Arc::new(Mutex::new(VecDeque::with_capacity(10))),
            current_activity: Arc::new(Mutex::new(None)),
            access_token: Arc::new(RwLock::new(access_token)),
            last_stderr: Arc::new(Mutex::new(VecDeque::with_capacity(10))),
            stdin_writer: Arc::new(Mutex::new(None)),
        }
    }
    
    /// 杀死会话
    pub async fn kill(&self) {
        let mut kill_sender = self.kill_sender.lock().await;
        if let Some(sender) = kill_sender.take() {
            let _ = sender.send(());
        }
    }
    
    /// 强制杀死会话
    pub async fn force_kill(&self) {
        let mut force_kill_sender = self.force_kill_sender.lock().await;
        if let Some(sender) = force_kill_sender.take() {
            let _ = sender.send(());
        }
    }
    
    /// 获取活动列表
    pub async fn get_activities(&self) -> Vec<SessionActivity> {
        let activities = self.activities.lock().await;
        activities.iter().cloned().collect()
    }
    
    /// 添加活动
    pub async fn add_activity(&self, activity: SessionActivity) {
        let mut activities = self.activities.lock().await;
        if activities.len() >= 10 {
            activities.pop_front();
        }
        activities.push_back(activity.clone());
        
        let mut current = self.current_activity.lock().await;
        *current = Some(activity);
    }
    
    /// 获取当前活动
    pub async fn get_current_activity(&self) -> Option<SessionActivity> {
        let current = self.current_activity.lock().await;
        current.clone()
    }
    
    /// 更新访问令牌
    pub async fn update_access_token(&self, token: String) {
        let mut access_token = self.access_token.write().await;
        *access_token = token;
    }
    
    /// 获取访问令牌
    pub async fn get_access_token(&self) -> String {
        let access_token = self.access_token.read().await;
        access_token.clone()
    }
    
    /// 写入标准输入
    pub async fn write_stdin(&self, data: &str) -> Result<()> {
        let mut stdin = self.stdin_writer.lock().await;
        if let Some(ref mut stdin) = *stdin {
            use tokio::io::AsyncWriteExt;
            stdin.write_all(data.as_bytes()).await?;
        }
        Ok(())
    }
    
    /// 添加标准错误输出
    pub async fn add_stderr(&self, line: String) {
        let mut stderr = self.last_stderr.lock().await;
        if stderr.len() >= 10 {
            stderr.pop_front();
        }
        stderr.push_back(line);
    }
    
    /// 获取最后的错误输出
    pub async fn get_last_stderr(&self) -> Vec<String> {
        let stderr = self.last_stderr.lock().await;
        stderr.iter().cloned().collect()
    }
}

/// 会话生成选项
#[derive(Clone)]
pub struct SessionSpawnOpts {
    /// 会话ID
    pub session_id: String,
    /// SDK URL
    pub sdk_url: String,
    /// 访问令牌
    pub access_token: String,
    /// 是否使用 CCR v2
    pub use_ccr_v2: Option<bool>,
    /// 工作器纪元
    pub worker_epoch: Option<u64>,
    /// 首次用户消息回调
    pub on_first_user_message: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

/// 会话管理器
pub struct SessionManager {
    /// 会话模式
    spawn_mode: SpawnMode,
    /// 最大会话数
    max_sessions: usize,
    /// 活动会话
    active_sessions: Arc<RwLock<HashMap<String, Arc<SessionHandle>>>>,
    /// 工作目录
    working_dir: PathBuf,
}

impl SessionManager {
    /// 创建新的会话管理器
    pub fn new(spawn_mode: SpawnMode, max_sessions: usize, working_dir: PathBuf) -> Self {
        Self {
            spawn_mode,
            max_sessions,
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            working_dir,
        }
    }
    
    /// 生成新会话
    pub async fn spawn(&self, opts: SessionSpawnOpts) -> Result<Arc<SessionHandle>> {
        let handle = Arc::new(SessionHandle::new(
            opts.session_id.clone(),
            opts.access_token.clone(),
        ));
        
        let mut active_sessions = self.active_sessions.write().await;
        
        if active_sessions.len() >= self.max_sessions {
            return Err(crate::error::ClaudeError::Bridge(
                "Maximum sessions reached".to_string()
            ));
        }
        
        let session_dir = self.prepare_session_directory(&opts.session_id).await?;
        
        let mut child = self.spawn_child_process(&opts, &session_dir).await?;
        
        let stdin = child.stdin.take();
        let mut handle_clone = Arc::clone(&handle);
        
        let session_id = opts.session_id.clone();
        tokio::spawn(async move {
            let status = child.wait().await;
            let done_status = match status {
                Ok(s) if s.success() => SessionDoneStatus::Completed,
                Ok(_) => SessionDoneStatus::Failed,
                Err(_) => SessionDoneStatus::Interrupted,
            };
            
            tracing::info!("Session {} finished with status: {:?}", session_id, done_status);
        });
        
        active_sessions.insert(opts.session_id.clone(), Arc::clone(&handle));
        
        Ok(handle)
    }
    
    /// 准备会话目录
    async fn prepare_session_directory(&self, session_id: &str) -> Result<PathBuf> {
        let dir = match self.spawn_mode {
            SpawnMode::SingleSession => self.working_dir.clone(),
            SpawnMode::Worktree => {
                let worktree_dir = self.working_dir.join(".worktrees").join(session_id);
                tokio::fs::create_dir_all(&worktree_dir).await?;
                worktree_dir
            }
            SpawnMode::SameDir => self.working_dir.clone(),
        };
        
        Ok(dir)
    }
    
    /// 生成子进程
    async fn spawn_child_process(
        &self,
        opts: &SessionSpawnOpts,
        working_dir: &PathBuf,
    ) -> Result<Child> {
        let mut cmd = Command::new("claude");
        
        cmd.arg("--bridge-mode")
            .arg("--session-id")
            .arg(&opts.session_id)
            .arg("--sdk-url")
            .arg(&opts.sdk_url)
            .current_dir(working_dir)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        
        if let Some(true) = opts.use_ccr_v2 {
            cmd.env("CCR_V2_ENABLED", "1");
            if let Some(epoch) = opts.worker_epoch {
                cmd.env("WORKER_EPOCH", epoch.to_string());
            }
        }
        
        let child = cmd.spawn()?;
        
        Ok(child)
    }
    
    /// 获取活动会话数
    pub async fn active_count(&self) -> usize {
        let sessions = self.active_sessions.read().await;
        sessions.len()
    }
    
    /// 获取会话
    pub async fn get_session(&self, session_id: &str) -> Option<Arc<SessionHandle>> {
        let sessions = self.active_sessions.read().await;
        sessions.get(session_id).cloned()
    }
    
    /// 移除会话
    pub async fn remove_session(&self, session_id: &str) {
        let mut sessions = self.active_sessions.write().await;
        sessions.remove(session_id);
    }
    
    /// 获取所有会话ID
    pub async fn get_all_session_ids(&self) -> Vec<String> {
        let sessions = self.active_sessions.read().await;
        sessions.keys().cloned().collect()
    }
    
    /// 获取会话模式
    pub fn spawn_mode(&self) -> SpawnMode {
        self.spawn_mode
    }
    
    /// 获取最大会话数
    pub fn max_sessions(&self) -> usize {
        self.max_sessions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_session_handle() {
        let handle = SessionHandle::new(
            "test-session".to_string(),
            "test-token".to_string(),
        );
        
        assert_eq!(handle.session_id, "test-session");
        
        let token = handle.get_access_token().await;
        assert_eq!(token, "test-token");
    }
    
    #[tokio::test]
    async fn test_session_manager() {
        let manager = SessionManager::new(
            SpawnMode::SingleSession,
            1,
            PathBuf::from("."),
        );
        
        assert_eq!(manager.spawn_mode(), SpawnMode::SingleSession);
        assert_eq!(manager.max_sessions(), 1);
        assert_eq!(manager.active_count().await, 0);
    }
}
