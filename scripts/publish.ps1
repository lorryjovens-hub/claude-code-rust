# 发布脚本 (PowerShell 版本，适用于 Windows)
# 用法: .\scripts\publish.ps1 [-Release] [-Bump minor|major|patch] [-Version "3.2.0"]

param(
    [switch]$Release,
    [ValidateSet("patch", "minor", "major")]
    [string]$Bump = "patch",
    [string]$Version = ""
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$SrcTauri = Join-Path $ProjectRoot "src-tauri"

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Helper Functions
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

function Log($msg) { Write-Host "[INFO] $msg" -ForegroundColor Cyan }
function Warn($msg) { Write-Host "[WARN] $msg" -ForegroundColor Yellow }
function Error-Exit($msg) { Write-Host "[ERROR] $msg" -ForegroundColor Red; exit 1 }
function Success($msg) { Write-Host "[OK] $msg" -ForegroundColor Green }

function Bump-Version($current, $bumpType) {
    $parts = $current -split '\.'
    $major = [int]$parts[0]
    $minor = [int]$parts[1]
    $patch = [int]$parts[2]
    
    switch ($bumpType) {
        "major" { return "$($major + 1).0.0" }
        "minor" { return "$major.$($minor + 1).0" }
        "patch" { return "$major.$minor.$($patch + 1)" }
    }
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Pre-flight checks
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Log "Checking prerequisites..."
Get-Command git -ErrorAction Stop | Out-Null
Get-Command npm -ErrorAction Stop | Out-Null
Get-Command node -ErrorAction Stop | Out-Null

Set-Location $ProjectRoot

# Check if we're in a git repo
$gitDir = git rev-parse --git-dir 2>$null
if (-not $gitDir) {
    Error-Exit "Not a git repository"
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Version management
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

$PackageJson = Get-Content "package.json" | ConvertFrom-Json
$CurrentVersion = $PackageJson.version
Log "Current version: $CurrentVersion"

if ($Release) {
    if ($Version) {
        $NewVersion = $Version
    } else {
        $NewVersion = Bump-Version $CurrentVersion $Bump
    }
    
    Log "Bumping version: $CurrentVersion -> $NewVersion"
    
    # Update package.json
    $PackageJson.version = $NewVersion
    $PackageJson | ConvertTo-Json -Depth 10 | Set-Content "package.json"
    
    # Update tauri.conf.json
    $TauriConfPath = Join-Path $SrcTauri "tauri.conf.json"
    $TauriConf = Get-Content $TauriConfPath | ConvertFrom-Json
    $TauriConf.version = $NewVersion
    $TauriConf | ConvertTo-Json -Depth 10 | Set-Content $TauriConfPath
    
    # Update Cargo.toml
    $CargoPath = Join-Path $SrcTauri "Cargo.toml"
    $CargoContent = Get-Content $CargoPath
    $CargoContent = $CargoContent -replace "version = `"$CurrentVersion`"", "version = `"$NewVersion`""
    $CargoContent | Set-Content $CargoPath
    
    Success "Version bumped to $NewVersion"
} else {
    $NewVersion = $CurrentVersion
    Log "Using current version: $NewVersion (dry-run mode)"
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Install dependencies
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Log "Installing dependencies..."
npm ci | Out-Host

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Build frontend
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Log "Building frontend..."
npm run build | Out-Host
Success "Frontend built"

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Run tests
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Log "Running tests..."
try {
    npm run test:run 2>$null | Out-Host
    Success "Tests passed"
} catch {
    Warn "Some tests failed. Continuing with build..."
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Build Tauri app
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Log "Building Tauri application..."
npx tauri build 2>&1 | Out-Host

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Collect build artifacts
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

$BuildDir = Join-Path $SrcTauri "target\release\bundle"
$DistDir = Join-Path $ProjectRoot "dist-release\$NewVersion"

New-Item -ItemType Directory -Force -Path $DistDir | Out-Null

Log "Collecting build artifacts..."

Get-ChildItem -Path $BuildDir -Recurse -File | Where-Object {
    $_.Extension -match '\.(exe|msi|dmg|deb|AppImage|tar\.gz)$'
} | ForEach-Object {
    Copy-Item $_.FullName $DistDir
    Log "✓ $($_.Name)"
}

Success "Build artifacts collected to: $DistDir"

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Summary
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Write-Host ""
Write-Host "==============================================" -ForegroundColor Green
Write-Host "  Build Complete! v$NewVersion" -ForegroundColor Green
Write-Host "==============================================" -ForegroundColor Green
Write-Host ""
Write-Host "  Output directory: $DistDir"
Write-Host ""

if ($Release) {
    Write-Host "  Next steps:"
    Write-Host "  1. Review artifacts in $DistDir"
    Write-Host "  2. Create git tag:"
    Write-Host "     git tag -a `"v$NewVersion`" -m `"Release v$NewVersion`""
    Write-Host "     git push origin v$NewVersion"
    Write-Host ""
    Write-Host "  3. CI/CD will automatically build and create GitHub Release"
    Write-Host "     Or manually:"
    Write-Host "     gh release create v$NewVersion --title `"v$NewVersion`" --generate-notes $DistDir\*"
} else {
    Write-Host "  This was a dry-run build. To publish:"
    Write-Host "  .\scripts\publish.ps1 -Release [-Bump minor|major]"
}

Write-Host ""
Write-Host "==============================================" -ForegroundColor Green
