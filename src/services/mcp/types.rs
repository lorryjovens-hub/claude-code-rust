//! MCP 类型定义
//! 
//! 这个模块定义了 MCP 协议的核心类型

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP 传输协议类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum McpTransport {
    /// 标准输入输出传输
    Stdio,
    /// 服务器发送事件
    Sse,
    /// SSE IDE 扩展
    #[serde(rename = "sse-ide")]
    SseIde,
    /// HTTP 传输
    Http,
    /// WebSocket 传输
    Ws,
    /// WebSocket IDE 扩展
    #[serde(rename = "ws-ide")]
    WsIde,
    /// SDK 集成
    Sdk,
    /// Claude.ai 代理
    #[serde(rename = "claudeai-proxy")]
    ClaudeAiProxy,
}

/// MCP Stdio 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpStdioConfig {
    /// 命令
    pub command: String,
    
    /// 参数
    #[serde(default)]
    pub args: Vec<String>,
    
    /// 环境变量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
}

/// MCP SSE 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSseConfig {
    /// URL
    pub url: String,
    
    /// 请求头
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    
    /// Headers helper
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers_helper: Option<String>,
    
    /// OAuth 配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth: Option<McpOAuthConfig>,
}

/// MCP HTTP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpHttpConfig {
    /// URL
    pub url: String,
    
    /// 请求头
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    
    /// Headers helper
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers_helper: Option<String>,
    
    /// OAuth 配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth: Option<McpOAuthConfig>,
}

/// MCP WebSocket 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpWebSocketConfig {
    /// URL
    pub url: String,
    
    /// 请求头
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    
    /// Headers helper
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers_helper: Option<String>,
}

/// MCP SDK 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSdkConfig {
    /// 名称
    pub name: String,
}

/// MCP Claude.ai 代理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpClaudeAiProxyConfig {
    /// URL
    pub url: String,
    
    /// ID
    pub id: String,
}

/// MCP OAuth 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpOAuthConfig {
    /// 客户端 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    
    /// 回调端口
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_port: Option<u16>,
    
    /// 认证服务器元数据 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_server_metadata_url: Option<String>,
    
    /// Cross-App Access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xaa: Option<bool>,
}

/// MCP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum McpServerConfig {
    /// Stdio 配置
    Stdio(McpStdioConfig),
    
    /// SSE 配置
    Sse(McpSseConfig),
    
    /// SSE IDE 配置
    #[serde(rename = "sse-ide")]
    SseIde {
        url: String,
        ide_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        ide_running_in_windows: Option<bool>,
    },
    
    /// HTTP 配置
    Http(McpHttpConfig),
    
    /// WebSocket 配置
    Ws(McpWebSocketConfig),
    
    /// WebSocket IDE 配置
    #[serde(rename = "ws-ide")]
    WsIde {
        url: String,
        ide_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        auth_token: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ide_running_in_windows: Option<bool>,
    },
    
    /// SDK 配置
    Sdk(McpSdkConfig),
    
    /// Claude.ai 代理配置
    #[serde(rename = "claudeai-proxy")]
    ClaudeAiProxy(McpClaudeAiProxyConfig),
}

/// MCP 服务器连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum McpConnectionStatus {
    /// 已连接
    Connected,
    /// 已断开
    Disconnected,
    /// 连接中
    Connecting,
    /// 错误
    Error,
}

/// MCP 服务器连接
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConnection {
    /// 服务器名称
    pub name: String,
    
    /// 配置
    pub config: McpServerConfig,
    
    /// 状态
    pub status: McpConnectionStatus,
    
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// MCP 服务器能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerCapabilities {
    /// 支持工具
    pub tools: bool,
    
    /// 支持资源
    pub resources: bool,
    
    /// 支持提示
    pub prompts: bool,
}

/// MCP 资源
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    /// URI
    pub uri: String,
    
    /// 名称
    pub name: String,
    
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// MIME 类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mcp_stdio_config() {
        let config = McpStdioConfig {
            command: "node".to_string(),
            args: vec!["server.js".to_string()],
            env: None,
        };
        
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"command\":\"node\""));
    }
    
    #[test]
    fn test_mcp_sse_config() {
        let config = McpSseConfig {
            url: "http://localhost:3000/sse".to_string(),
            headers: None,
            headers_helper: None,
            oauth: None,
        };
        
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"url\":\"http://localhost:3000/sse\""));
    }
    
    #[test]
    fn test_mcp_server_config() {
        let config = McpServerConfig::Stdio(McpStdioConfig {
            command: "node".to_string(),
            args: vec![],
            env: None,
        });
        
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"type\":\"stdio\""));
    }
}
