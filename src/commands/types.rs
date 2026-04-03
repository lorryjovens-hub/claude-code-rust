//! 命令类型系统
//! 
//! 这个模块定义了命令系统的核心类型，对应 TypeScript 的 types/command.ts

use serde::{Deserialize, Serialize};

/// 命令可用性类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandAvailability {
    /// claude.ai OAuth 订阅者 (Pro/Max/Team/Enterprise)
    ClaudeAi,
    /// Console API 密钥用户 (直接使用 api.anthropic.com)
    Console,
}

/// 命令来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandSource {
    /// 内置命令
    Builtin,
    /// MCP 服务器
    Mcp,
    /// 插件
    Plugin,
    /// 打包的技能
    Bundled,
    /// 用户设置
    User,
    /// 项目设置
    Project,
}

/// 命令加载来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadedFrom {
    /// 旧版命令目录
    CommandsDeprecated,
    /// 技能目录
    Skills,
    /// 插件
    Plugin,
    /// 托管
    Managed,
    /// 打包
    Bundled,
    /// MCP
    Mcp,
}

/// 命令基础属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandBase {
    /// 命令名称
    pub name: String,
    
    /// 命令描述
    pub description: String,
    
    /// 用户指定的描述标志
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_user_specified_description: Option<bool>,
    
    /// 命令别名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliases: Option<Vec<String>>,
    
    /// 可用性要求
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability: Option<Vec<CommandAvailability>>,
    
    /// 是否隐藏
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_hidden: Option<bool>,
    
    /// 是否为 MCP 命令
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_mcp: Option<bool>,
    
    /// 参数提示
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argument_hint: Option<String>,
    
    /// 使用场景说明
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when_to_use: Option<String>,
    
    /// 命令版本
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    
    /// 是否禁用模型调用
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_model_invocation: Option<bool>,
    
    /// 用户是否可调用
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_invocable: Option<bool>,
    
    /// 加载来源
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loaded_from: Option<LoadedFrom>,
    
    /// 命令类型标识
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<CommandKind>,
    
    /// 是否立即执行
    #[serde(skip_serializing_if = "Option::is_none")]
    pub immediate: Option<bool>,
    
    /// 是否敏感命令
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_sensitive: Option<bool>,
}

/// 命令类型标识
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandKind {
    /// 工作流命令
    Workflow,
}

/// 本地命令结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LocalCommandResult {
    /// 文本结果
    Text {
        value: String,
    },
    /// 压缩结果
    Compact {
        compaction_result: CompactionResult,
        display_text: Option<String>,
    },
    /// 跳过
    Skip,
}

/// 压缩结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionResult {
    /// 压缩的 token 数
    pub tokens_compacted: usize,
    /// 保留的 token 数
    pub tokens_kept: usize,
    /// 压缩比例
    pub compression_ratio: f32,
}

/// 执行上下文
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionContext {
    /// 内联执行（默认）
    Inline,
    /// 分叉执行（作为子代理）
    Fork,
}

/// 努力程度值
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffortValue {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
}

/// 提示命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCommand {
    /// 基础属性
    #[serde(flatten)]
    pub base: CommandBase,
    
    /// 进度消息
    pub progress_message: String,
    
    /// 内容长度（字符数）
    pub content_length: usize,
    
    /// 参数名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arg_names: Option<Vec<String>>,
    
    /// 允许的工具
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<Vec<String>>,
    
    /// 使用的模型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    
    /// 命令来源
    pub source: CommandSource,
    
    /// 插件信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_info: Option<PluginInfo>,
    
    /// 是否禁用非交互模式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_non_interactive: Option<bool>,
    
    /// 执行上下文
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<ExecutionContext>,
    
    /// 分叉时使用的代理类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    
    /// 努力程度
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<EffortValue>,
    
    /// 适用的文件路径模式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths: Option<Vec<String>>,
}

/// 插件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// 插件清单
    pub plugin_manifest: PluginManifest,
    /// 仓库地址
    pub repository: String,
}

/// 插件清单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// 插件名称
    pub name: String,
    /// 插件版本
    pub version: String,
    /// 插件描述
    pub description: String,
}

/// 本地命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalCommand {
    /// 基础属性
    #[serde(flatten)]
    pub base: CommandBase,
    
    /// 是否支持非交互模式
    pub supports_non_interactive: bool,
}

/// 本地 JSX 命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalJsxCommand {
    /// 基础属性
    #[serde(flatten)]
    pub base: CommandBase,
}

/// 命令枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Command {
    /// 提示命令
    #[serde(rename = "prompt")]
    Prompt(PromptCommand),
    
    /// 本地命令
    #[serde(rename = "local")]
    Local(LocalCommand),
    
    /// 本地 JSX 命令
    #[serde(rename = "local-jsx")]
    LocalJsx(LocalJsxCommand),
}

impl Command {
    /// 获取命令名称
    pub fn name(&self) -> &str {
        match self {
            Command::Prompt(cmd) => &cmd.base.name,
            Command::Local(cmd) => &cmd.base.name,
            Command::LocalJsx(cmd) => &cmd.base.name,
        }
    }
    
    /// 获取命令描述
    pub fn description(&self) -> &str {
        match self {
            Command::Prompt(cmd) => &cmd.base.description,
            Command::Local(cmd) => &cmd.base.description,
            Command::LocalJsx(cmd) => &cmd.base.description,
        }
    }
    
