# 维护者指南

本文档面向项目维护者，说明如何管理项目和社区。

## 维护者职责

### 代码审查

- 及时审查 Pull Request（最好在 48 小时内）
- 确保代码符合项目规范
- 提供建设性的反馈
- 感谢贡献者的努力

### Issue 管理

- 标记和分类 Issue
- 及时回复问题
- 关闭已解决的 Issue
- 将 Issue 分配给适当的贡献者

### 发布管理

- 维护发布说明
- 管理版本号
- 创建发布标签
- 更新文档

## 添加新维护者

当贡献者持续为项目做出高质量贡献时，可以考虑邀请他们成为维护者。

### 标准

- 至少 5 个被合并的 Pull Request
- 积极参与 Issue 讨论
- 遵守行为准则
- 对项目有深入理解

### 流程

1. 现有维护者提名
2. 获得多数维护者同意
3. 向被提名者发送邀请
4. 更新 MAINTAINERS.md 文件

## 当前维护者

| 用户名 | GitHub | 职责 |
|--------|--------|------|
| @lorryjovens-hub | [lorryjovens-hub](https://github.com/lorryjovens-hub) | 项目创始人，核心维护 |

## 权限级别

### 1. 贡献者 (Contributor)

- 可以 Fork 仓库
- 可以提交 Pull Request
- 可以创建 Issue

### 2. 协作者 (Collaborator)

- 所有贡献者权限
- 可以直接推送到非保护分支
- 可以审查 Pull Request
- 可以管理 Issue 和 PR

### 3. 维护者 (Maintainer)

- 所有协作者权限
- 可以推送到保护分支
- 可以管理发布
- 可以添加/删除协作者
- 可以管理 GitHub Actions

### 4. 管理员 (Admin)

- 所有维护者权限
- 可以管理仓库设置
- 可以删除仓库
- 可以管理组织设置

## 分支保护规则

### main 分支

- 需要 Pull Request 审查（至少 1 人）
- 需要通过 CI 检查
- 禁止强制推送
- 禁止删除

### develop 分支

- 需要 Pull Request 审查（建议）
- 需要通过 CI 检查

## 发布流程

1. 更新版本号
2. 更新 CHANGELOG.md
3. 创建发布分支
4. 运行完整测试
5. 创建 Pull Request 到 main
6. 合并后创建标签
7. 创建 GitHub Release
8. 更新文档

## 社区管理

### 处理冲突

1. 保持冷静和专业
2. 私下联系相关人员
3. 寻求第三方调解（如需要）
4. 必要时执行行为准则

### 鼓励贡献

- 感谢所有贡献
- 标记 "good first issue"
- 提供清晰的贡献指南
- 及时回复问题

### 沟通渠道

- GitHub Issues - Bug 报告和功能请求
- GitHub Discussions - 一般讨论
- Pull Requests - 代码审查

## 安全

### 报告安全漏洞

1. 不要公开披露
2. 发送邮件到维护者
3. 等待修复后再公开

### 依赖管理

- 定期更新依赖
- 监控安全公告
- 使用 `cargo audit` 检查 Rust 依赖
- 使用 `npm audit` 检查 Node.js 依赖

## 文档维护

- 保持 README 更新
- 维护 API 文档
- 更新贡献指南
- 维护变更日志

---

最后更新：2024年4月
