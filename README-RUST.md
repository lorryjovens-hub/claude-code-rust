# Claude Code - Rust Implementation

这是 Claude Code 项目的 Rust 重构版本，提供与原 TypeScript 版本相同的 AI 辅助编程能力，同时带来 Rust 语言的性能优势和类型安全保障。

## 项目概述

claude-code-rs 是一个完整的 AI 编程助手，具有以下特性：

- **高性能**: Rust 语言带来的原生性能优势
- **类型安全**: 编译时类型检查，减少运行时错误
- **内存安全**: 无垃圾回收，内存使用更高效
- **跨平台**: 支持 Windows、macOS 和 Linux
- **完整功能**: 与原 TypeScript 版本保持功能完整性

## 架构设计

### 项目结构

```
claude-code-rust/
├── Cargo.toml              # Rust 包配置
├── src/
│   ├── main.rs            # 主入口文件
│   ├── lib.rs             # 库文件
│   ├── config.rs          # 配置系统
│   ├── error.rs           # 错误处理
│   ├── state.rs           # 状态管理
│   ├── commands/          # 命令实现
│   │   ├── mod.rs
│   │   ├── interactive.rs
│   │   ├── query.rs
│   │   ├── config.rs
│   │   └── auth.rs
│   ├── tools/             # 工具系统
│   │   ├── mod.rs
│   │   ├── file.rs        # 文件操作工具
│   │   ├── bash.rs        # Bash 命令工具
│   │   ├── search.rs      # 搜索工具 (Glob/Grep)
│   │   └── git.rs         # Git 工具
│   ├── mcp/               # MCP 协议集成
│   │   └── mod.rs
│   └── utils/             # 工具函数
│       └── mod.rs
└── README-RUST.md         # 本文档
```

### 核心模块

#### 1. 配置系统 (config.rs)

提供完整的配置管理功能，包括：
- 宏配置 (`MacroConfig`)
- 主配置 (`Config`)
- API 配置 (`ApiConfig`)
- Bridge 配置 (`BridgeConfig`)
- 权限配置 (`PermissionConfig`)
- 特性开关 (`FeatureFlags`)
- 主题配置 (`ThemeConfig`)

配置加载顺序：
1. 全局配置 (`~/.config/claude-code/config.toml`)
2. 项目配置 (`.claude/settings.json`)
3. 环境变量

#### 2. 错误处理 (error.rs)

统一的错误类型 `ClaudeError`，包含：
- 配置错误
- IO 错误
- 网络错误
- 序列化错误
- 工具错误
- 命令错误
- 认证错误
- 权限错误
- Bridge 错误
- MCP 错误

#### 3. 状态管理 (state.rs)

线程安全的应用状态管理：
- `AppState`: 外层包装，使用 `Arc<RwLock>` 提供并发安全
- `AppStateInner`: 内部状态，包含完整的应用状态数据
- 会话 ID、任务 ID、模型使用统计等

#### 4. 命令系统 (commands/)

基于 Clap 的命令行接口：
- `interactive`: 交互模式
- `query`: 单次查询
- `config`: 配置管理
- `login/logout`: 认证
- `bridge`: Bridge 模式（可选功能）
- `mcp`: MCP 管理（可选功能）
- `voice`: 语音模式（可选功能）
- `daemon`: 守护进程（可选功能）

#### 5. 工具系统 (tools/)

可扩展的工具架构：
- `Tool` trait: 所有工具必须实现的 trait
- `ToolRegistry`: 工具注册和管理
- 内置工具：
  - `file_read`: 读取文件
  - `file_edit`: 编辑文件
  - `file_write`: 写入文件
  - `bash`: 执行 Bash 命令
  - `glob`: 文件模式匹配
  - `grep`: 内容搜索
  - `git`: Git 命令

#### 6. MCP 集成 (mcp/)

Model Context Protocol 集成：
- 服务器管理
- 工具发现
- 资源访问
- 命令执行

