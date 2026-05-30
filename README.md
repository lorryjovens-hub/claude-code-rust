# 🚀 Claude Desktop — Tauri AI 操作系统

> **完整的 AI 桌面客户端** | CLI + GUI 双引擎驱动 | 编程 · 文件 · 编译 · 打包一站式

<p align="center">
  <img src="https://img.shields.io/badge/Tauri-2.0-FFC131?style=for-the-badge&logo=tauri&logoColor=white" alt="Tauri 2.0">
  <img src="https://img.shields.io/badge/React-19-61DAFB?style=for-the-badge&logo=react&logoColor=black" alt="React 19">
  <img src="https://img.shields.io/badge/Rust-2024-CE422B?style=for-the-badge&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/TypeScript-5.7-3178C6?style=for-the-badge&logo=typescript&logoColor=white" alt="TypeScript">
  <br>
  <img src="https://img.shields.io/badge/License-MIT-green?style=for-the-badge" alt="License MIT">
  <img src="https://img.shields.io/badge/Platform-Win%20%7C%20macOS%20%7C%20Linux-blue?style=for-the-badge" alt="Platform">
  <img src="https://img.shields.io/badge/Version-3.1.0-orange?style=for-the-badge" alt="Version">
</p>

---

## 📸 产品截图

<div align="center">
  <p><strong>🖥️ 三栏工作台 — 文件树 · Monaco 编辑器 · 实时预览</strong></p>
  <img src="https://private-user-images.githubusercontent.com/229292403/582982314-2748c7a9-4269-4952-8af9-9e1cacc735fb.png" alt="三栏工作台" width="90%">
  <p><strong>💻 多标签终端 — ANSI 颜色 · 右键菜单</strong></p>
  <img src="https://private-user-images.githubusercontent.com/229292403/582982307-f32fe710-1be8-4ac7-a99c-3181bcdb3340.png" alt="多标签终端" width="90%">
  <p><strong>🔀 Git 可视化 — Diff 查看 · Commit 界面</strong></p>
  <img src="https://private-user-images.githubusercontent.com/229292403/582982305-8cc2c093-2dbd-4188-9720-1f9d5111c4ed.png" alt="Git 可视化" width="90%">
  <p><strong>⚡ 编译打包 — 自动检测项目类型</strong></p>
  <img src="https://private-user-images.githubusercontent.com/229292403/582982302-a26e5676-6020-4c05-8186-af66a9eaaa85.png" alt="编译打包" width="90%">
  <p><strong>🎨 实时预览 — HTML/JSX/WebGPU 渲染</strong></p>
  <img src="https://private-user-images.githubusercontent.com/229292403/582982292-7f5c4321-f433-4f0b-a3a8-91b290c1ba3e.png" alt="实时预览" width="90%">
  <p><strong>🎙️ 语音输入 — 语音转文字交互</strong></p>
  <img src="https://private-user-images.githubusercontent.com/229292403/582982279-67152893-e093-4011-ba75-a84eb3d9a023.png" alt="语音输入" width="90%">
</div>

---

## 🌟 一句话介绍

**Claude Desktop** 是基于 **Tauri 2.0 + Rust + React** 构建的完整 AI 桌面客户端，集成 CLI 命令行工具和 GUI 图形界面双引擎。它不仅是一个聊天工具，更是一台 **顶级 AI 操作系统** —— 支持文件管理、代码编辑、终端模拟、Git 可视化、编译打包、语音输入等复杂电脑操作能力。

---

## 🎯 核心亮点

| 维度 | CLI 版本 | Tauri GUI 版本 |
|------|----------|----------------|
| **定位** | 命令行工具链 | 完整桌面操作系统 |
| **性能** | ⚡ 启动 63ms，体积仅 5MB | 🖥️ 原生窗口，丝滑体验 |
| **交互** | 终端 REPL 交互 | 三栏可视化拖拽布局 |
| **文件管理** | 文件系统命令 | 文件树 + Monaco 编辑器 |
| **终端** | 原生终端 | 多标签终端模拟 |
| **Git** | 命令行操作 | 可视化 Diff + Commit 界面 |
| **编译** | cargo build | 自动检测项目类型 + 一键构建 |
| **预览** | 无 | HTML/JSX 实时渲染 |
| **语音** | CLI 语音模式 | 模态弹窗 + 波形动画 |
| **存储** | 本地文件 | SQLite 持久化数据库 |

---

## ✨ 功能全景

### 📁 文件与工作区
- **文件树浏览器** — 浏览任意本地目录，显示文件类型图标
- **Git 状态标记** — 实时显示文件 Git 状态 (M/A/D/U)
- **右键菜单** — 新建文件/文件夹、删除、重命名、复制路径
- **工作区管理** — 记住最近打开的文件夹，快速切换
- **文件下载** — 一键下载 AI 生成的文件
- **系统打开** — 在默认应用中打开文件

