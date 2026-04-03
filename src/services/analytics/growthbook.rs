//! GrowthBook 特性开关系统
//! 
//! 这个模块实现了 GrowthBook 特性开关系统，对应 TypeScript 的 growthbook.ts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

/// GitHub Actions 元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubActionsMetadata {
    /// 工作流名称
    pub workflow: String,
    /// 运行 ID
    pub run_id: String,
    /// 运行编号
    pub run_number: String,
    /// 仓库
    pub repository: String,
    /// 分支
    pub ref_name: String,
    /// SHA
    pub sha: String,
}

/// GrowthBook 用户属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthBookUserAttributes {
    /// 用户 ID
    pub id: String,
    
    /// 会话 ID
    pub session_id: String,
    
    /// 设备 ID
    pub device_id: String,
    
    /// 平台
    pub platform: String,
    
    /// API 基础 URL 主机
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_base_url_host: Option<String>,
    
    /// 组织 UUID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_uuid: Option<String>,
    
    /// 账户 UUID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_uuid: Option<String>,
    
    /// 用户类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_type: Option<String>,
    
    /// 订阅类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_type: Option<String>,
    
    /// 速率限制层级
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_tier: Option<String>,
    
    /// 首次 Token 时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_token_time: Option<i64>,
    
    /// 邮箱
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    
    /// 应用版本
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_version: Option<String>,
    
    /// GitHub Actions 元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github: Option<GitHubActionsMetadata>,
}

/// 实验数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentData {
    /// 实验 ID
    pub experiment_id: String,
    
    /// 变体 ID
    pub variation_id: u32,
    
    /// 是否在实验中
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_experiment: Option<bool>,
    
    /// 哈希属性
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_attribute: Option<String>,
    
    /// 哈希值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_value: Option<String>,
}

/// 特性标志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    /// 特性名称
    pub name: String,
    
    /// 特性值
    pub value: serde_json::Value,
    
    /// 是否启用
    pub enabled: bool,
    
    /// 实验数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experiment: Option<ExperimentData>,
}

/// GrowthBook 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthBookConfig {
    /// API 主机
    pub api_host: String,
    
    /// 客户端密钥
    pub client_key: String,
    
    /// 用户属性
    pub attributes: GrowthBookUserAttributes,
    
    /// 是否启用远程评估
    pub remote_eval: bool,
    
    /// 缓存键属性
    pub cache_key_attributes: Vec<String>,
    
    /// 刷新间隔（毫秒）
    pub refresh_interval_ms: u64,
}

impl Default for GrowthBookConfig {
    fn default() -> Self {
        Self {
            api_host: "https://api.anthropic.com/".to_string(),
            client_key: String::new(),
            attributes: GrowthBookUserAttributes {
                id: String::new(),
                session_id: String::new(),
                device_id: String::new(),
                platform: std::env::consts::OS.to_string(),
                api_base_url_host: None,
                organization_uuid: None,
                account_uuid: None,
                user_type: None,
                subscription_type: None,
                rate_limit_tier: None,
                first_token_time: None,
                email: None,
                app_version: None,
                github: None,
            },
            remote_eval: true,
            cache_key_attributes: vec!["id".to_string(), "organization_uuid".to_string()],
            refresh_interval_ms: if option_env!("USER_TYPE") == Some("ant") {
                20 * 60 * 1000 // 20 分钟
            } else {
                6 * 60 * 60 * 1000 // 6 小时
            },
        }
    }
}

/// GrowthBook 客户端
pub struct GrowthBookClient {
    /// 配置
    config: GrowthBookConfig,
    
    /// 特性值缓存
    features: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    
    /// 实验数据
    experiments: Arc<RwLock<HashMap<String, ExperimentData>>>,
    
    /// 环境变量覆盖
    env_overrides: HashMap<String, serde_json::Value>,
    
    /// 配置覆盖
    config_overrides: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    
    /// 是否已初始化
    initialized: Arc<RwLock<bool>>,
}

impl GrowthBookClient {
    /// 创建新的 GrowthBook 客户端
    pub fn new(config: GrowthBookConfig) -> Self {
        // 解析环境变量覆盖
        let env_overrides = Self::parse_env_overrides();
        
        Self {
            config,
            features: Arc::new(RwLock::new(HashMap::new())),
            experiments: Arc::new(RwLock::new(HashMap::new())),
            env_overrides,
            config_overrides: Arc::new(RwLock::new(HashMap::new())),
            initialized: Arc::new(RwLock::new(false)),
        }
    }
    
    /// 解析环境变量覆盖
    fn parse_env_overrides() -> HashMap<String, serde_json::Value> {
        let mut overrides = HashMap::new();
        
        if option_env!("USER_TYPE") == Some("ant") {
            if let Ok(raw) = std::env::var("CLAUDE_INTERNAL_FC_OVERRIDES") {
                if let Ok(json) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&raw) {
                    overrides = json;
                    tracing::debug!(
                        "GrowthBook: Using env var overrides for {} features",
                        overrides.len()
                    );
                }
            }
        }
        