## 功能特性

### 已实现

- ✅ 项目框架和配置系统
- ✅ CLI 入口和基础命令结构
- ✅ 状态管理系统
- ✅ 错误处理架构
- ✅ 核心工具系统 (FileRead/FileEdit/FileWrite/Bash/Glob/Grep/Git)
- ✅ 命令注册表
- ✅ 工具注册表

### 进行中

- 🔄 交互模式 REPL
- 🔄 API 客户端集成
- 🔄 MCP 协议完整实现
- 🔄 Bridge 远程控制

### 计划中

- ⏳ 完整的 AI 对话系统
- ⏳ 语音交互
- ⏳ 多代理系统
- ⏳ 插件系统
- ⏳ 自动记忆整合
- ⏳ 上下文压缩

## 安装和使用

### 前置要求

- Rust 1.70 或更高版本
- Cargo 包管理器

### 编译

```bash
# 克隆项目
cd claude-code-rust

# 编译（debug 模式）
cargo build

# 编译（release 模式）
cargo build --release
```

### 运行

```bash
# 显示帮助
cargo run -- --help

# 显示版本
cargo run -- version

# 交互模式
cargo run -- interactive
```

### 可选功能

项目支持多个可选功能，可以通过 Cargo features 启用：

```bash
# 启用所有功能
cargo build --release --features "full"

# 启用 Bridge 模式
cargo build --release --features "bridge"

# 启用 MCP 支持
cargo build --release --features "mcp-support"

# 启用语音模式
cargo build --release --features "voice"

# 启用守护进程
cargo build --release --features "daemon"

# 启用实验性功能
cargo build --release --features "experimental"
```

## 开发指南

### 代码规范

项目遵循 Rust 官方代码规范：
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码
- 所有公共 API 必须有文档注释

### 添加新工具

1. 在 `src/tools/` 中创建新的工具模块
2. 实现 `Tool` trait
3. 在 `src/tools/mod.rs` 的 `Default` 实现中注册工具

示例：

```rust
// src/tools/my_tool.rs
use async_trait::async_trait;
use crate::tools::{Tool, ToolInputSchema, ToolResult, ToolUseContext};
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str {
        "my_tool"
    }
    
    fn description(&self) -> &str {
        "My custom tool description"
    }
    
    fn input_schema(&self) -> ToolInputSchema {
        // 实现输入 schema
        ToolInputSchema::default()
    }
    
    async fn execute(&self, input: serde_json::Value, context: ToolUseContext) -> Result<ToolResult> {
        // 实现工具逻辑
        Ok(ToolResult::success("Done!".to_string()))
    }
}
```

### 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test <test_name>

# 显示测试输出
cargo test -- --nocapture
```

## 性能优势

相比原 TypeScript 版本，Rust 版本具有以下优势：

1. **启动速度**: 无需 Node.js 启动开销
2. **内存使用**: 更高效的内存管理
3. **执行性能**: 原生代码执行速度
4. **并发能力**: 更好的多线程支持
5. **二进制大小**: 单一可执行文件，无需依赖

## 项目对比

| 特性 | TypeScript 版本 | Rust 版本 |
|------|---------------|-----------|
| 语言 | TypeScript | Rust |
| 运行时 | Node.js | 原生 |
| 类型安全 | 运行时 | 编译时 |
| 内存管理 | GC | 所有权系统 |
| 启动速度 | 较慢 | 快速 |
| 并发模型 | 单线程异步 | 多线程异步 |
| 打包 | npm 包 | 单二进制 |

## 贡献指南

我们欢迎任何形式的贡献！

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 开启 Pull Request

## 许可证

本项目采用与原 Claude Code 项目相同的许可证。

## 致谢

感谢 Anthropic 团队创建了优秀的 Claude Code 项目，本 Rust 版本是对其的致敬和重构。

---

**注意**: 这是一个正在进行中的项目，部分功能尚未完全实现。欢迎参与开发！