    /// 获取命令别名
    pub fn aliases(&self) -> Option<&[String]> {
        match self {
            Command::Prompt(cmd) => cmd.base.aliases.as_deref(),
            Command::Local(cmd) => cmd.base.aliases.as_deref(),
            Command::LocalJsx(cmd) => cmd.base.aliases.as_deref(),
        }
    }
    
    /// 是否隐藏
    pub fn is_hidden(&self) -> bool {
        match self {
            Command::Prompt(cmd) => cmd.base.is_hidden.unwrap_or(false),
            Command::Local(cmd) => cmd.base.is_hidden.unwrap_or(false),
            Command::LocalJsx(cmd) => cmd.base.is_hidden.unwrap_or(false),
        }
    }
    
    /// 是否为 MCP 命令
    pub fn is_mcp(&self) -> bool {
        match self {
            Command::Prompt(cmd) => cmd.base.is_mcp.unwrap_or(false),
            Command::Local(cmd) => cmd.base.is_mcp.unwrap_or(false),
            Command::LocalJsx(cmd) => cmd.base.is_mcp.unwrap_or(false),
        }
    }
    
    /// 是否立即执行
    pub fn is_immediate(&self) -> bool {
        match self {
            Command::Prompt(cmd) => cmd.base.immediate.unwrap_or(false),
            Command::Local(cmd) => cmd.base.immediate.unwrap_or(false),
            Command::LocalJsx(cmd) => cmd.base.immediate.unwrap_or(false),
        }
    }
    
    /// 是否敏感命令
    pub fn is_sensitive(&self) -> bool {
        match self {
            Command::Prompt(cmd) => cmd.base.is_sensitive.unwrap_or(false),
            Command::Local(cmd) => cmd.base.is_sensitive.unwrap_or(false),
            Command::LocalJsx(cmd) => cmd.base.is_sensitive.unwrap_or(false),
        }
    }
    
    /// 是否禁用模型调用
    pub fn disable_model_invocation(&self) -> bool {
        match self {
            Command::Prompt(cmd) => cmd.base.disable_model_invocation.unwrap_or(false),
            Command::Local(cmd) => cmd.base.disable_model_invocation.unwrap_or(false),
            Command::LocalJsx(cmd) => cmd.base.disable_model_invocation.unwrap_or(false),
        }
    }
    
    /// 获取加载来源
    pub fn loaded_from(&self) -> Option<LoadedFrom> {
        match self {
            Command::Prompt(cmd) => cmd.base.loaded_from,
            Command::Local(cmd) => cmd.base.loaded_from,
            Command::LocalJsx(cmd) => cmd.base.loaded_from,
        }
    }
}

/// 命令结果展示方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandResultDisplay {
    /// 跳过
    Skip,
    /// 系统消息
    System,
    /// 用户消息
    User,
}

/// 命令执行上下文
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// 当前工作目录
    pub cwd: std::path::PathBuf,
    /// 配置
    pub config: crate::config::Config,
    /// 应用状态
    pub state: crate::state::AppState,
    /// 参数
    pub args: String,
}

/// 命令执行结果
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// 结果内容
    pub content: String,
    /// 展示方式
    pub display: CommandResultDisplay,
    /// 是否应该查询模型
    pub should_query: bool,
    /// 元消息
    pub meta_messages: Vec<String>,
    /// 下一个输入
    pub next_input: Option<String>,
    /// 是否提交下一个输入
    pub submit_next_input: bool,
}

impl Default for CommandResult {
    fn default() -> Self {
        Self {
            content: String::new(),
            display: CommandResultDisplay::User,
            should_query: false,
            meta_messages: Vec::new(),
            next_input: None,
            submit_next_input: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_serialization() {
        let cmd = Command::Prompt(PromptCommand {
            base: CommandBase {
                name: "test".to_string(),
                description: "Test command".to_string(),
                has_user_specified_description: None,
                aliases: None,
                availability: None,
                is_hidden: None,
                is_mcp: None,
                argument_hint: None,
                when_to_use: None,
                version: None,
                disable_model_invocation: None,
                user_invocable: None,
                loaded_from: None,
                kind: None,
                immediate: None,
                is_sensitive: None,
            },
            progress_message: "Testing...".to_string(),
            content_length: 100,
            arg_names: None,
            allowed_tools: None,
            model: None,
            source: CommandSource::Builtin,
            plugin_info: None,
            disable_non_interactive: None,
            context: None,
            agent: None,
            effort: None,
            paths: None,
        });
        
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"type\":\"prompt\""));
        assert!(json.contains("\"name\":\"test\""));
    }
    
    #[test]
    fn test_command_name() {
        let cmd = Command::Prompt(PromptCommand {
            base: CommandBase {
                name: "test".to_string(),
                description: "Test command".to_string(),
                has_user_specified_description: None,
                aliases: None,
                availability: None,
                is_hidden: None,
                is_mcp: None,
                argument_hint: None,
                when_to_use: None,
                version: None,
                disable_model_invocation: None,
                user_invocable: None,
                loaded_from: None,
                kind: None,
                immediate: None,
                is_sensitive: None,
            },
            progress_message: "Testing...".to_string(),
            content_length: 100,
            arg_names: None,
            allowed_tools: None,
            model: None,
            source: CommandSource::Builtin,
            plugin_info: None,
            disable_non_interactive: None,
            context: None,
            agent: None,
            effort: None,
            paths: None,
        });
        
        assert_eq!(cmd.name(), "test");
        assert_eq!(cmd.description(), "Test command");
    }
}
