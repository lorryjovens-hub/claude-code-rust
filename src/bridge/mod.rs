//! Bridge 远程控制系统
//! 
//! 提供完整的远程控制功能，包括：
//! - 会话管理（single-session, worktree, same-dir）
//! - JWT 认证和令牌刷新
//! - 远程会话生命周期管理
//! - 工作密钥和权限验证
//! - 点对点连接

pub mod types;
pub mod session;
pub mod auth;
pub mod manager;
pub mod worker;
pub mod connection;

pub use types::*;
pub use session::{SessionManager, SessionHandle};
pub use auth::{AuthManager, JwtToken, TokenRefreshScheduler};
pub use manager::BridgeManager;
pub use worker::{WorkerManager, WorkKey};
pub use connection::{ConnectionManager, P2PConnection};

use crate::error::Result;

/// 初始化 Bridge 系统
pub async fn initialize() -> Result<()> {
    tracing::info!("Initializing Bridge remote control system");
    Ok(())
}