### 💻 VS Code 同款编辑器
- **Monaco Editor** — 完整的代码编辑器，支持 100+ 语言语法高亮
- **代码折叠 / 展开** — 快速浏览大文件
- **多标签切换** — 同时打开多个文件
- **行号显示** — 精确定位代码行
- **保存 / 编辑** — 实时保存修改

### 🖥️ 多标签终端
- **xterm.js 终端模拟** — 完整 ANSI 颜色转义序列
- **多标签支持** — 同时运行多个终端会话
- **右键菜单** — 复制 / 粘贴 / 清除
- **ANSI 颜色渲染** — 彩色输出，一目了然

### ⚡ 编译打包流水线
- **自动检测项目类型** — npm / cargo / gradle / make / pip 等
- **一键编译构建** — 自动执行对应构建命令
- **实时输出日志** — 彩色编译输出
- **产物预览** — 构建产物可视化展示

### 🔀 Git 可视化
- **文件树 Git 状态** — 文件图标标记变更状态
- **可视化 Diff** — 代码差异对比，行级高亮
- **Commit 界面** — 选择文件、编写提交信息、一键提交
- **Git Bash 检测** — 自动检测系统 Git Bash

### 🤖 AI 核心能力
- **多模型支持** — Anthropic Claude / OpenAI GPT / DeepSeek / 其他兼容 API
- **流式响应** — SSE 实时逐字输出，零等待体验
- **语音输入** — 语音转文字，模态弹窗 + 波形动画
- **图片上传** — 发送截图、拍照图片给 AI 分析
- **文档上传** — 支持 PDF / DOCX / PPTX / XLSX 上传预览
- **Markdown 渲染** — KaTeX 数学公式 + Mermaid 图表 + 代码高亮

### 💾 数据持久化
- **SQLite 数据库** — 本地对话历史永久保存
- **模型使用记录** — 跟踪每次 API 调用消耗
- **成本追踪** — 实时统计 Token 消耗和费用
- **对话搜索** — 快速查找历史对话

### 🔧 高级功能
- **MCP 集成** — Model Context Protocol 外部工具调用
- **多 Agent 协作** — 协调多个 AI 代理并行处理
- **技能系统** — 用户可扩展的插件模块
- **斜杠命令** — `/compact` `/clear` 等快速操作
- **权限管理** — 每次询问 / 自动接受 / 仅计划 / 完全绕过
- **自动更新** — 检测新版本并提示升级
- **国际化** — 中文 / 英文多语言支持

### 🖥️ 桌面集成
- 系统托盘图标
- 原生文件对话框
- 剪贴板集成
- 桌面通知
- 拖拽调整布局
- 响应式设计

---

## 🏗️ 架构设计

### 双引擎架构

