
# 添加 Claude Code 到系统 PATH
$installPath = "$env:USERPROFILE\.claude-code\bin"

# 获取当前用户 PATH
$currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")

# 检查是否已经添加
if (-not $currentPath.Contains($installPath)) {
    # 添加到 PATH
    [Environment]::SetEnvironmentVariable("PATH", "$currentPath;$installPath", "User")
    Write-Host "✅ 已添加 $installPath 到用户 PATH"
    Write-Host "⚠️ 请重启终端或重新登录以应用更改"
} else {
    Write-Host "✅ $installPath 已存在于 PATH 中"
}

# 显示当前配置
Write-Host "`n📋 当前 Claude Code 配置："
&amp; "$installPath\claude-code.exe" config show
