# 快速开始指南

欢迎！这个指南将帮助你快速开始使用 Claude Code Rust。

## 📋 目录

- [安装](#安装)
- [基本用法](#基本用法)
- [配置](#配置)
- [常见任务](#常见任务)
- [下一步](#下一步)

---

## 安装

### 选项 1: 自动化 CLI 安装 ⚡ **推荐**

使用我们的自动化脚本快速安装，支持所有主流操作系统。

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/lorryjovens-hub/claude-code-rust/master/install.ps1 | iex
```

**Linux / macOS:**
```bash
curl -sSL https://raw.githubusercontent.com/lorryjovens-hub/claude-code-rust/master/install-unix.sh | bash
```

### 选项 2: 直接下载二进制

从 [GitHub Releases 页面](https://github.com/lorryjovens-hub/claude-code-rust/releases) 下载预编译的二进制文件。

```bash
# 手动下载后，添加到 PATH
chmod +x claude-code  # Linux/macOS
./claude-code --version
```

### 选项 3: 从源代码编译

```bash
# 克隆仓库
git clone https://github.com/lorryjovens-hub/claude-code-rust.git
cd claude-code-rust

# 构建
cargo build --release

# 可执行文件位于: ./target/release/claude-code (Linux/macOS) 或 claude-code.exe (Windows)
./target/release/claude-code --version
```

### 选项 4: Docker

```bash
# 构建本地镜像
docker build -t claude-code-rust .

# 运行容器
docker run -it --rm claude-code-rust --version
docker run -it --rm claude-code-rust repl
```

### 验证安装

```bash
claude-code --version
# 输出: claude-code v0.1.0
```

---

## 基本用法

### 1. 第一次运行

```bash
# 显示帮助
claude-code --help

# 显示版本
claude-code --version
```

### 2. 单次查询

```bash
# 最简单的用法
claude-code "What is Rust?"

# 使用特定模型
claude-code --model claude-3-sonnet "Explain async/await"

# 使用 --prompt 标志
claude-code query --prompt "分析这个项目的结构"
```

### 3. REPL 交互模式

```bash
# 启动交互式 REPL
claude-code repl

# 在 REPL 中你可以输入多个问题
> What is machine learning?
> Explain neural networks
> .help     # 显示帮助
> .config   # 显示当前配置
> .exit     # 退出
```

### 4. 配置管理

```bash
# 查看当前配置
claude-code-rs config show

# 设置 API 密钥
claude-code-rs config set api_key "sk-ant-..."

# 设置默认模型
claude-code-rs config set model "claude-3-5-sonnet-20241022"

# 查看特定配置
claude-code-rs config get api_key

# 重置配置到默认值
claude-code-rs config reset
```

---

## 配置

### 快速配置

#### 1. 获取 API 密钥

访问 [Anthropic 控制台](https://console.anthropic.com/keys) 获取你的 API 密钥。

#### 2. 设置环境变量

```bash
# Linux/macOS
export CLAUDE_API_KEY="sk-ant-..."
export CLAUDE_MODEL="claude-3-5-sonnet-20241022"

# Windows (PowerShell)
$env:CLAUDE_API_KEY="sk-ant-..."
$env:CLAUDE_MODEL="claude-3-5-sonnet-20241022"
```

#### 3. 或创建配置文件

```bash
# 创建配置目录
mkdir -p ~/.config/claude-code-rust

# 创建配置文件
cat > ~/.config/claude-code-rust/config.toml << EOF
[api]
provider = "anthropic"
api_key = "sk-ant-..."
model = "claude-3-5-sonnet-20241022"

[settings]
theme = "dark"
language = "zh-CN"
EOF
```

### 完整配置选项

```toml
[api]
provider = "anthropic"              # API 提供者
api_key = "sk-ant-..."              # API 密钥
model = "claude-3-5-sonnet-20241022" # 模型名称
timeout = 30                        # 请求超时 (秒)
max_retries = 3                     # 重试次数

[terminal]
theme = "dark"                      # "dark" 或 "light"
language = "zh-CN"                  # 显示语言
enable_colors = true                # 彩色输出
enable_unicode = true               # Unicode 支持

[cache]
enabled = true                      # 启用缓存
ttl = 3600                          # 缓存 TTL (秒)
max_size = 1000                     # 最大缓存条目
```

---

## 常见任务

### 📝 代码分析

```bash
# 分析 Python 代码
claude-code-rs << EOF
请分析这个函数：

def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

有什么性能问题吗？
EOF
```

### 🐛 调试帮助

```bash
# 获取错误信息解释
claude-code-rs << EOF
我遇到这个错误：
TypeError: Cannot read property 'map' of undefined

这是什么意思？如何修复？
EOF
```

### 🚀 项目初始化

```bash
# 创建新项目
claude-code-rs init my-project
cd my-project

# 使用模板
claude-code-rs init --template web my-web-app
```

### 🔌 MCP 服务器

```bash
# 启动 MCP 服务器
claude-code-rs mcp start

# 列出可用工具
claude-code-rs mcp tools

# 执行工具
claude-code-rs mcp exec tool-name --args "..."
```

### 🧩 插件管理

```bash
# 列出插件
claude-code-rs plugin list

# 安装插件
claude-code-rs plugin install https://github.com/.../plugin

# 卸载插件
claude-code-rs plugin uninstall plugin-name

# 查看插件详情
claude-code-rs plugin info plugin-name
```

---

## 下一步

### 📚 更多资源

- [完整文档](./README.md)
- [性能基准](./PERFORMANCE_BENCHMARKS.md)
- [迁移指南](./MIGRATION_GUIDE.md) (从 TypeScript 版本)
- [API 参考](./docs/API.md)

### 🆘 获帮助

- GitHub Issues: [报告问题](../../issues)
- Discussions: [讨论和建议](../../discussions)
- 性能问题: 查看[性能基准](./PERFORMANCE_BENCHMARKS.md)

### 💡 学习更多

- Rust: https://www.rust-lang.org/learn
- Tokio: https://tokio.rs/
- Claude API: https://docs.anthropic.com

---

**祝你使用愉快！** ⚡
