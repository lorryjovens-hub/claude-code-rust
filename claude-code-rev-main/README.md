
# Restored Claude Code Source


![Preview](preview.png)


  This repository is a restored Claude Code source tree reconstructed primarily from source maps and missing-module backfilling.

  It is not the original upstream repository state. Some files were unrecoverable from source maps and have been replaced with compatibility shims or degraded implementations so the
  project can install and run again.

  ## Current status

  - The source tree is restorable and runnable in a local development workflow.
  - `bun install` succeeds.
  - `bun run version` succeeds.
  - `bun run dev` now routes through the restored CLI bootstrap instead of the temporary `dev-entry` shim.
  - `bun run dev --help` shows the full command tree from the restored CLI.
  - A number of modules still contain restoration-time fallbacks, so behavior may differ from the original Claude Code implementation.

  ## Restored so far

  Recent restoration work has recovered several pieces beyond the initial source-map import:

  - the default Bun scripts now start the real CLI bootstrap path
  - bundled skill content for `claude-api` and `verify` has been rewritten from placeholder files into usable reference docs
  - compatibility layers for Chrome MCP and Computer Use MCP now expose realistic tool catalogs and structured degraded-mode responses instead of empty stubs
  - several explicit placeholder resources have been replaced with working fallback prompts for planning and permission-classifier flows

  Remaining gaps are mostly private/native integrations where the original implementation was not recoverable from source maps, so those areas still rely on shims or reduced behavior.

  ## Why this exists

  Source maps do not contain a full original repository:

  - type-only files are often missing
  - build-time generated files may be absent
  - private package wrappers and native bindings may not be recoverable
  - dynamic imports and resource files are frequently incomplete

  This repository fills those gaps enough to produce a usable, runnable restored workspace.

  ## Run

  Requirements:

  - Bun 1.3.5 or newer
  - Node.js 24 or newer

  Install dependencies:

  ```bash
  bun install
  ```

  Run the restored CLI:

  ```bash
  bun run dev
  ```

  Print the restored version:

  ```bash
  bun run version
  ```

  ## 中文说明

  # 还原后的 Claude Code 源码

  ![Preview](preview.png)

  这个仓库是一个主要通过 source map 逆向还原、再补齐缺失模块后得到的 Claude Code 源码树。

  它并不是上游仓库的原始状态。部分文件无法仅凭 source map 恢复，因此目前仍包含兼容 shim 或降级实现，以便项目可以重新安装并运行。

  ### 当前状态

  - 该源码树已经可以在本地开发流程中恢复并运行。
  - `bun install` 可以成功执行。
  - `bun run version` 可以成功执行。
  - `bun run dev` 现在会通过还原后的真实 CLI bootstrap 启动，而不是临时的 `dev-entry`。
  - `bun run dev --help` 可以显示还原后的完整命令树。
  - 仍有部分模块保留恢复期 fallback，因此行为可能与原始 Claude Code 实现不同。

  ### 已恢复内容

  最近一轮恢复工作已经补回了最初 source-map 导入之外的几个关键部分：

  - 默认 Bun 脚本现在会走真实的 CLI bootstrap 路径
  - `claude-api` 和 `verify` 的 bundled skill 内容已经从占位文件恢复为可用参考文档
  - Chrome MCP 和 Computer Use MCP 的兼容层现在会暴露更接近真实的工具目录，并返回结构化的降级响应，而不是空 stub
  - 一些显式占位资源已经替换为可用的 planning 与 permission-classifier fallback prompt

  当前剩余缺口主要集中在私有或原生集成部分，这些实现无法仅凭 source map 完整恢复，因此这些区域仍依赖 shim 或降级行为。

  ### 为什么会有这个仓库

  source map 本身并不能包含完整的原始仓库：

  - 类型专用文件经常缺失
  - 构建时生成的文件可能不存在
  - 私有包包装层和原生绑定可能无法恢复
  - 动态导入和资源文件经常不完整

  这个仓库的目标是把这些缺口补到“可用、可运行”的程度，形成一个可继续修复的恢复工作区。

  ### 运行方式

  环境要求：

  - Bun 1.3.5 或更高版本
  - Node.js 24 或更高版本

  安装依赖：

  ```bash
  bun install
  ```

  运行恢复后的 CLI：

  ```bash
  bun run dev
  ```

  输出恢复后的版本号：

  ```bash
  bun run version
  ```