```
┌─────────────────────────────────────────────────────────┐
│                    Claude Desktop                       │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌────────────────────┐    ┌────────────────────────┐  │
│  │    CLI 引擎         │    │    GUI 引擎 (Tauri)     │  │
│  │                    │    │                        │  │
│  │  • 命令行交互       │    │  • React 19 前端       │  │
│  │  • REPL 模式       │    │  • 三栏可视化布局       │  │
│  │  • 单次查询执行    │    │  • Monaco 编辑器        │  │
│  │  • MCP 服务器      │    │  • xterm.js 终端        │  │
│  │  • 5MB 单文件      │    │  • SQLite 数据库        │  │
│  └────────┬───────────┘    └───────────┬────────────┘  │
│           │                            │               │
│           └────────────┬───────────────┘               │
│                        │                               │
│              ┌─────────▼─────────┐                     │
│              │  Rust 后端核心     │                     │
│              │                   │                     │
│              │  • Axum HTTP 桥接  │                     │
│              │  • Tokio 异步运行时│                     │
│              │  • rusqlite 数据库 │                     │
│              │  • reqwest API调用 │                     │
│              │  • notify 文件监听 │                     │
│              │  • portable-pty    │                     │
│              └───────────────────┘                     │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### CLI 版本架构

```
claude-code-rust/
├── src/
│   ├── cli/              # CLI 命令解析 (clap)
│   ├── api/              # API 客户端 (reqwest)
│   ├── mcp/              # MCP 协议实现
│   ├── memory/           # 内存/会话管理
│   ├── plugins/          # 插件系统
│   ├── config/           # 配置管理
│   └── tools/            # 工具实现
├── Cargo.toml            # Rust 依赖
└── scripts/              # 安装脚本
```

### Tauri GUI 版本架构

```
claude-desktop-tauri/
├── src/                          # React 前端源码
│   ├── components/               # UI 组件 (~80+ 文件)
│   │   ├── MonacoEditor.tsx      # VS Code 编辑器
│   │   ├── TerminalPanel.tsx     # 多标签终端
│   │   ├── FileExplorer.tsx      # 文件树浏览器
│   │   ├── ResizableLayout.tsx   # 可拖拽布局
│   │   ├── BuildPanel.tsx        # 编译流水线
│   │   ├── GitDiffView.tsx       # Git 可视化 Diff
│   │   ├── GitCommitPanel.tsx    # Git Commit 界面
│   │   ├── VoiceInput.tsx        # 语音输入
│   │   ├── ChatMainContent.tsx   # 聊天主界面
│   │   └── ...
│   ├── features/                 # 功能模块
│   ├── stores/                   # Zustand 状态管理
│   ├── api/                      # API 客户端
│   └── hooks/                    # React 自定义 Hooks
│
├── src-tauri/                    # Rust 后端源码
│   ├── src/
│   │   ├── bridge/               # Axum HTTP API 服务
│   │   ├── db/                   # SQLite 数据库
│   │   ├── native_engine/        # AI 引擎核心
│   │   ├── mcp/                  # MCP 客户端/服务端
│   │   ├── memory/               # 对话记忆系统
│   │   ├── terminal/             # PTY 终端
│   │   ├── git/                  # Git 集成
│   │   ├── tools/                # 工具实现
│   │   └── ...
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── package.json                  # Node.js 依赖
├── vite.config.ts                # Vite 配置
└── tailwind.config.js            # Tailwind 配置
```

---

## 🚀 快速开始

### 系统要求

| 组件 | 版本 |
|------|------|
| **Rust** | 1.75+ ([rustup](https://rustup.rs)) |
| **Node.js** | 18+ ([nvm](https://github.com/nvm-sh/nvm)) |
| **Windows** | Visual Studio C++ Build Tools, WebView2 |
| **macOS** | Xcode Command Line Tools |
| **Linux** | `libwebkit2gtk-4.1-dev`, `libgtk-3-dev` |

### CLI 版本

```bash
# 克隆仓库
git clone https://github.com/lorryjovens-hub/claude-code-rust.git
cd claude-code-rust

# 编译发布版本
cargo build --release

# 运行
./target/release/claude-code --version
```

### Tauri GUI 版本

```bash
# 克隆仓库
git clone https://github.com/lorryjovens-hub/claude-code-rust.git
cd claude-code-rust
cd claude-desktop-tauri

# 安装依赖
npm install

# 开发模式（热重载）
npm run tauri dev

# 构建生产版本
npm run tauri build
```

### 配置 API

```bash
# 方式 1: 环境变量
export ANTHROPIC_API_KEY="your-api-key"
export API_BASE_URL="https://api.anthropic.com"