        overrides
    }
    
    /// 初始化客户端
    pub async fn init(&self) -> crate::error::Result<()> {
        tracing::debug!("Initializing GrowthBook client");
        
        // TODO: 实现远程评估
        // 1. 从服务器获取特性值
        // 2. 处理响应数据
        // 3. 更新缓存
        
        *self.initialized.write().await = true;
        
        tracing::info!("GrowthBook client initialized successfully");
        Ok(())
    }
    
    /// 获取特性值（阻塞式）
    pub async fn get_feature_value<T>(&self, feature: &str, default_value: T) -> T
    where
        T: for<'de> Deserialize<'de>,
    {
        // 1. 检查环境变量覆盖
        if let Some(value) = self.env_overrides.get(feature) {
            if let Ok(v) = serde_json::from_value(value.clone()) {
                return v;
            }
        }
        
        // 2. 检查配置覆盖
        if let Some(value) = self.config_overrides.read().await.get(feature) {
            if let Ok(v) = serde_json::from_value(value.clone()) {
                return v;
            }
        }
        
        // 3. 从缓存获取
        if let Some(value) = self.features.read().await.get(feature) {
            if let Ok(v) = serde_json::from_value(value.clone()) {
                return v;
            }
        }
        
        default_value
    }
    
    /// 获取特性值（缓存，可能过期）
    pub fn get_feature_value_cached<T>(&self, feature: &str, default_value: T) -> T
    where
        T: for<'de> Deserialize<'de>,
    {
        // 1. 检查环境变量覆盖
        if let Some(value) = self.env_overrides.get(feature) {
            if let Ok(v) = serde_json::from_value(value.clone()) {
                return v;
            }
        }
        
        // TODO: 实现磁盘缓存读取
        
        default_value
    }
    
    /// 检查特性标志
    pub async fn check_feature_gate(&self, gate: &str) -> bool {
        self.get_feature_value(gate, false).await
    }
    
    /// 检查特性标志（缓存）
    pub fn check_feature_gate_cached(&self, gate: &str) -> bool {
        self.get_feature_value_cached(gate, false)
    }
    
    /// 设置配置覆盖
    pub async fn set_config_override(&self, feature: String, value: Option<serde_json::Value>) {
        let mut overrides = self.config_overrides.write().await;
        
        match value {
            Some(v) => {
                overrides.insert(feature, v);
            }
            None => {
                overrides.remove(&feature);
            }
        }
    }
    
    /// 清除所有配置覆盖
    pub async fn clear_config_overrides(&self) {
        self.config_overrides.write().await.clear();
    }
    
    /// 刷新特性值
    pub async fn refresh_features(&self) -> crate::error::Result<()> {
        tracing::debug!("Refreshing GrowthBook features");
        
        // TODO: 实现特性刷新
        // 1. 从服务器获取最新特性值
        // 2. 更新缓存
        // 3. 同步到磁盘
        
        Ok(())
    }
    
    /// 重置客户端
    pub async fn reset(&self) {
        self.features.write().await.clear();
        self.experiments.write().await.clear();
        *self.initialized.write().await = false;
    }
}

impl std::fmt::Debug for GrowthBookClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GrowthBookClient")
            .field("initialized", &self.initialized.try_read().ok())
            .finish_non_exhaustive()
    }
}

/// 全局 GrowthBook 客户端
static GROWTHBOOK_CLIENT: once_cell::sync::Lazy<Arc<RwLock<Option<GrowthBookClient>>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(None)));

/// 初始化 GrowthBook
pub async fn init() -> crate::error::Result<()> {
    let config = GrowthBookConfig::default();
    let client = GrowthBookClient::new(config);
    client.init().await?;
    
    *GROWTHBOOK_CLIENT.write().await = Some(client);
    
    Ok(())
}

/// 获取 GrowthBook 客户端
pub async fn get_client() -> Option<Arc<GrowthBookClient>> {
    // 这里简化实现，实际应该返回 Arc
    None
}

/// 获取特性值
pub async fn get_feature_value<T>(feature: &str, default_value: T) -> T
where
    T: for<'de> Deserialize<'de>,
{
    if let Some(client) = get_client().await {
        return client.get_feature_value(feature, default_value).await;
    }
    default_value
}

/// 检查特性标志
pub async fn check_feature_gate(gate: &str) -> bool {
    get_feature_value(gate, false).await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_growth_book_config_default() {
        let config = GrowthBookConfig::default();
        assert_eq!(config.api_host, "https://api.anthropic.com/");
        assert!(config.remote_eval);
    }
    
    #[test]
    fn test_user_attributes() {
        let attrs = GrowthBookUserAttributes {
            id: "test-id".to_string(),
            session_id: "session-123".to_string(),
            device_id: "device-456".to_string(),
            platform: "win32".to_string(),
            api_base_url_host: None,
            organization_uuid: None,
            account_uuid: None,
            user_type: None,
            subscription_type: None,
            rate_limit_tier: None,
            first_token_time: None,
            email: None,
            app_version: None,
            github: None,
        };
        
        assert_eq!(attrs.id, "test-id");
        assert_eq!(attrs.platform, "win32");
    }
    
    #[tokio::test]
    async fn test_growth_book_client() {
        let config = GrowthBookConfig::default();
        let client = GrowthBookClient::new(config);
        
        // 测试获取特性值
        let value: bool = client.get_feature_value("test_feature", false).await;
        assert!(!value);
    }
    
    #[tokio::test]
    async fn test_config_overrides() {
        let config = GrowthBookConfig::default();
        let client = GrowthBookClient::new(config);
        
        // 设置覆盖
        client.set_config_override("test_feature".to_string(), Some(serde_json::json!(true))).await;
        
        // 获取特性值
        let value: bool = client.get_feature_value("test_feature", false).await;
        assert!(value);
        
        // 清除覆盖
        client.clear_config_overrides().await;
        
        // 再次获取
        let value: bool = client.get_feature_value("test_feature", false).await;
        assert!(!value);
    }
}
