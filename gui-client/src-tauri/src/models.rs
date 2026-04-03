use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type ProviderId = String;
pub type ConversationId = String;
pub type TaskId = String;
pub type ModelId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: i64,
    pub model: Option<String>,
    pub tokens: Option<Tokens>,
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tokens {
    pub prompt: u32,
    pub completion: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub thinking: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
    pub result: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: ConversationId,
    pub title: String,
    pub messages: Vec<Message>,
    pub model: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub completed_at: Option<i64>,
    pub progress: u8,
    pub subtasks: Vec<SubTask>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: ModelId,
    pub name: String,
    pub provider: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub max_tokens: u32,
    pub context_window: u32,
    pub pricing: Pricing,
    pub is_available: bool,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pricing {
    pub input: f64,
    pub output: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelProvider {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub models: Vec<Model>,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub language: String,
    pub default_model: String,
    pub api_keys: HashMap<String, String>,
    pub shortcuts: HashMap<String, String>,
    pub auto_save: bool,
    pub stream_response: bool,
    pub show_thinking: bool,
    pub max_context_messages: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        let mut shortcuts = HashMap::new();
        shortcuts.insert("new-chat".to_string(), "Ctrl+N".to_string());
        shortcuts.insert("send-message".to_string(), "Enter".to_string());
        shortcuts.insert("new-line".to_string(), "Shift+Enter".to_string());
        shortcuts.insert("search".to_string(), "Ctrl+K".to_string());

        Self {
            theme: "system".to_string(),
            language: "zh-CN".to_string(),
            default_model: "claude-3-5-sonnet".to_string(),
            api_keys: HashMap::new(),
            shortcuts,
            auto_save: true,
            stream_response: true,
            show_thinking: false,
            max_context_messages: 20,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            message: None,
        }
    }

    pub fn error(error: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.into()),
            message: None,
        }
    }
}

