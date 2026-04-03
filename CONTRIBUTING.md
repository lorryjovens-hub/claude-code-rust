# 贡献指南

感谢您有兴趣为 Claude Code Rust 项目做出贡献！我们欢迎所有形式的贡献，包括代码、文档、测试和反馈。

## 如何开始

### 1. Fork 仓库

首先，点击 GitHub 页面右上角的 "Fork" 按钮，将仓库 fork 到您的个人账户。

### 2. 克隆您的 Fork

```bash
git clone https://github.com/YOUR_USERNAME/claude-code-rust.git
cd claude-code-rust
```

### 3. 添加上游仓库

```bash
git remote add upstream https://github.com/lorryjovens-hub/claude-code-rust.git
```

### 4. 创建分支

```bash
git checkout -b feature/your-feature-name
```

## 开发环境设置

### 前置要求

- Rust (最新稳定版)
- Node.js (v16+)
- Git

### 安装依赖

```bash
# 安装 Rust 依赖
cargo build

# 安装 GUI 客户端依赖
cd gui-client
npm install
```

### 运行测试

```bash
# 运行 Rust 测试
cargo test

# 运行 GUI 客户端测试
cd gui-client
npm test
```

## 贡献流程

### 1. 查找或创建 Issue

- 查看现有的 [Issues](https://github.com/lorryjovens-hub/claude-code-rust/issues)
- 如果您想处理某个问题，请在 Issue 下留言
- 如果没有合适的 Issue，可以创建一个新的

### 2. 编写代码

- 遵循现有的代码风格
- 添加适当的注释
- 确保代码通过所有测试

### 3. 提交更改

```bash
git add .
git commit -m "feat: 添加新功能描述"
git push origin feature/your-feature-name
```

### 4. 创建 Pull Request

- 访问您的 fork 页面
- 点击 "Compare & pull request"
- 填写 PR 描述，包括：
  - 解决了什么问题
  - 如何测试
  - 相关的 Issue 编号

## 代码规范

### Rust 代码

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码
- 遵循 [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

### TypeScript/JavaScript 代码

- 使用 `npm run lint` 检查代码
- 遵循项目中的 ESLint 配置

### 提交信息规范

我们使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

- `feat:` 新功能
- `fix:` 修复 bug
- `docs:` 文档更新
- `style:` 代码格式（不影响代码运行的变动）
- `refactor:` 重构
- `test:` 测试相关
- `chore:` 构建过程或辅助工具的变动

## 项目结构

```
claude-code-rust/
├── src/                    # Rust 核心代码
│   ├── gui/               # GUI 模块
│   ├── i18n/              # 国际化
│   ├── wasm/              # WebAssembly 支持
│   └── web/               # Web 服务器
├── gui-client/            # GUI 客户端 (React + Tauri)
├── locales/               # 翻译文件
└── docs/                  # 文档
```

## 功能模块

### 当前功能

- ✅ CLI 版本
- ✅ GUI 版本 (egui)
- ✅ WebAssembly 支持
- ✅ 多语言支持
- ✅ 插件系统
- ✅ 插件市场 Web 界面

### 计划中功能

- 🔄 PI 流式响应优化
- 🔄 完整的 API 集成测试
- 🔄 更多 GUI 改进

## 需要帮助？

如果您在贡献过程中遇到任何问题：

1. 查看 [Issues](https://github.com/lorryjovens-hub/claude-code-rust/issues) 是否有类似问题
2. 创建新的 Issue 描述您的问题
3. 加入我们的讨论社区

## 行为准则

- 尊重所有贡献者
- 保持专业和友善
- 接受建设性的批评
- 关注什么是最好的社区和项目

## 许可证

通过贡献代码，您同意您的贡献将在 MIT 许可证下发布。

---

再次感谢您的贡献！🎉
