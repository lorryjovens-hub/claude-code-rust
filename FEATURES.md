# Claude Code Rust - 特性开关模块文档

本文档详细说明了 claude-code-rs 中实现的所有特性开关模块。

## 特性开关清单

| 特性标识 | 模块文件 | 状态 | 描述 |
|---------|---------|------|------|
| PROACTIVE | features/proactive.rs | ✅ 完成 | 主动建议模式 |
| KAIROS | features/kairos.rs | ✅ 完成 | 时间感知系统 |
| VOICE_MODE | features/voice.rs | ✅ 完成 | 语音交互模式 |
| ULTRAPLAN | features/ultraplan.rs | ✅ 完成 | 超级规划系统 |
| TORCH | features/torch.rs | ✅ 完成 | 未知功能模块 |
| BUDDY | features/buddy.rs | ✅ 完成 | 伴侣系统 |
| FORK_SUBAGENT | features/fork_subagent.rs | ✅ 完成 | 子代理分叉 |
| BRIDGE_MODE | bridge/mod.rs | ✅ 完成 | 远程控制模式 |
| DAEMON | daemon/mod.rs | ✅ 完成 | 守护进程模式 |
| COORDINATOR_MODE | features/coordinator.rs | ✅ 完成 | 协调器模式 |
| AGENT_TRIGGERS | features/agent_triggers.rs | ✅ 完成 | 代理触发器 |
| WEB_BROWSER_TOOL | features/web_browser.rs | ✅ 完成 | Web 浏览器工具 |
| HISTORY_SNIP | features/history_snip.rs | ✅ 完成 | 历史片段 |
| WORKFLOW_SCRIPTS | features/workflow.rs | ✅ 完成 | 工作流脚本 |

## 特性模块详解

### 1. PROACTIVE - 主动建议模式

**模块文件**: `src/features/proactive.rs`

**功能说明**:
- AI 主动分析代码并提供建议
- 支持多种建议类型（代码改进、潜在错误、最佳实践等）
- 优先级管理和过滤
- 自动和手动触发模式

**核心类型**:
- `ProactiveSuggestionType`: 建议类型枚举
- `ProactiveSuggestion`: 建议结构体
- `ProactiveManager`: 建议管理器

### 2. KAIROS - 时间感知系统

**模块文件**: `src/features/kairos.rs`

**功能说明**:
- 基于时间的智能响应
- 时间段识别（早晨、下午、晚上等）
- 用户活动模式学习
- 时间相关建议

**核心类型**:
- `TimePeriod`: 时间段枚举
- `UserActivityPattern`: 用户活动模式
- `KairosManager`: 时间感知管理器

### 3. VOICE_MODE - 语音交互模式

**模块文件**: `src/features/voice.rs`

**功能说明**:
- 语音输入识别
- 语音输出合成
- 多种语音配置
- 唤醒词支持
- 语音历史记录

**核心类型**:
- `VoiceConfig`: 语音配置
- `VoiceManager`: 语音管理器
- `SpeechRecognitionResult`: 识别结果

### 4. ULTRAPLAN - 超级规划系统

**模块文件**: `src/features/ultraplan.rs`

**功能说明**:
- 复杂任务智能规划
- 多代理探索
- 计划审批流程
- 远程执行支持
- 进度跟踪

**核心类型**:
- `PlanPhase`: 规划阶段
- `UltraPlan`: 超级计划
- `UltraPlanManager`: 计划管理器

### 5. TORCH - 未知功能模块

**模块文件**: `src/features/torch.rs`

**功能说明**:
- 可扩展的实验性功能框架
- 功能注册和管理
- 插件式架构
- 多种模式支持

**核心类型**:
- `TorchMode`: Torch 模式
- `TorchFeature`: Torch 功能 trait
- `TorchManager`: Torch 管理器

### 6. BUDDY - 伴侣系统

**模块文件**: `src/features/buddy.rs`

**功能说明**:
- AI 伙伴陪伴式交互
- 多种性格选择
- 对话历史管理
- 用户偏好学习
- 情感化回应

**核心类型**:
- `BuddyPersonality`: 伙伴性格
- `BuddyManager`: 伙伴管理器
- `ConversationHistory`: 对话历史

### 7. FORK_SUBAGENT - 子代理分叉

**模块文件**: `src/features/fork_subagent.rs`

**功能说明**:
- 创建独立子代理
- 共享缓存机制
- 消息传递系统
- 子代理生命周期管理
- 结果收集

**核心类型**:
- `SubagentState`: 子代理状态
- `SubagentConfig`: 子代理配置
- `SubagentManager`: 子代理管理器

