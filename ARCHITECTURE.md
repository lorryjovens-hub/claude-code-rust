# Claude Code Rust - 架构设计文档

## 概述

本文档详细说明了 claude-code-rs 的系统架构设计，这是 Claude Code 的 Rust 重构版本。

## 设计原则

### 1. 类型安全
- 利用 Rust 的强类型系统
- 编译时错误检查
- 避免运行时意外

### 2. 并发安全
- 使用 `Arc<RwLock>` 进行线程安全的状态管理
- 异步 I/O 基于 Tokio
- 无数据竞争保证

### 3. 模块化设计
- 清晰的模块边界
- 最小化模块间依赖
- 易于测试和维护

### 4. 性能优先
- 零成本抽象
- 最小化内存分配
- 高效的数据结构

## 核心架构

### 分层架构

```
┌─────────────────────────────────────────────────────────┐
│                     CLI 入口层                          │
│  (main.rs - 命令行解析和调度)                          │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│                   命令层 (commands/)                    │
│  交互式 / 查询 / 配置 / 认证 / Bridge / MCP          │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│                   业务逻辑层                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ 工具系统     │  │ 多代理系统   │  │ 分析系统     │ │
│  │ (tools/)    │  │ (agents/)   │  │(analytics/)  │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ Bridge 系统  │  │ MCP 系统     │  │ 语音系统     │ │
│  │ (bridge/)   │  │ (mcp/)      │  │ (voice/)    │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│                   核心服务层                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ 状态管理     │  │ 配置系统     │  │ 错误处理     │ │
│  │ (state.rs)  │  │ (config.rs) │  │ (error.rs)   │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│                   基础设施层                              │
│  Tokio 运行时 / 网络 I/O / 文件系统 / 序列化         │
└─────────────────────────────────────────────────────────┘
```

## 核心模块详解

### 1. 配置系统 (config.rs)

#### 设计目标
- 多源配置合并
- 类型安全的配置访问
- 运行时配置更新

#### 关键类型

```rust
pub struct Config {
    pub project_root: PathBuf,
    pub cwd: PathBuf,
    pub api: ApiConfig,
    pub bridge: BridgeConfig,
    pub permissions: PermissionConfig,
    pub features: FeatureFlags,
    pub theme: ThemeConfig,
    // ...
}
```

#### 配置加载顺序
1. 全局配置 (`~/.config/claude-code/config.toml`)
2. 项目配置 (`.claude/settings.json`)
3. 环境变量（最高优先级）

### 2. 状态管理 (state.rs)

#### 设计模式
- **内部可变性**: 使用 `Arc<RwLock<AppStateInner>>`
- **线程安全**: 多线程环境下的安全访问
- **细粒度锁**: 读写锁分离，提高并发性能

#### 状态结构
```rust
pub struct AppState {
    inner: Arc<RwLock<AppStateInner>>,
}

pub struct AppStateInner {
    // 会话信息
    pub session_id: SessionId,
    pub parent_session_id: Option<SessionId>,
    
    // 统计信息
    pub total_cost_usd: f64,
    pub total_api_duration: i64,
    
    // 代理系统
    pub agent_color_map: HashMap<String, AgentColorName>,
    
    // ... 更多状态
}
```

### 3. 错误处理 (error.rs)

#### 错误类型设计
使用枚举统一所有错误类型：

```rust
pub enum ClaudeError {
    Config(String),
    Io(std::io::Error),
    Network(reqwest::Error),
    Serialization(serde_json::Error),
    Tool(String),
    Command(String),
    Auth(String),
    Permission(String),
    Bridge(String),
    Mcp(String),
    State(String),
    NotImplemented(String),
    Other(String),
}
```

#### 错误转换
实现 `From` trait 以便自动转换：

```rust
impl From<std::io::Error> for ClaudeError {
    fn from(err: std::io::Error) -> Self {
        ClaudeError::Io(err)
    }
}
```

### 4. 工具系统 (tools/)

#### Trait 设计
```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> ToolInputSchema;
    
    async fn validate_input(&self, input: &Value) -> Result<()> {
        Ok(())
    }
    
    async fn can_use(&self, context: &ToolUseContext) -> Result<PermissionResult> {
        Ok(PermissionResult::Allowed)
    }
    
    async fn execute(&self, input: Value, context: ToolUseContext) -> Result<ToolResult>;
}
```

#### 工具注册表
```rust
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool + Send + Sync>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register(FileReadTool::new());
        registry.register(FileEditTool::new());
        // ... 更多工具
        registry
    }
}
```

#### 内置工具
| 工具名 | 描述 | 权限级别 |
|--------|------|---------|
| file_read | 读取文件 | 只读 |
| file_edit | 编辑文件 | 标准 |
| file_write | 写入文件 | 标准 |
| bash | 执行命令 | 危险 |
| glob | 文件匹配 | 只读 |
| grep | 内容搜索 | 只读 |
| git | Git 命令 | 标准 |

### 5. 命令系统 (commands/)

#### Clap 驱动的 CLI
```rust
#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(alias = "i")]
    Interactive,
    
    #[command(alias = "q")]
    Query { query: Vec<String> },
    
    Config { key: Option<String>, value: Option<String> },
    
    Login,
    Logout,
    Version,
    Help,
    
    #[cfg(feature = "bridge")]
    Bridge,
    
    // ... 更多命令
}
```

