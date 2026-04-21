# 贡献指南

感谢你对 Claude Code Rust 项目的兴趣！本文档指导如何为项目贡献代码。

## 🚀 快速开始

### 1. 设置开发环境

```bash
# 克隆仓库
git clone https://github.com/yourusername/claude-code-rust.git
cd claude-code-rust

# 安装 Rust (如果未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 构建项目
cargo build

# 运行测试
cargo test

# 检查代码质量
cargo clippy
```

### 2. 开发工作流

```bash
# 创建特性分支
git checkout -b feature/your-feature-name

# 进行更改并提交
git add .
git commit -m "feat: 描述你的改进"

# 推送分支
git push origin feature/your-feature-name

# 创建 Pull Request
# 在 GitHub 上提交 PR
```

## 📋 提交规范

我们遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<type>(<scope>): <subject>

<body>

<footer>
```

### 提交类型

- `feat`: 新功能
- `fix`: 错误修复
- `docs`: 文档更新
- `style`: 代码风格 (不改变功能)
- `refactor`: 代码重构
- `perf`: 性能优化
- `test`: 添加/修改测试
- `chore`: 项目管理

### 示例

```
feat(cli): 添加 config reset 命令

- 支持重置配置到默认值
- 添加确认提示
- 添加 --force 标志

Closes #123
```

## ✅ 审查标准

在提交 PR 前，请确保：

### 代码质量
- [ ] 运行 `cargo clippy` 无错误
- [ ] 运行 `cargo fmt` 格式化代码
- [ ] 添加/更新相关测试
- [ ] 运行 `cargo test` 所有测试通过

### 文档
- [ ] 更新相关文档
- [ ] 添加代码注释 (复杂逻辑)
- [ ] 更新 CHANGELOG.md

### 性能
- [ ] 未显著增加启动时间
- [ ] 未显著增加内存占用
- [ ] 已优化关键路径

## 🏆 最佳实践

### 代码风格

```rust
// ✅ 好的例子
pub async fn query(client: &Client, q: &str) -> Result<Response> {
    // 完整的错误处理
    client
        .post("/query")
        .json(&QueryRequest { query: q })
        .send()
        .await
        .context("Failed to send query")?
        .json()
        .await
        .context("Failed to parse response")
}

// ❌ 避免
pub async fn query(client: &Client, q: &str) -> Result<Response> {
    let resp = client.post("/query").json(&QueryRequest { query: q }).send().await?;
    Ok(resp.json().await?)  // 信息不足的错误处理
}
```

### 错误处理

```rust
// ✅ 使用 anyhow + thiserror
use thiserror::Error;
use anyhow::{Context, Result};

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid configuration: {0}")]
    Invalid(String),
    
    #[error("Configuration file not found: {0}")]
    NotFound(String),
}

// ❌ 有时候对
pub fn load_config(path: &str) -> Result<Config> {
    std::fs::read_to_string(path)
        .context("Failed to read config file")?
        .parse()
        .context("Failed to parse config file")
}
```

### 异步编程

```rust
// ✅ 使用 Tokio
#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::new();
    client.query("test").await?;
    Ok(())
}

// ✅ 批量操作使用 join_all
let futures: Vec<_> = urls
    .iter()
    .map(|url| fetch(url))
    .collect();

let results = futures::future::join_all(futures).await;
```

## 📊 性能期望

新增代码应该：

- **启动速度**: 不增将启动时间超过 5%
- **内存占用**: 不增加基础内存超过 2%
- **二进制大小**: 不增加体积超过 500KB

### 性能测试

```bash
# 构建 release 版本
cargo build --release

# 测试启动速度
time ./target/release/claude_code_rs --version

# 测试内存占用
/usr/bin/time -v ./target/release/claude_code_rs --help

# 检查二进制大小
ls -lh ./target/release/claude_code_rs
```

## 🐛 报告 Bug

在提交 Issue 时：

1. **检查已有 Issue** - 避免重复
2. **提供复现步骤** - 清晰的步骤说明
3. **提供环境信息** - OS, Rust 版本等
4. **提供日志输出** - RUST_LOG=debug 的输出

### Bug 报告模板

```markdown
## 描述
简明描述 bug

## 复现步骤
1. ...
2. ...
3. ...

## 预期行为
应该发生什么

## 实际行为
实际发生了什么

## 环境
- OS: [e.g. Windows 10, Ubuntu 22.04]
- Rust: 1.75+ (cargo --version)
- 其他相关信息

## 日志
\`\`\`
RUST_LOG=debug cargo run ...
<输出>
\`\`\`
```

## 💡 功能建议

良好的功能建议应该包括：

- [ ] 明确的用例说明
- [ ] 与现有功能的关系
- [ ] 可能的实现方向 (可选)
- [ ] 性能/安全性考虑 (如适用)

## 📚 资源

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Tokio 文档](https://tokio.rs/)
- 项目代码注释和文档

## ❓ 需要帮助？

- 提交 Discussion - 提问和讨论
- 加入 Discord/Slack - (待添加)
- 阅读现有 PR - 学习如何做好贡献

---

感谢你的贡献！🙌
