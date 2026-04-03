//! 响应式信号机制
//! 
//! 这个模块实现了响应式信号系统，用于状态变更通知

use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// 信号订阅者
pub type SignalSubscriber<T> = broadcast::Receiver<T>;

/// 信号发布者
pub type SignalPublisher<T> = broadcast::Sender<T>;

/// 信号
pub struct Signal<T> {
    /// 发布者
    sender: broadcast::Sender<T>,
    
    /// 订阅者数量
    subscriber_count: Arc<RwLock<usize>>,
}

impl<T: Clone + Send + 'static> Signal<T> {
    /// 创建新信号
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1024);
        Self {
            sender,
            subscriber_count: Arc::new(RwLock::new(0)),
        }
    }
    
    /// 创建带容量的信号
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender,
            subscriber_count: Arc::new(RwLock::new(0)),
        }
    }
    
    /// 订阅信号
    pub fn subscribe(&self) -> SignalSubscriber<T> {
        let mut count = self.subscriber_count.blocking_write();
        *count += 1;
        self.sender.subscribe()
    }
    
    /// 发送信号
    pub fn send(&self, value: T) -> Result<usize, broadcast::error::SendError<T>> {
        self.sender.send(value)
    }
    
    /// 获取订阅者数量
    pub async fn subscriber_count(&self) -> usize {
        *self.subscriber_count.read().await
    }
    
    /// 清除所有订阅者
    pub fn clear(&self) {
        let mut count = self.subscriber_count.blocking_write();
        *count = 0;
    }
}

impl<T: Clone + Send + 'static> Default for Signal<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> std::fmt::Debug for Signal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signal")
            .field("subscriber_count", &self.subscriber_count.blocking_read())
            .finish_non_exhaustive()
    }
}

/// 会话切换信号
pub type SessionSwitchedSignal = Signal<String>;

/// 状态变更信号
pub type StateChangedSignal = Signal<String>;

/// 全局信号管理器
pub struct SignalManager {
    /// 会话切换信号
    pub session_switched: SessionSwitchedSignal,
    
    /// 状态变更信号
    pub state_changed: StateChangedSignal,
}

impl SignalManager {
    /// 创建新信号管理器
    pub fn new() -> Self {
        Self {
            session_switched: SessionSwitchedSignal::new(),
            state_changed: StateChangedSignal::new(),
        }
    }
}

impl Default for SignalManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局信号管理器实例
static SIGNAL_MANAGER: once_cell::sync::Lazy<SignalManager> = 
    once_cell::sync::Lazy::new(SignalManager::new);

/// 获取信号管理器
pub fn get_signal_manager() -> &'static SignalManager {
    &SIGNAL_MANAGER
}

/// 订阅会话切换事件
pub fn on_session_switched() -> SignalSubscriber<String> {
    SIGNAL_MANAGER.session_switched.subscribe()
}

/// 发送会话切换事件
pub fn emit_session_switched(session_id: String) {
    let _ = SIGNAL_MANAGER.session_switched.send(session_id);
}

/// 订阅状态变更事件
pub fn on_state_changed() -> SignalSubscriber<String> {
    SIGNAL_MANAGER.state_changed.subscribe()
}

/// 发送状态变更事件
pub fn emit_state_changed(field: String) {
    let _ = SIGNAL_MANAGER.state_changed.send(field);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};
    
    #[tokio::test]
    async fn test_signal_creation() {
        let signal: Signal<i32> = Signal::new();
        let count = signal.subscriber_count().await;
        assert_eq!(count, 0);
    }
    
    #[tokio::test]
    async fn test_signal_subscribe() {
        let signal: Signal<i32> = Signal::new();
        let _receiver = signal.subscribe();
        
        let count = signal.subscriber_count().await;
        assert_eq!(count, 1);
    }
    
    #[tokio::test]
    async fn test_signal_send_receive() {
        let signal: Signal<i32> = Signal::new();
        let mut receiver = signal.subscribe();
        
        signal.send(42).unwrap();
        
        let value = receiver.try_recv().unwrap();
        assert_eq!(value, 42);
    }
    
    #[tokio::test]
    async fn test_signal_manager() {
        let manager = SignalManager::new();
        
        let mut receiver = manager.session_switched.subscribe();
        
        manager.session_switched.send("session-123".to_string()).unwrap();
        
        let value = receiver.try_recv().unwrap();
        assert_eq!(value, "session-123");
    }
    
    #[test]
    fn test_global_signal_manager() {
        let manager = get_signal_manager();
        
        let _receiver = on_session_switched();
        
        emit_session_switched("test-session".to_string());
    }
}
