//! 代理颜色管理器
//! 
//! 这个模块实现了代理颜色管理功能

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 代理颜色名称
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentColor {
    Red,
    Green,
    Blue,
    Yellow,
    Magenta,
    Cyan,
    White,
    BrightRed,
    BrightGreen,
    BrightBlue,
    BrightYellow,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl AgentColor {
    /// 获取颜色代码
    pub fn code(&self) -> &str {
        match self {
            Self::Red => "\x1b[31m",
            Self::Green => "\x1b[32m",
            Self::Blue => "\x1b[34m",
            Self::Yellow => "\x1b[33m",
            Self::Magenta => "\x1b[35m",
            Self::Cyan => "\x1b[36m",
            Self::White => "\x1b[37m",
            Self::BrightRed => "\x1b[91m",
            Self::BrightGreen => "\x1b[92m",
            Self::BrightBlue => "\x1b[94m",
            Self::BrightYellow => "\x1b[93m",
            Self::BrightMagenta => "\x1b[95m",
            Self::BrightCyan => "\x1b[96m",
            Self::BrightWhite => "\x1b[97m",
        }
    }
    
    /// 获取重置代码
    pub fn reset() -> &'static str {
        "\x1b[0m"
    }
    
    /// 着色文本
    pub fn colorize(&self, text: &str) -> String {
        format!("{}{}{}", self.code(), text, Self::reset())
    }
    
    /// 获取所有颜色
    pub fn all() -> Vec<Self> {
        vec![
            Self::Red,
            Self::Green,
            Self::Blue,
            Self::Yellow,
            Self::Magenta,
            Self::Cyan,
            Self::White,
            Self::BrightRed,
            Self::BrightGreen,
            Self::BrightBlue,
            Self::BrightYellow,
            Self::BrightMagenta,
            Self::BrightCyan,
            Self::BrightWhite,
        ]
    }
}

/// 代理颜色管理器
pub struct AgentColorManager {
    /// 代理颜色映射
    color_map: Arc<RwLock<HashMap<String, AgentColor>>>,
    
    /// 颜色索引
    color_index: Arc<RwLock<usize>>,
}

impl AgentColorManager {
    /// 创建新的颜色管理器
    pub fn new() -> Self {
        Self {
            color_map: Arc::new(RwLock::new(HashMap::new())),
            color_index: Arc::new(RwLock::new(0)),
        }
    }
    
    /// 获取或分配代理颜色
    pub async fn get_or_assign(&self, agent_id: &str) -> AgentColor {
        let mut color_map = self.color_map.write().await;
        
        if let Some(&color) = color_map.get(agent_id) {
            return color;
        }
        
        // 分配新颜色
        let colors = AgentColor::all();
        let mut index = self.color_index.write().await;
        
        let color = colors[*index % colors.len()];
        *index += 1;
        
        color_map.insert(agent_id.to_string(), color);
        
        color
    }
    
    /// 获取代理颜色
    pub async fn get(&self, agent_id: &str) -> Option<AgentColor> {
        self.color_map.read().await.get(agent_id).copied()
    }
    
    /// 设置代理颜色
    pub async fn set(&self, agent_id: String, color: AgentColor) {
        self.color_map.write().await.insert(agent_id, color);
    }
    
    /// 移除代理颜色
    pub async fn remove(&self, agent_id: &str) -> Option<AgentColor> {
        self.color_map.write().await.remove(agent_id)
    }
    
    /// 清空所有颜色
    pub async fn clear(&self) {
        self.color_map.write().await.clear();
        *self.color_index.write().await = 0;
    }
    
    /// 着色代理名称
    pub async fn colorize_name(&self, agent_id: &str, name: &str) -> String {
        if let Some(color) = self.get(agent_id).await {
            color.colorize(name)
        } else {
            name.to_string()
        }
    }
}

impl Default for AgentColorManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_color() {
        let color = AgentColor::Red;
        
        assert_eq!(color.code(), "\x1b[31m");
        
        let colored = color.colorize("test");
        assert!(colored.contains("\x1b[31m"));
        assert!(colored.contains("\x1b[0m"));
    }
    
    #[tokio::test]
    async fn test_color_manager() {
        let manager = AgentColorManager::new();
        
        let color1 = manager.get_or_assign("agent-1").await;
        let color2 = manager.get_or_assign("agent-2").await;
        
        assert_ne!(color1, color2);
        
        // 再次获取应该返回相同的颜色
        let color1_again = manager.get_or_assign("agent-1").await;
        assert_eq!(color1, color1_again);
    }
    
    #[tokio::test]
    async fn test_colorize_name() {
        let manager = AgentColorManager::new();
        
        manager.set("agent-1".to_string(), AgentColor::Red).await;
        
        let colored = manager.colorize_name("agent-1", "TestAgent").await;
        assert!(colored.contains("\x1b[31m"));
    }
}
