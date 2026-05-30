param(
    [switch]$SkipBuild,
    [switch]$SkipTag
)

$ErrorActionPreference = "Stop"

$pkg = Get-Content "package.json" -Raw | ConvertFrom-Json
$version = $pkg.version

Write-Host "[release] Starting release for v$version" -ForegroundColor Cyan
Write-Host ""

if (-not $SkipTag) {
    $existingTag = git tag -l "v$version" 2>$null
    if ($existingTag) {
        Write-Host "[release] Tag v$version already exists" -ForegroundColor Yellow
    } else {
        Write-Host "[release] Creating git tag v$version..." -ForegroundColor Cyan
        git tag -a "v$version" -m "v$version"
        Write-Host "[release] Tag v$version created" -ForegroundColor Green
    }
}

if (-not $SkipBuild) {
    Write-Host "[release] Building release package..." -ForegroundColor Cyan
    npx tauri build 2>&1 | ForEach-Object { Write-Host "  $_" }
    
    $exePath = "src-tauri\target\release\claude-desktop-tauri.exe"
    $msiPath = Get-ChildItem "src-tauri\target\release\bundle\msi\*.msi" -ErrorAction SilentlyContinue | Select-Object -First 1
    
    if (Test-Path $exePath) {
        $size = (Get-Item $exePath).Length / 1MB
        Write-Host "[release] EXE: $exePath ($([math]::Round($size, 1)) MB)" -ForegroundColor Green
    }
    if ($msiPath) {
        $size = $msiPath.Length / 1MB
        Write-Host "[release] MSI: $($msiPath.FullName) ($([math]::Round($size, 1)) MB)" -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "[release] Release v$version complete!" -ForegroundColor Green
Write-Host "[release] To push to remote:" -ForegroundColor Yellow
Write-Host "  git push origin main --tags" -ForegroundColor White
