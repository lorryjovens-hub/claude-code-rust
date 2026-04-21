# Claude Code Rust 安装脚本 (Windows PowerShell)
# 用法: irm https://install.claude-code-rs.io/ps1 | iex

param(
    [string]$InstallPath = "$env:LOCALAPPDATA\claude-code-rust",
    [switch]$AddToPath = $false,
    [switch]$SystemWide = $false
)

# 常量
$REPO = "lorryjovens-hub/claude-code-rust"
$APP_NAME = "Claude Code Rust"

# 颜色函数
function Write-Success { Write-Host "✓ $args" -ForegroundColor Green }
function Write-Error { Write-Host "✗ $args" -ForegroundColor Red }
function Write-Warning { Write-Host "⚠ $args" -ForegroundColor Yellow }
function Write-Header { 
    Write-Host ""
    Write-Host "═════════════════════════════════════════" -ForegroundColor Cyan
    Write-Host "$args" -ForegroundColor Cyan
    Write-Host "═════════════════════════════════════════" -ForegroundColor Cyan
}

# 检测 PowerShell 版本
function Test-PowerShellVersion {
    Write-Header "检查 PowerShell 版本"
    
    $version = $PSVersionTable.PSVersion
    if ($version.Major -lt 5) {
        Write-Error "需要 PowerShell 5.0 或更高版本 (当前: $version)"
        Write-Warning "请升级 PowerShell: https://github.com/PowerShell/PowerShell"
        exit 1
    }
    Write-Success "PowerShell 版本: $version"
}

# 检测已有版本
function Test-ExistingInstallation {
    Write-Header "检查现有安装"
    
    if (Get-Command claude-code-rs -ErrorAction SilentlyContinue) {
        try {
            $version = & claude-code-rs --version
            Write-Warning "已安装版本: $version"
            
            $confirm = Read-Host "继续安装将覆盖现有版本，是否继续? (y/N)"
            if ($confirm -ne "y" -and $confirm -ne "Y") {
                Write-Error "取消安装"
                exit 1
            }
        } catch {
            Write-Warning "检测到了旧版本，将进行更新"
        }
    }
}

# 设置安装路径
function Set-InstallationPath {
    Write-Header "设置安装路径"
    
    # 创建安装目录
    if (!(Test-Path $InstallPath)) {
        New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null
    }
    
    Write-Success "安装路径: $InstallPath"
}

# 获取最新版本
function Get-LatestVersion {
    Write-Header "获取最新版本"
    
    try {
        $releases = Invoke-WebRequest -Uri "https://api.github.com/repos/$REPO/releases/latest" -UseBasicParsing
        $json = $releases.Content | ConvertFrom-Json
        $version = $json.tag_name
        
        Write-Success "最新版本: $version"
        return $version
    } catch {
        Write-Warning "无法获取最新版本，使用默认版本: v0.1.0"
        return "v0.1.0"
    }
}

# 下载并安装二进制
function Install-Binary {
    param([string]$Version)
    
    Write-Header "下载并安装"
    
    $binary = "claude-code-rust-windows-x86_64.exe"
    $downloadUrl = "https://github.com/$REPO/releases/download/$Version/$binary"
    $exePath = Join-Path $InstallPath "claude-code-rs.exe"
    $tempFile = Join-Path $env:TEMP "claude-code-rs.tmp"
    
    Write-Host "从以下地址下载: " -NoNewline
    Write-Host $downloadUrl -ForegroundColor Cyan
    
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
        
        $progressPreference = 'SilentlyContinue'
        Invoke-WebRequest -Uri $downloadUrl -OutFile $tempFile -UseBasicParsing
        $progressPreference = 'Continue'
        
        Move-Item -Path $tempFile -Destination $exePath -Force
        Write-Success "文件已下载: $exePath"
    } catch {
        Write-Error "下载失败: $_"
        Write-Error "请手动从以下地址下载:"
        Write-Error "https://github.com/$REPO/releases"
        exit 1
    }
}

# 添加到 PATH
function Add-ToSystemPath {
    Write-Header "配置 PATH"
    
    if ($AddToPath) {
        $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
        
        if ($currentPath -notlike "*$InstallPath*") {
            $newPath = "$InstallPath;$currentPath"
            
            try {
                if ($SystemWide) {
                    [Environment]::SetEnvironmentVariable("Path", $newPath, "Machine")
                    Write-Success "已添加到系统 PATH (需要重启)"
                } else {
                    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
                    Write-Success "已添加到用户 PATH"
                }
                
                # 刷新当前会话的 PATH
                $env:Path = "$InstallPath;$env:Path"
            } catch {
                Write-Warning "无法自动添加到 PATH: $_"
                Write-Warning "请手动将以下路径添加到环境变量:"
                Write-Warning $InstallPath
            }
        } else {
            Write-Success "已在 PATH 中"
        }
    } else {
        Write-Host "安装路径: $InstallPath"
        Write-Host ""
        Write-Warning "使用 -AddToPath 参数可自动添加到 PATH"
        Write-Host "示例: Set-ExecutionPolicy -ExecutionPolicy Bypass -Scope Process; iex "&'$PSScriptRoot\install.ps1' -AddToPath"
    }
}

# 验证安装
function Test-Installation {
    Write-Header "验证安装"
    
    $exePath = Join-Path $InstallPath "claude-code-rs.exe"
    
    if (Test-Path $exePath) {
        Write-Success "安装成功!"
        
        Write-Host ""
        Write-Host "版本信息:" -ForegroundColor Green
        try {
            & $exePath --version
        } catch {
            Write-Warning "无法运行可执行文件，请检查依赖"
        }
        
        Write-Host ""
        Write-Host "快速开始:" -ForegroundColor Green
        Write-Host "  $exePath --help       显示帮助信息"
        Write-Host "  $exePath --version    显示版本"
        Write-Host "  $exePath              启动 REPL 模式"
        
        Write-Host ""
        if ($env:Path -notlike "*$InstallPath*") {
            Write-Warning "提示: 请添加 $InstallPath 到 PATH 环境变量"
            Write-Host "或者重新运行: Set-ExecutionPolicy -ExecutionPolicy Bypass -Scope Process; & '.\install.ps1' -AddToPath"
        }
        
        Write-Success "准备好开始使用了!"
    } else {
        Write-Error "验证失败！"
        exit 1
    }
}

# 主函数
function Main {
    Clear-Host
    Write-Header "$APP_NAME 安装程序"
    
    Write-Host ""
    Write-Host "这个脚本将在你的系统上安装 $APP_NAME"
    Write-Host ""
    
    Test-PowerShellVersion
    Test-ExistingInstallation
    Set-InstallationPath
    
    $version = Get-LatestVersion
    Install-Binary -Version $version
    Add-ToSystemPath
    Test-Installation
    
    Write-Header "感谢使用 $APP_NAME!"
    Write-Host ""
}

# 运行主程序
Main
