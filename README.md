# 🚀 Claude Desktop - Tauri AI 操作系统

> 顶级 AI 桌面客户端 | 完整的编程、文件管理、编译打包能力

<p align="center">
  <a href="https://claude-rust-gui.netlify.app/">项目介绍</a> •
  <a href="#-快速开始">快速开始</a> •
  <a href="#-核心功能">核心功能</a> •
  <a href="#-技术栈">技术栈</a>
</p>

---

## 🌟 项目简介

这是一个基于 **Tauri 2.0** 构建的完整 AI 桌面客户端，集成了文件管理、代码编辑、终端模拟、Git 可视化、编译打包等复杂电脑操作能力，打造顶级 AI 操作系统体验。

---

## ✨ 核心功能

### 📁 三栏可拖拽布局
- **文件树面板**：浏览本地文件系统，显示 Git 状态标记
- **Monaco 编辑器**：VS Code 同款编辑器，支持语法高亮、代码折叠、多标签
- **实时预览面板**：支持 HTML/JSX 实时渲染

### 💻 开发工具集成
- **多标签终端**：基于 xterm.js，支持 ANSI 颜色转义序列
- **Git 可视化**：文件树显示 Git 状态，可视化 Diff 和 Commit 界面
- **编译打包流水线**：自动检测项目类型并执行构建命令
- **文件操作**：下载、复制路径、在系统默认应用中打开

### 🤖 AI 能力
- **多模型支持**：Anthropic、OpenAI、DeepSeek 等兼容 API
- **流式响应**：SSE 实时显示 AI 回复
- **语音输入**：支持语音转文字输入
- **图片上传**：支持发送图片给 AI 分析
- **SQLite 持久化**：对话历史、模型使用记录、API 调用追踪

### 🔧 高级功能
- **MCP 集成**：Model Context Protocol 支持外部工具调用
- **多 Agent 协作**：协调多个 AI 代理并行处理子任务
- **技能系统**：用户可扩展的插件模块
- **斜杠命令**：快速操作输入
- **权限管理**：可配置的权限模式（每次询问、自动接受、仅计划等）

### 🖥️ 桌面集成
- 系统托盘图标
- 原生文件对话框
- 剪贴板集成
- 桌面通知
- 自动更新检查

---

## 🚀 快速开始

### 系统要求

- **Rust** 1.70+ ([rustup](https://rustup.rs))
- **Node.js** 18+ ([nvm](https://github.com/nvm-sh/nvm))
- **Windows**: Visual Studio C++ Build Tools, WebView2
- **macOS**: Xcode Command Line Tools
- **Linux**: `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`

### 安装运行

```bash
# 克隆仓库
git clone https://github.com/lorryjovens-hub/claude-code-rust.git
cd claude-code-rust

# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 构建生产版本
npm run tauri build
```

### 配置 API

```bash
# 方式 1: 环境变量
export ANTHROPIC_API_KEY="your-api-key"

# 方式 2: 配置文件 .env
ANTHROPIC_API_KEY=your-api-key
API_BASE_URL=https://api.anthropic.com
```

---

## 🏗️ 技术栈

| 层级 | 技术 | 用途 |
|------|------|------|
| **前端框架** | React 19 + TypeScript | UI 组件 |
| **构建工具** | Vite 6 | 开发服务器和打包 |
| **样式方案** | Tailwind CSS 3 | 原子化 CSS |
| **状态管理** | Zustand 5 | 轻量级状态管理 |
| **编辑器** | Monaco Editor | VS Code 同款编辑器 |
| **终端** | xterm.js 5 | 终端模拟 |
| **数学渲染** | KaTeX | 公式渲染 |
| **图表渲染** | Mermaid + recharts | 图表和流程图 |

| 层级 | 技术 | 用途 |
|------|------|------|
| **桌面框架** | Tauri 2.0 | 跨平台桌面应用 |
| **后端语言** | Rust | 系统级后端 |
| **HTTP 框架** | Axum 0.8 | 前后端桥接 |
| **异步运行时** | Tokio 1 | 异步任务处理 |
| **数据库** | rusqlite (SQLite) | 本地数据持久化 |
| **HTTP 客户端** | reqwest 0.12 | AI API 调用 |
| **序列化** | Serde + serde_json | JSON 处理 |

---

## 📁 项目结构

```
claude-desktop-tauri/
├── src/                    # React 前端源码
│   ├── components/         # UI 组件 (~80+ 文件)
│   │   ├── chat/          # 聊天界面组件
│   │   ├── ui/            # 通用 UI 组件
│   │   └── ...
│   ├── features/           # 功能模块
│   ├── stores/             # Zustand 状态管理
│   ├── hooks/              # 自定义 React Hooks
│   ├── services/           # API 服务层
│   ├── api/                # API 客户端
│   └── utils/              # 工具函数
│
├── src-tauri/              # Rust 后端源码
│   ├── src/
│   │   ├── main.rs        # Tauri 入口
│   │   ├── bridge/        # Axum HTTP API 服务
│   │   ├── db/            # SQLite 数据库
│   │   ├── mcp/           # MCP 客户端/服务端
│   │   ├── native_engine/ # AI 引擎核心
│   │   ├── memory/        # 对话记忆系统
│   │   └── ...            # 更多模块
│   ├── Cargo.toml         # Rust 依赖配置
│   └── tauri.conf.json    # Tauri 配置
│
├── package.json           # Node.js 依赖
├── vite.config.ts         # Vite 配置
├── tailwind.config.js     # Tailwind 配置
└── tsconfig.json          # TypeScript 配置
```

---

## 📊 功能对比

| 功能 | 状态 |
|------|------|
| 三栏可拖拽布局 | ✅ 完整 |
| Monaco 编辑器 (VS Code 同款) | ✅ 完整 |
| 多标签终端 (xterm.js) | ✅ 完整 |
| Git 可视化 Diff/Commit | ✅ 完整 |
| 编译打包流水线 | ✅ 完整 |
| 语音输入 & 图片上传 | ✅ 完整 |
| SQLite 对话历史 | ✅ 完整 |
| MCP 集成 | ✅ 完整 |
| 文件管理器 + 实时预览 | ✅ 完整 |
| 多 Agent 协作 | ✅ 完整 |
| 权限管理 | ✅ 完整 |
| 自动更新 | ✅ 完整 |

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

MIT License - 详见 [LICENSE](LICENSE) 文件

---

## 🙏 致谢

本项目基于以下优秀开源项目构建：

- [Tauri](https://tauri.app/) - 桌面应用框架
- [React](https://react.dev/) - UI 库
- [Axum](https://github.com/tokio-rs/axum) - Rust Web 框架
- [SQLite](https://www.sqlite.org/) - 嵌入式数据库
- [Zustand](https://github.com/pmndrs/zustand) - 状态管理
- [Monaco Editor](https://microsoft.github.io/monaco-editor/) - 代码编辑器
- [xterm.js](https://xtermjs.org/) - 终端模拟器

---

**Made with ❤️ using Tauri + Rust + React**

如果这个项目对你有帮助，请给一个 ⭐️ Star 支持一下！
