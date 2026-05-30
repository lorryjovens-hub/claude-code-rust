# CI/CD 配置指南

## 概述

本项目使用 GitHub Actions 实现自动化的跨平台构建和发布。

## 触发条件

| 事件 | 行为 |
|------|------|
| `push` 到 main/master | 运行测试并构建所有平台 |
| `push` 标签 (v*) | 构建 + 创建 GitHub Release |
| `pull_request` | 仅运行测试和 lint |
| 手动触发 | 运行测试并构建所有平台 |

## GitHub Secrets 配置

在 GitHub 仓库的 Settings > Secrets and variables > Actions 中配置以下变量：

### 必需（用于签名和更新验证）

| Secret | 说明 | 获取方式 |
|--------|------|----------|
| `TAURI_SIGNING_PRIVATE_KEY` | Tauri 更新私钥 | `npx tauri signer generate --key-password <password>` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 私钥密码 | 上面命令使用的密码 |

### 可选（macOS 代码签名和公证）

| Secret | 说明 | 获取方式 |
|--------|------|----------|
| `APPLE_CERTIFICATE` | Apple 分发证书 (base64) | Keychain Access > 导出 .p12 > `base64 <file>` |
| `APPLE_CERTIFICATE_PASSWORD` | 证书密码 | 导出 .p12 时设置的密码 |
| `APPLE_SIGNING_IDENTITY` | 签名身份名称 | Keychain Access > 证书 > 显示名称 |
| `APPLE_ID` | Apple ID 邮箱 | 你的 Apple ID |
| `APPLE_PASSWORD` | 应用专用密码 | appleid.apple.com > 应用专用密码 |
| `APPLE_TEAM_ID` | Apple 团队 ID | developer.apple.com > Membership |

### 配置步骤

```bash
# 1. 生成 Tauri 签名密钥
npx tauri signer generate --key-password your-strong-password

# 2. 获取公钥并配置到 tauri.conf.json
cat ~/.tauri/key.pub
# 将内容复制到 plugins.updater.pubkey

# 3. 配置 GitHub Secrets
gh secret set TAURI_SIGNING_PRIVATE_KEY < ~/.tauri/key
gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD <<< "your-strong-password"
```

## 本地发布

### 快速发布

```bash
# 自动 bump patch 版本并构建
npm run publish:patch

# 或者 bump minor 版本
npm run publish:minor

# 或者 bump major 版本
npm run publish:major

# 只构建不发布
npm run publish:dry-run
```

### 使用 PowerShell (Windows)

```powershell
# 发布新版本
.\scripts\publish.ps1 -Release

# 指定 bump 类型
.\scripts\publish.ps1 -Release -Bump minor

# 指定版本号
.\scripts\publish.ps1 -Release -Version "3.2.0"
```

### 使用 GitHub CLI 上传

```bash
# 上传本地构建到 GitHub Release
./scripts/upload-release.sh v3.2.0

# 上传最新的本地构建
./scripts/upload-release.sh
```

## 手动创建 Release

```bash
# 1. 创建并推送标签
git tag -a "v3.2.0" -m "Release v3.2.0"
git push origin v3.2.0

# 2. CI 会自动构建并发布
# 或者手动创建：
gh release create v3.2.0 \
    --title "v3.2.0" \
    --notes-file RELEASE_NOTES.md \
    ./dist-release/3.2.0/*
```

## 构建产物

### Windows
- `.exe` - NSIS 安装包
- `.msi` - Windows Installer 包

### macOS
- `.dmg` - 磁盘镜像安装包
  - 支持 Intel (x86_64) 和 Apple Silicon (aarch64)
- `.app` - 独立应用

### Linux
- `.deb` - Debian/Ubuntu 安装包
- `.AppImage` - 通用 Linux 应用

## 自动更新

配置 `tauri.conf.json` 中的 `plugins.updater` 后，应用会自动检查更新：

```json
{
  "plugins": {
    "updater": {
      "active": true,
      "dialog": true,
      "pubkey": "<你的公钥>",
      "endpoints": [
        "https://github.com/<owner>/<repo>/releases/latest/download/update.json"
      ]
    }
  }
}
```

## 故障排除

### 构建失败

1. 检查 Actions 标签页查看详细日志
2. 确认所有系统依赖已安装
3. 尝试手动触发工作流

### macOS 签名失败

```bash
# 检查证书是否有效
security find-identity -v -p codesigning

# 检查环境变量
gh secret list
```

### 签名公钥不匹配

确保 `tauri.conf.json` 中的 `pubkey` 与生成签名时使用的公钥一致。
