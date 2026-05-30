# ⚠️ 此分支已废弃 / This Branch Is Deprecated

> **请使用 `main` 分支获取最新版本**
> 
> 👉 [切换到 main 分支](https://github.com/lorryjovens-hub/claude-code-rust/tree/main)
> 
> 👉 [项目完整介绍和下载](https://claude-rust-gui.netlify.app/)

---

## 为什么此分支被废弃？

此分支 (`old-main`) 包含早期的混乱开发历史，包含大量临时文件、构建产物和半成品代码：

- ❌ 27,946+ 个 git 对象，仓库体积过大
- ❌ 大量临时文件和调试脚本 (`fix-*.cjs`, `build_log.txt` 等)
- ❌ 半成品独立应用 (`prd-creator`, `remotion-studio`, `worker`)
- ❌ 大型设计素材库 (音效、展示图片等 ~100 个文件)
- ❌ 重复的图标和资源文件
- ❌ AI 生成的设计文档混杂在代码中

## 新的 `main` 分支有什么改进？

✅ **整洁的仓库结构** - 只有 518 个对象，1.46 MiB
✅ **完整的 Tauri 客户端** - 顶级 AI 操作系统
✅ **干净的 git 历史** - 从零开始的干净提交
✅ **完善的 .gitignore** - 防止临时文件进入仓库
✅ **所有半成品代码已移除**

### 核心功能

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

## 快速开始

```bash
# 确保在 main 分支
git checkout main

# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 构建生产版本
npm run tauri build
```

## 技术栈

- **前端**: React + TypeScript + Tailwind CSS
- **后端**: Rust + Tauri 2.0
- **数据库**: SQLite
- **编辑器**: Monaco Editor
- **终端**: xterm.js

---

**此分支仅供历史参考，请勿基于此分支进行开发。**
