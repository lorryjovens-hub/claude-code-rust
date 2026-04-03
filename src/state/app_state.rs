//! 应用状态管理器
//! 
//! 这个模块实现了线程安全的状态管理器

use std::sync::Arc;
use tokio::sync::RwLock;
use super::State;

/// 应用状态（线程安全的状态管理器）
pub type AppState = Arc<RwLock<State>>;

/// 创建新的应用状态
pub fn new_app_state() -> AppState {
    Arc::new(RwLock::new(State::new()))
}

/// 应用状态扩展方法
pub trait AppStateExt {
    /// 获取会话 ID
    async fn get_session_id(&self) -> String;
    
    /// 获取原始工作目录
    async fn get_original_cwd(&self) -> std::path::PathBuf;
    
    /// 获取当前工作目录
    async fn get_cwd(&self) -> std::path::PathBuf;
    
    /// 设置当前工作目录
    async fn set_cwd(&self, cwd: std::path::PathBuf);
    
    /// 获取总成本
    async fn get_total_cost(&self) -> f64;
    
    /// 添加成本
    async fn add_cost(&self, cost: f64, model: String, usage: super::ModelUsage);
    
    /// 获取总持续时间
    async fn get_total_duration(&self) -> i64;
    
    /// 是否交互式
    async fn is_interactive(&self) -> bool;
    
    /// 设置交互式
    async fn set_interactive(&self, value: bool);
    
    /// 是否绕过权限模式
    async fn is_bypass_permissions_mode(&self) -> bool;
    
    /// 设置绕过权限模式
    async fn set_bypass_permissions_mode(&self, value: bool);
}

impl AppStateExt for AppState {
    async fn get_session_id(&self) -> String {
        self.read().await.session_id.clone()
    }
    
    async fn get_original_cwd(&self) -> std::path::PathBuf {
        self.read().await.original_cwd.clone()
    }
    
    async fn get_cwd(&self) -> std::path::PathBuf {
        self.read().await.cwd.clone()
    }
    
    async fn set_cwd(&self, cwd: std::path::PathBuf) {
        self.write().await.cwd = cwd;
    }
    
    async fn get_total_cost(&self) -> f64 {
        self.read().await.total_cost_usd
    }
    
    async fn add_cost(&self, cost: f64, model: String, usage: super::ModelUsage) {
        self.write().await.add_cost(cost, model, usage);
    }
    
    async fn get_total_duration(&self) -> i64 {
        self.read().await.get_total_duration()
    }
    
    async fn is_interactive(&self) -> bool {
        self.read().await.is_interactive
    }
    
    async fn set_interactive(&self, value: bool) {
        self.write().await.is_interactive = value;
    }
    
    async fn is_bypass_permissions_mode(&self) -> bool {
        self.read().await.session_bypass_permissions_mode
    }
    
    async fn set_bypass_permissions_mode(&self, value: bool) {
        self.write().await.session_bypass_permissions_mode = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_app_state_creation() {
        let state = new_app_state();
        let session_id = state.get_session_id().await;
        
        assert!(!session_id.is_empty());
    }
    
    #[tokio::test]
    async fn test_app_state_cost() {
        let state = new_app_state();
        
        let usage = super::super::ModelUsage {
            input_tokens: 100,
            output_tokens: 50,
            cache_read_input_tokens: None,
            cache_creation_input_tokens: None,
            web_search_requests: None,
        };
        
        state.add_cost(0.01, "claude-3-opus".to_string(), usage).await;
        
        let cost = state.get_total_cost().await;
        assert_eq!(cost, 0.01);
    }
    
    #[tokio::test]
    async fn test_app_state_interactive() {
        let state = new_app_state();
        
        assert!(state.is_interactive().await);
        
        state.set_interactive(false).await;
        assert!(!state.is_interactive().await);
    }
}