#### 特性条件编译
使用 `#[cfg(feature = "...")]` 实现可选功能：

```rust
#[cfg(feature = "bridge")]
Some(Commands::Bridge) => {
    commands::bridge::run(config, state).await?;
}
```

### 6. 多代理系统 (agents/)

#### Agent Trait
```rust
#[async_trait]
pub trait Agent {
    fn name(&self) -> &str;
    fn agent_type(&self) -> AgentType;
    fn description(&self) -> &str;
    async fn run(&self, task: &str) -> Result<String>;
}
```

#### 代理类型
| 代理类型 | 用途 | 工具集 |
|---------|------|--------|
| GeneralPurpose | 通用任务 | 全工具 |
| Explore | 代码探索 | 只读工具 |
| Plan | 规划设计 | 分析工具 |
| Verification | 验证测试 | 测试工具 |
| Bash | 命令执行 | Bash + 文件 |

### 7. MCP 集成 (mcp/)

#### 核心概念
- **服务器**: 提供工具、资源和命令的外部服务
- **工具**: MCP 服务器提供的可调用函数
- **资源**: 可访问的数据（文件、数据库等）
- **命令**: 用户可执行的高级命令

#### 服务器管理
```rust
pub struct McpManager {
    servers: HashMap<String, McpServerInfo>,
    state: AppState,
}

impl McpManager {
    pub async fn list_servers(&self) -> Vec<McpServerInfo>;
    pub async fn enable_server(&mut self, name: String) -> Result<()>;
    pub async fn disable_server(&mut self, name: String) -> Result<()>;
    pub async fn reconnect_server(&mut self, name: String) -> Result<()>;
}
```

### 8. Bridge 系统 (bridge/)

#### 架构模式
```
┌──────────────┐        ┌──────────────┐
│  客户端      │◄──────►│  服务器      │
│  (Client)    │  网络   │  (Server)    │
└──────────────┘        └──────────────┘
```

#### 会话模式
- **SingleSession**: 单会话模式，在当前目录
- **Worktree**: Git 工作树，每个会话独立工作区
- **SameDir**: 共享目录，所有会话共用

## 数据流设计

### 命令执行流程

```
用户输入
   │
   ▼
┌──────────────────┐
│  CLI 解析器      │
│  (clap)         │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  配置加载        │
│  (ConfigLoader)  │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  状态初始化      │
│  (AppState)      │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  命令执行        │
│  (Command)       │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  工具调用        │
│  (ToolRegistry)  │
└────────┬─────────┘
         │
         ▼
    结果输出
```

### 工具调用流程

```
AI 模型决定调用工具
         │
         ▼
┌──────────────────────┐
│  工具查找            │
│  (ToolRegistry)      │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│  权限检查            │
│  (can_use)           │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│  输入验证            │
│  (validate_input)    │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│  工具执行            │
│  (execute)           │
└──────────┬───────────┘
           │
           ▼
    返回结果
```

## 并发模型

### Tokio 运行时
- **多线程调度器**: 默认使用多线程调度器
- **异步 I/O**: 所有 I/O 操作都是异步的
- **任务隔离**: 使用 `tokio::spawn` 隔离任务

### 状态并发访问
```rust
// 读操作
let state = app_state.read();
let session_id = &state.session_id;

// 写操作
let mut state = app_state.write();
state.total_cost_usd += 0.01;
```

## 性能优化策略

### 1. 内存优化
- 使用 `Arc` 共享数据，避免克隆
- 使用 `Cow<'a, str>` 避免不必要的分配
- 使用 `Vec::with_capacity` 预分配

### 2. 启动优化
- 条件编译减少二进制大小
- 懒加载重型模块
- 配置缓存

### 3. 运行时优化
- 对象池复用
- 连接池管理
- 批量操作

## 安全性设计

### 1. 权限系统
- 多层权限检查
- 白名单/黑名单规则
- 用户确认机制

### 2. 沙箱机制
- 命令执行隔离
- 文件系统访问限制
- 网络访问控制

### 3. 审计日志
- 所有工具调用记录
- 权限决策记录
- 文件操作记录

## 扩展性设计

### 1. 插件系统
- Trait 对象实现动态分发
- 运行时插件加载
- 插件间通信

### 2. 工具扩展
- 实现 `Tool` trait
- 注册到 `ToolRegistry`
- 自动发现和加载

### 3. 功能开关
- Cargo features 条件编译
- 运行时特性开关
- GrowthBook 集成

## 测试策略

### 1. 单元测试
- 每个模块独立测试
- Mock 外部依赖
- 覆盖所有公共 API

### 2. 集成测试
- 端到端流程测试
- 真实工具调用测试
- 多模块协作测试

### 3. 性能测试
- 基准测试
- 内存使用监控
- 并发压力测试

## 部署和分发

### 1. 编译目标
- Windows (x86_64)
- macOS (x86_64, aarch64)
- Linux (x86_64, aarch64)

### 2. 打包
- 单二进制文件
- 无外部依赖
- 跨平台兼容

## 未来扩展方向

### 1. 短期目标
- 完整的 AI 对话系统
- API 客户端集成
- 完善的错误处理

### 2. 中期目标
- 插件系统完整实现
- Web 界面
- 移动端支持

### 3. 长期目标
- 分布式执行
- 企业级功能
- 机器学习模型集成

---

*本文档将随着项目发展持续更新。*