pub struct AppState {
    pub settings: tokio::sync::RwLock<AppSettings>,
    pub conversations: tokio::sync::RwLock<Vec<Conversation>>,
    pub tasks: tokio::sync::RwLock<Vec<Task>>,
    pub providers: tokio::sync::RwLock<Vec<ModelProvider>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            settings: tokio::sync::RwLock::new(AppSettings::default()),
            conversations: tokio::sync::RwLock::new(Vec::new()),
            tasks: tokio::sync::RwLock::new(Vec::new()),
            providers: tokio::sync::RwLock::new(Self::default_providers()),
        }
    }

    fn default_providers() -> Vec<ModelProvider> {
        vec![
            ModelProvider {
                id: "anthropic".to_string(),
                name: "Anthropic".to_string(),
                icon: Some("claude".to_string()),
                api_key: None,
                base_url: None,
                models: vec![
                    Model {
                        id: "claude-3-5-sonnet".to_string(),
                        name: "Claude 3.5 Sonnet".to_string(),
                        provider: "anthropic".to_string(),
                        description: "Most intelligent model".to_string(),
                        capabilities: vec!["coding".to_string(), "analysis".to_string(), "writing".to_string()],
                        max_tokens: 8192,
                        context_window: 200000,
                        pricing: Pricing { input: 3.0, output: 15.0 },
                        is_available: true,
                        is_default: true,
                    },
                    Model {
                        id: "claude-3-opus".to_string(),
                        name: "Claude 3 Opus".to_string(),
                        provider: "anthropic".to_string(),
                        description: "Powerful model for complex tasks".to_string(),
                        capabilities: vec!["coding".to_string(), "analysis".to_string(), "writing".to_string()],
                        max_tokens: 4096,
                        context_window: 200000,
                        pricing: Pricing { input: 15.0, output: 75.0 },
                        is_available: true,
                        is_default: false,
                    },
                    Model {
                        id: "claude-3-sonnet".to_string(),
                        name: "Claude 3 Sonnet".to_string(),
                        provider: "anthropic".to_string(),
                        description: "Fast and efficient model".to_string(),
                        capabilities: vec!["coding".to_string(), "analysis".to_string()],
                        max_tokens: 4096,
                        context_window: 200000,
                        pricing: Pricing { input: 3.0, output: 15.0 },
                        is_available: true,
                        is_default: false,
                    },
                ],
                is_enabled: true,
            },
            ModelProvider {
                id: "openai".to_string(),
                name: "OpenAI".to_string(),
                icon: Some("openai".to_string()),
                api_key: None,
                base_url: None,
                models: vec![
                    Model {
                        id: "gpt-4o".to_string(),
                        name: "GPT-4o".to_string(),
                        provider: "openai".to_string(),
                        description: "Most capable GPT-4 model".to_string(),
                        capabilities: vec!["coding".to_string(), "analysis".to_string(), "vision".to_string()],
                        max_tokens: 4096,
                        context_window: 128000,
                        pricing: Pricing { input: 5.0, output: 15.0 },
                        is_available: true,
                        is_default: false,
                    },
                    Model {
                        id: "gpt-4-turbo".to_string(),
                        name: "GPT-4 Turbo".to_string(),
                        provider: "openai".to_string(),
                        description: "Fast and powerful".to_string(),
                        capabilities: vec!["coding".to_string(), "analysis".to_string()],
                        max_tokens: 4096,
                        context_window: 128000,
                        pricing: Pricing { input: 10.0, output: 30.0 },
                        is_available: true,
                        is_default: false,
                    },
                    Model {
                        id: "gpt-3.5-turbo".to_string(),
                        name: "GPT-3.5 Turbo".to_string(),
                        provider: "openai".to_string(),
                        description: "Cost-effective and fast".to_string(),
                        capabilities: vec!["coding".to_string(), "analysis".to_string()],
                        max_tokens: 4096,
                        context_window: 16000,
                        pricing: Pricing { input: 0.5, output: 1.5 },
                        is_available: true,
                        is_default: false,
                    },
                ],
                is_enabled: false,
            },
            ModelProvider {
                id: "google".to_string(),
                name: "Google AI".to_string(),
                icon: Some("google".to_string()),
                api_key: None,
                base_url: None,
                models: vec![
                    Model {
                        id: "gemini-1.5-pro".to_string(),
                        name: "Gemini 1.5 Pro".to_string(),
                        provider: "google".to_string(),
                        description: "Google's most capable model".to_string(),
                        capabilities: vec!["coding".to_string(), "analysis".to_string(), "vision".to_string()],
                        max_tokens: 8192,
                        context_window: 1000000,
                        pricing: Pricing { input: 0.15, output: 0.6 },
                        is_available: true,
                        is_default: false,
                    },
                    Model {
                        id: "gemini-1.5-flash".to_string(),
                        name: "Gemini 1.5 Flash".to_string(),
                        provider: "google".to_string(),
                        description: "Fast and cost-effective".to_string(),
                        capabilities: vec!["coding".to_string(), "analysis".to_string(), "vision".to_string()],
                        max_tokens: 8192,
                        context_window: 1000000,
                        pricing: Pricing { input: 0.075, output: 0.3 },
                        is_available: true,
                        is_default: false,
                    },
                    Model {
                        id: "gemini-1.0-pro".to_string(),
                        name: "Gemini 1.0 Pro".to_string(),
                        provider: "google".to_string(),
                        description: "Reliable performance".to_string(),
                        capabilities: vec!["coding".to_string(), "analysis".to_string()],
                        max_tokens: 4096,
                        context_window: 32768,
                        pricing: Pricing { input: 0.125, output: 0.375 },
                        is_available: true,
                        is_default: false,
                    },
                ],
                is_enabled: false,
            },
            ModelProvider {
                id: "meta".to_string(),
                name: "Meta AI".to_string(),
                icon: Some("meta".to_string()),
                api_key: None,
                base_url: None,
                models: vec![
                    Model {
                        id: "llama-3.1-70b".to_string(),
                        name: "Llama 3.1 70B".to_string(),
                        provider: "meta".to_string(),
                        description: "Meta's most powerful model".to_string(),
                        capabilities: vec!["coding".to_string(), "analysis".to_string(), "writing".to_string()],
                        max_tokens: 8192,
                        context_window: 128000,
                        pricing: Pricing { input: 1.5, output: 6.0 },
                        is_available: true,
                        is_default: false,
                    },
                    Model {
                        id: "llama-3.1-8b".to_string(),
                        name: "Llama 3.1 8B".to_string(),
                        provider: "meta".to_string(),
                        description: "Fast and efficient".to_string(),
                        capabilities: vec!["coding".to_string(), "analysis".to_string()],
                        max_tokens: 8192,
                        context_window: 128000,
                        pricing: Pricing { input: 0.15, output: 0.6 },
                        is_available: true,
                        is_default: false,
                    },
                    Model {
                        id: "llama-3-70b".to_string(),
                        name: "Llama 3 70B".to_string(),
                        provider: "meta".to_string(),
                        description: "Powerful open model".to_string(),
                        capabilities: vec!["coding".to_string(), "analysis".to_string()],
                        max_tokens: 8192,
                        context_window: 128000,
                        pricing: Pricing { input: 1.0, output: 4.0 },
                        is_available: true,
                        is_default: false,
                    },
                ],
                is_enabled: false,
            },
            ModelProvider {
                id: "mistral".to_string(),
                name: "Mistral AI".to_string(),
                icon: Some("mistral".to_string()),
                api_key: None,
                base_url: None,
                models: vec![
                    Model {
                        id: "mistral-large".to_string(),
                        name: "Mistral Large".to_string(),
                        provider: "mistral".to_string(),
                        description: "Mistral's flagship model".to_string(),
                        capabilities: vec!["coding".to_string(), "analysis".to_string(), "writing".to_string()],
                        max_tokens: 8192,
                        context_window: 128000,
                        pricing: Pricing { input: 2.0, output: 6.0 },
                        is_available: true,
                        is_default: false,
                    },
                    Model {
                        id: "mistral-medium".to_string(),
                        name: "Mistral Medium".to_string(),
                        provider: "mistral".to_string(),
                        description: "Balanced performance".to_string(),
                        capabilities: vec!["coding".to_string(), "analysis".to_string()],
                        max_tokens: 8192,
                        context_window: 128000,
                        pricing: Pricing { input: 0.5, output: 1.5 },
                        is_available: true,
                        is_default: false,
                    },
                    Model {
                        id: "mistral-small".to_string(),
                        name: "Mistral Small".to_string(),
                        provider: "mistral".to_string(),
                        description: "Fast and cost-effective".to_string(),
                        capabilities: vec!["coding".to_string(), "analysis".to_string()],
                        max_tokens: 8192,
                        context_window: 128000,
                        pricing: Pricing { input: 0.1, output: 0.3 },
                        is_available: true,
                        is_default: false,
                    },
                ],
                is_enabled: false,
            },
        ]
    }
}