### 8. COORDINATOR_MODE - 协调器模式

**模块文件**: `src/features/coordinator.rs`

**功能说明**:
- 多工作器协调
- 任务队列管理
- 依赖关系处理
- 简单/完整双模式
- 自动任务分配

**核心类型**:
- `CoordinatorMode`: 协调器模式
- `WorkerInfo`: 工作器信息
- `CoordinatorManager`: 协调器管理器

### 9. AGENT_TRIGGERS - 代理触发器

**模块文件**: `src/features/agent_triggers.rs`

**功能说明**:
- 基于事件的代理触发
- 多种触发器类型
- 条件评估
- 动作执行
- 冷却机制

**核心类型**:
- `TriggerType`: 触发器类型
- `AgentTrigger`: 代理触发器
- `AgentTriggerManager`: 触发器管理器

### 10. WEB_BROWSER_TOOL - Web 浏览器工具

**模块文件**: `src/features/web_browser.rs`

**功能说明**:
- 网页导航
- 截图功能
- 元素交互
- JavaScript 执行
- 内容提取

**核心类型**:
- `BrowserAction`: 浏览器操作
- `BrowserManager`: 浏览器管理器
- `NavigationResult`: 导航结果

### 11. HISTORY_SNIP - 历史片段

**模块文件**: `src/features/history_snip.rs`

**功能说明**:
- 对话历史管理
- 多种压缩策略
- Token 计数
- 优先级保留
- 智能摘要

**核心类型**:
- `SnippetType`: 片段类型
- `HistorySnippet`: 历史片段
- `HistorySnipManager`: 历史片段管理器

### 12. WORKFLOW_SCRIPTS - 工作流脚本

**模块文件**: `src/features/workflow.rs`

**功能说明**:
- 工作流定义
- 步骤编排
- 条件判断
- 循环和并行
- 触发器系统

**核心类型**:
- `WorkflowDefinition`: 工作流定义
- `WorkflowStep`: 工作流步骤
- `WorkflowManager`: 工作流管理器

## 使用指南

### 特性管理器

所有特性都通过 `FeatureManager` 进行统一管理：

```rust
use claude_code_rs::features::{FeatureManager, FeatureFlag};

let mut feature_manager = FeatureManager::new();

// 启用特性
feature_manager.enable(FeatureFlag::Proactive);

// 禁用特性
feature_manager.disable(FeatureFlag::Proactive);

// 检查特性状态
if feature_manager.is_enabled(FeatureFlag::Proactive) {
    // 使用特性
}

// 从环境变量加载
feature_manager.load_from_env();
```

### 使用具体特性模块

```rust
use claude_code_rs::features::proactive::ProactiveManager;
use claude_code_rs::state::AppState;

let state = AppState::new();
let mut proactive_manager = ProactiveManager::new(state);

// 使用主动建议功能
```

## 扩展开发

### 添加新特性

1. 在 `src/features/` 中创建新模块
2. 在 `src/features/mod.rs` 中添加 `FeatureFlag` 变体
3. 实现特性管理器
4. 更新特性管理器中的默认状态

### 特性最佳实践

1. **类型安全**: 使用 Rust 的类型系统确保安全
2. **错误处理**: 合理使用 `Result` 类型
3. **异步优先**: 使用 `async_trait` 实现异步功能
4. **状态管理**: 使用 `Arc<RwLock>` 管理共享状态
5. **文档完整**: 为公共 API 添加文档注释

## 项目结构

```
src/
├── features/
│   ├── mod.rs              # 特性管理器和 FeatureFlag 枚举
│   ├── proactive.rs        # PROACTIVE 主动建议
│   ├── kairos.rs           # KAIROS 时间感知
│   ├── voice.rs            # VOICE_MODE 语音
│   ├── ultraplan.rs        # ULTRAPLAN 超级规划
│   ├── torch.rs            # TORCH 未知功能
│   ├── buddy.rs            # BUDDY 伴侣系统
│   ├── fork_subagent.rs    # FORK_SUBAGENT 子代理
│   ├── coordinator.rs      # COORDINATOR_MODE 协调器
│   ├── agent_triggers.rs   # AGENT_TRIGGERS 触发器
│   ├── web_browser.rs      # WEB_BROWSER_TOOL 浏览器
│   ├── history_snip.rs     # HISTORY_SNIP 历史片段
│   └── workflow.rs         # WORKFLOW_SCRIPTS 工作流
├── bridge/                 # BRIDGE_MODE 远程控制
└── daemon/                 # DAEMON 守护进程
```

## 许可证

与原 Claude Code 项目相同的许可证。
