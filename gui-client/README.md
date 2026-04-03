# Claude Code GUI Client

一个功能完善的GUI客户端应用，为Rust版本的Claude代码提供现代化的用户界面。

## 项目概述

本项目是一个基于Tauri + React的跨平台桌面应用，提供了与Rust Claude Code后端的无缝集成，支持对话、任务管理、多模型集成等核心功能。

## 技术栈

- **前端框架**: React 18 + TypeScript
- **构建工具**: Vite
- **桌面框架**: Tauri 2.0
- **样式**: Tailwind CSS
- **UI组件**: Radix UI
- **状态管理**: Zustand
- **数据获取**: Tauri API

## 项目结构

```
gui-client/
├── src/                    # 前端源代码
│   ├── components/         # React组件
│   │   ├── chat/          # 对话功能模块
│   │   ├── tasks/         # 任务管理模块
│   │   ├── models/        # 模型管理模块
│   │   ├── settings/      # 设置页面
│   │   └── ui/            # 基础UI组件
│   ├── hooks/             # 自定义Hooks
│   ├── lib/               # 工具函数和API
│   ├── types/             # TypeScript类型定义
│   ├── App.tsx            # 主应用组件
│   └── main.tsx           # 应用入口
├── src-tauri/             # Tauri后端代码
│   ├── src/               # Rust源代码
│   ├── Cargo.toml         # Rust依赖配置
│   └── tauri.conf.json    # Tauri配置
├── package.json           # Node.js依赖
├── tsconfig.json          # TypeScript配置
├── vite.config.ts         # Vite配置
└── tailwind.config.js     # Tailwind配置
```

## 核心功能

### 1. 对话功能
- 支持流式响应显示
- Markdown渲染支持
- 对话历史管理
- 多模型切换

### 2. 任务管理
- 创建、编辑、删除任务
- 任务优先级设置
- 子任务管理
- AI辅助生成子任务
- 进度追踪

### 3. 多模型集成
- 支持多个AI提供商
- 模型配置管理
- API密钥管理
- 模型测试功能

### 4. 设置管理
- 主题切换（浅色/深色/系统）
- 语言设置
- 快捷键配置
- 对话行为设置

## 快速开始

### 安装依赖

```bash
# 安装Node.js依赖
npm install

# 安装Rust依赖（Tauri）
cd src-tauri
cargo build
```

### 开发模式

```bash
# 启动开发服务器
npm run tauri:dev
```

### 构建应用

```bash
# 构建生产版本
npm run tauri:build
```

## 与Rust后端集成

GUI客户端通过Tauri的Invoke API与Rust后端通信。主要的API调用包括：

- **Chat API**: 发送消息、获取对话历史
- **Task API**: 任务CRUD操作
- **Model API**: 模型管理和配置
- **Settings API**: 应用设置管理

## 自定义主题

应用使用Claude品牌色（橙色）作为主色调，支持浅色和深色模式。主题配置位于：
- `tailwind.config.js`: 颜色定义
- `src/index.css`: CSS变量和全局样式

## 响应式设计

应用支持响应式布局：
- 桌面端：侧边栏导航
- 移动端：顶部导航栏
- 自适应内容区域

## 开发计划

### 已完成
- [x] 项目架构搭建
- [x] UI组件库
- [x] 对话功能
- [x] 任务管理
- [x] 模型管理
- [x] 设置页面
- [x] 响应式布局

### 待开发
- [ ] 与Rust后端的完整集成
- [ ] 流式响应实现
- [ ] 文件上传功能
- [ ] 代码高亮优化
- [ ] 国际化完善
- [ ] 插件系统

## 贡献指南

1. Fork项目
2. 创建特性分支
3. 提交更改
4. 推送到分支
5. 创建Pull Request

## 许可证

MIT License