# 方式 2: .env 文件
ANTHROPIC_API_KEY=your-api-key
API_BASE_URL=https://api.anthropic.com
```

---

## 📊 功能对比矩阵

| 功能 | CLI 版本 | Tauri GUI 版本 |
|------|:--------:|:--------------:|
| 多模型聊天 | ✅ | ✅ |
| 流式响应 | ✅ | ✅ |
| 语音输入 | ✅ | ✅ 波形动画 |
| MCP 集成 | ✅ | ✅ |
| 插件系统 | ✅ | ✅ 技能系统 |
| 终端交互 | ✅ 原生 | ✅ 多标签 + ANSI |
| 文件浏览 | ❌ | ✅ 文件树 + 图标 |
| 代码编辑 | ❌ | ✅ Monaco 编辑器 |
| 实时预览 | ❌ | ✅ HTML/JSX |
| Git 可视化 | ❌ | ✅ Diff + Commit |
| 编译打包 | ❌ | ✅ 自动检测 |
| 图片上传 | ❌ | ✅ 附件 |
| 文档预览 | ❌ | ✅ PDF/DOCX/PPTX |
| SQLite 历史 | ❌ | ✅ |
| 成本追踪 | ❌ | ✅ |
| 权限管理 | ❌ | ✅ |
| 布局调整 | ❌ | ✅ 拖拽 |
| 自动更新 | ❌ | ✅ |
| 安装包大小 | **5MB** | ~50MB |

---

## 🔧 技术栈

### 前端 (Tauri GUI)

| 技术 | 用途 |
|------|------|
| React 19 | UI 组件框架 |
| TypeScript 5.7 | 类型安全 |
| Vite 6 | 构建工具 |
| Tailwind CSS 3 | 原子化样式 |
| Zustand 5 | 轻量状态管理 |
| Monaco Editor | 代码编辑器 |
| xterm.js 5 | 终端模拟 |
| KaTeX | 数学渲染 |
| Mermaid | 图表渲染 |
| recharts | 数据可视化 |
| framer-motion | 动画 |
| @dnd-kit | 拖拽排序 |

### 后端 (Rust)

| 技术 | 用途 |
|------|------|
| Tauri 2.0 | 桌面框架 |
| Axum 0.8 | HTTP 桥接 |
| Tokio 1 | 异步运行时 |
| rusqlite 0.31 | SQLite 数据库 |
| reqwest 0.12 | HTTP 客户端 |
| portable-pty 0.8 | PTY 终端 |
| notify 6 | 文件监听 |
| enigo 0.2 | 输入模拟 |
| diffy 0.4 | 文本差异 |
| dashmap 6 | 并发 HashMap |
| prometheus 0.13 | 指标暴露 |

---

## 📁 项目结构总览

```
.
├── claude-code-rust/              # CLI 版本（Rust 命令行工具）
│   ├── src/
│   │   ├── cli/                   # 命令行解析
│   │   ├── api/                   # API 客户端
│   │   ├── mcp/                   # MCP 协议
│   │   ├── memory/                # 会话管理
│   │   ├── plugins/               # 插件系统
│   │   └── tools/                 # 工具实现
│   ├── Cargo.toml
│   └── scripts/                   # 安装脚本
│
├── claude-desktop-tauri/          # Tauri GUI 版本（桌面客户端）
│   ├── src/                       # React 前端
│   │   ├── components/            # 80+ UI 组件
│   │   ├── features/              # 功能模块
│   │   ├── stores/                # Zustand 状态
│   │   ├── api/                   # API 客户端
│   │   └── hooks/                 # React Hooks
│   ├── src-tauri/                 # Rust 后端
│   │   ├── src/
│   │   │   ├── bridge/            # HTTP API 服务
│   │   │   ├── db/                # SQLite 数据库
│   │   │   ├── native_engine/     # AI 引擎
│   │   │   ├── mcp/               # MCP
│   │   │   ├── memory/            # 记忆系统
│   │   │   ├── terminal/          # PTY 终端
│   │   │   ├── git/               # Git 集成
│   │   │   ├── tools/             # 工具
│   │   │   └── ...                # 更多模块
│   │   ├── Cargo.toml
│   │   └── tauri.conf.json
│   ├── package.json
│   └── vite.config.ts
│
└── README.md                      # 本文档
```

---

## 🗺️ 开发路线

### CLI 版本 — 已完成 ✅

- [x] CLI 基础命令框架
- [x] 配置管理系统
- [x] REPL 交互模式
- [x] MCP 协议支持
- [x] 工具系统 (文件/命令/Git/搜索)
- [x] 内存管理模块
- [x] 插件系统架构
- [x] 安装脚本
- [x] 测试覆盖

### Tauri GUI 版本 — 已完成 ✅

- [x] 三栏可拖拽布局
- [x] Monaco 编辑器集成
- [x] 多标签终端
- [x] Git 可视化 (Diff/Commit)
- [x] 编译打包流水线
- [x] 语音输入
- [x] 图片上传
- [x] SQLite 持久化
- [x] MCP 集成
- [x] 权限管理
- [x] 自动更新

### 进行中 🚧

- [ ] API 流式响应优化
- [ ] 完整的 API 集成测试
- [ ] 性能调优

### 计划中 📋

- [ ] WebAssembly 支持
- [ ] 插件市场 Web 界面
- [ ] 多语言扩展
- [ ] 移动端适配

---

## 🤝 贡献指南

欢迎贡献代码、报告问题或提出建议！

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/your-feature`)
3. 提交更改 (`git commit -m 'feat: add your feature'`)
4. 推送到分支 (`git push origin feature/your-feature`)
5. 创建 Pull Request

### 代码规范

- **Rust**: 使用 `rustfmt` 格式化，通过 `cargo clippy` 检查
- **TypeScript**: 遵循项目 ESLint 配置
- **提交信息**: 使用约定式提交 (`feat:`, `fix:`, `refactor:`, `docs:`, `chore:`, `test:`)

---

## 📄 许可证

MIT License — 详见 [LICENSE](LICENSE) 文件

---

## 🙏 致谢

本项目基于以下优秀开源项目构建：

- [Tauri](https://tauri.app/) — 跨平台桌面框架
- [React](https://react.dev/) — UI 组件库
- [Rust](https://www.rust-lang.org/) — 系统级编程语言
- [Axum](https://github.com/tokio-rs/axum) — Rust Web 框架
- [Monaco Editor](https://microsoft.github.io/monaco-editor/) — VS Code 编辑器核心
- [xterm.js](https://xtermjs.org/) — 终端模拟器
- [SQLite](https://www.sqlite.org/) — 嵌入式数据库
- [Zustand](https://github.com/pmndrs/zustand) — 轻量状态管理

---

<p align="center">
  <strong>Made with ❤️ using Tauri + Rust + React</strong>
  <br><br>
  如果这个项目对你有帮助，请给一个 ⭐️ Star 支持一下！
</p>
