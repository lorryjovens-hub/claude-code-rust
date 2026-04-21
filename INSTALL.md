# Claude Code Rust 安装指南

本指南将帮助您通过命令行安装和配置 Claude Code Rust 版本。

## 系统要求

- **Rust**：需要安装 Rust 1.70 或更高版本
- **Git**：需要安装 Git 用于克隆仓库
- **操作系统**：支持 Windows、Linux、macOS

## 安装步骤

### 1. 克隆仓库

```bash
git clone https://github.com/lorryjovens-hub/claude-code-rust
cd claude-code-rust
```

### 2. 运行安装脚本

#### Windows (PowerShell)

**默认安装（临时目录）：**
```powershell
# 使用 PowerShell 运行以下命令
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser -Force
.\scripts\install-windows.ps1
```

**安装到D盘：**
```powershell
# 使用 PowerShell 运行以下命令
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser -Force
.\scripts\install-windows.ps1 -InstallDir "D:\claude-code\install"
```

#### Linux / macOS (Bash)

**默认安装：**
```bash
# 使用 Bash 运行以下命令
chmod +x ./scripts/install-linux.sh
./scripts/install-linux.sh
```

**指定安装目录：**
```bash
# 使用 Bash 运行以下命令
chmod +x ./scripts/install-linux.sh
./scripts/install-linux.sh --install-dir "/path/to/install"
```

### 3. 手动安装（可选）

如果安装脚本出现问题，您可以手动安装：

#### 构建项目
```bash
cargo build --release
```

#### 安装到系统

##### Windows
```powershell
# 创建安装目录
$installDir = "$env:USERPROFILE\.claude-code\bin"
New-Item -ItemType Directory -Path $installDir -Force

# 复制可执行文件
Copy-Item ".\target\release\claude-code.exe" "$installDir\claude-code.exe"

# 添加到 PATH
$currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if (-not $currentPath.Contains($installDir)) {
    [Environment]::SetEnvironmentVariable("PATH", "$currentPath;$installDir", "User")
}
```

##### Linux / macOS
```bash
# 创建安装目录
mkdir -p ~/.claude-code/bin

# 复制可执行文件
cp ./target/release/claude-code ~/.claude-code/bin/
chmod +x ~/.claude-code/bin/claude-code

# 添加到 PATH
echo "export PATH=\"$HOME/.claude-code/bin:\$PATH\"" >> ~/.bashrc
# 或对于 zsh
# echo "export PATH=\"$HOME/.claude-code/bin:\$PATH\"" >> ~/.zshrc
```

## 配置

### 设置 API 密钥

```bash
# 设置 API 密钥
claude-code config set api_key "your-api-key"

# 设置 API 基础 URL
claude-code config set base_url "https://api.deepseek.com"

# 设置模型
claude-code config set model "deepseek-reasoner"
```

### 验证配置

```bash
claude-code config list
```

## 测试安装

```bash
# 测试基本功能
claude-code query --prompt "Hello!"

# 启动交互模式
claude-code repl

# 查看帮助信息
claude-code --help
```

## 升级

### 通过安装脚本升级

重新运行安装脚本即可自动升级到最新版本：

#### Windows
```powershell
.\scripts\install-windows.ps1
```

#### Linux / macOS
```bash
./scripts/install-linux.sh
```

### 手动升级

```bash
cd claude-code-rust
git pull
cargo build --release
# 然后重新复制可执行文件到安装目录
```

## 卸载

### Windows

```powershell
# 删除安装目录
Remove-Item -Path "$env:USERPROFILE\.claude-code" -Recurse -Force

# 从 PATH 中移除
$currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
$newPath = $currentPath -replace "$env:USERPROFILE\\.claude-code\\bin;?", ""
[Environment]::SetEnvironmentVariable("PATH", $newPath, "User")

# 删除配置目录
Remove-Item -Path "$env:USERPROFILE\.config\claude-code" -Recurse -Force
```

### Linux / macOS

```bash
# 删除安装目录
rm -rf ~/.claude-code

# 从 PATH 中移除
# 编辑 ~/.bashrc 或 ~/.zshrc 文件，删除包含 "~/.claude-code/bin" 的行

# 删除配置目录
rm -rf ~/.config/claude-code
```

## 故障排除

### 常见问题

1. **Rust 未安装**
   - 访问 https://rustup.rs/ 安装 Rust

2. **Git 未安装**
   - Windows: 访问 https://git-scm.com/ 安装 Git
   - Linux: 使用包管理器安装 (e.g., `apt install git`)
   - macOS: 使用 Homebrew 安装 (`brew install git`)

3. **API 密钥配置错误**
   - 确保使用正确的 API 密钥格式
   - 确保 API 密钥具有足够的权限

4. **PATH 配置问题**
   - 安装后重启终端
   - 手动检查 PATH 环境变量是否包含安装目录

### 查看日志

```bash
# 设置详细日志
RUST_LOG=claude_code=debug claude-code query --prompt "Hello!"
```

## 联系方式

如果您遇到任何问题，请在 GitHub 仓库中创建 issue：
https://github.com/lorryjovens-hub/claude-code-rust/issues
