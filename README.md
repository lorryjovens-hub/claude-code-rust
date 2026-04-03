# Claude Code Rust

[![CI](https://github.com/lorryjovens-hub/claude-code-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/lorryjovens-hub/claude-code-rust/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

🚀 高性能的 Claude Code CLI Rust 实现，支持 WebAssembly、GUI 和多语言。

[English](README_EN.md) | 简体中文

## ✨ 特性

- **⚡ 高性能** - 使用 Rust 编写，内存安全且性能卓越
- **🖥️ 多平台支持** - CLI、GUI (egui/Tauri)、WebAssembly
- **🌐 多语言** - 支持 10+ 种语言（中文、英文、日文、韩文等）
- **🔌 插件系统** - 可扩展的插件架构
- **🛒 插件市场** - Web 界面的插件市场
- **🎨 现代化 GUI** - 基于 CC Switch 的美观界面
- **🔒 安全可靠** - 类型安全，内存安全

## 🚀 快速开始

### 安装

```bash
# 使用 cargo 安装
cargo install claude-code-rs

# 或者从源码构建
git clone https://github.com/lorryjovens-hub/claude-code-rust.git
cd claude-code-rust
cargo build --release
```

### CLI 使用

```bash
# 启动交互式会话
claude-code

# 执行单个命令
claude-code --command "解释这段代码"

# 使用特定模型
claude-code --model claude-3-opus-20240229
```

### GUI 使用

```bash
# 启动 GUI 版本
cargo run --features gui-egui

# 或者启动 Tauri 版本
cd gui-client
npm install
npm run dev
```

## 🏗️ 项目结构

```
claude-code-rust/
├── src/                    # Rust 核心代码
│   ├── gui/               # GUI 模块 (egui)
│   ├── i18n/              # 国际化支持
│   ├── wasm/              # WebAssembly 支持
│   └── web/               # Web 服务器（插件市场）
├── gui-client/            # Tauri GUI 客户端
├── locales/               # 翻译文件
└── docs/                  # 文档
```

## 🛠️ 开发

### 环境要求

- Rust 1.75+
- Node.js 18+ (GUI 开发)
- Git

### 构建

```bash
# 构建 CLI 版本
cargo build --release

# 构建 GUI 版本
cargo build --release --features gui-egui

# 构建 WebAssembly 版本
cargo build --release --features wasm --target wasm32-unknown-unknown

# 构建 Web 服务器
cargo build --release --features web
```

### 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test --package claude_code_rs
```

### 代码质量

```bash
# 格式化代码
cargo fmt

# 运行 Clippy
cargo clippy -- -D warnings

# GUI 客户端
cd gui-client
npm run lint
```

## 🌍 国际化

项目支持以下语言：

- 🇨🇳 简体中文
- 🇺🇸 English
- 🇯🇵 日本語
- 🇰🇷 한국어
- 🇫🇷 Français
- 🇩🇪 Deutsch
- 🇪🇸 Español
- 🇷🇺 Русский
- 🇮🇹 Italiano
- 🇧🇷 Português

## 🤝 贡献

我们欢迎所有形式的贡献！请查看我们的[贡献指南](CONTRIBUTING.md)了解如何参与。

### 快速贡献步骤

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

## 📋 路线图

### 已完成 ✅

- [x] CLI 版本
- [x] GUI 版本 (egui)
- [x] WebAssembly 支持
- [x] 多语言支持
- [x] 插件系统
- [x] 插件市场 Web 界面
- [x] Tauri GUI 客户端

### 进行中 🔄

- [ ] PI 流式响应优化
- [ ] 完整的 API 集成测试
- [ ] 更多 GUI 改进
- [ ] 性能优化

### 计划中 📋

- [ ] VS Code 扩展
- [ ] 更多 AI 提供商支持
- [ ] 团队协作功能
- [ ] 云端同步

## 📄 许可证

本项目采用 [MIT](LICENSE) 许可证。

## 🙏 致谢

- [Anthropic](https://www.anthropic.com/) - 提供 Claude API
- [CC Switch](https://github.com/cc-switch/cc-switch) - GUI 设计灵感
- [Tauri](https://tauri.app/) - 跨平台 GUI 框架
- [egui](https://github.com/emilk/egui) - 即时模式 GUI 库

## 📞 联系我们

- GitHub Issues: [提交问题](https://github.com/lorryjovens-hub/claude-code-rust/issues)
- Discussions: [参与讨论](https://github