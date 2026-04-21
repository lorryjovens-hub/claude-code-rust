#!/usr/bin/env pwsh

# Claude Code Rust - Windows Installation Script
# This script installs Claude Code Rust CLI tool on Windows

param(
    [string]$InstallDir = "$env:LOCALAPPDATA\temp\claude-code\install"
)

Write-Host "==========================================="
Write-Host "Claude Code Rust - Windows Installation"
Write-Host "==========================================="
Write-Host

# Check if Rust is installed
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Error: Rust is not installed. Please install Rust first from https://rustup.rs/" -ForegroundColor Red
    exit 1
}

# Check if Git is installed
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Host "Error: Git is not installed. Please install Git first from https://git-scm.com/" -ForegroundColor Red
    exit 1
}

# Set installation directory
$binDir = "$InstallDir\bin"

# Create installation directory
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}
if (-not (Test-Path $binDir)) {
    New-Item -ItemType Directory -Path $binDir -Force | Out-Null
}

Write-Host "Installing Claude Code Rust to: $InstallDir"
Write-Host

# Use local source code
Write-Host "Using local source code..."
$sourceDir = "$PSScriptRoot\.."

# Build project
Write-Host "Building project..."
Set-Location "$sourceDir"
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "Error: Failed to build project" -ForegroundColor Red
    exit 1
}

# Copy executable
Write-Host "Copying executable..."
if (-not (Test-Path $binDir)) {
    New-Item -ItemType Directory -Path $binDir -Force | Out-Null
}
Copy-Item "$sourceDir\target\release\claude-code.exe" "$binDir\claude-code.exe"

# Note about PATH
Write-Host "Note: You may need to add $binDir to your PATH manually to use 'claude-code' command from any terminal."

# Create configuration directory
$configDir = "$InstallDir\config"
if (-not (Test-Path $configDir)) {
    New-Item -ItemType Directory -Path $configDir -Force | Out-Null
}

# Create default config file
$configFile = "$configDir\config.toml"
if (-not (Test-Path $configFile)) {
    @"
[api]
api_key = ""
base_url = "https://api.deepseek.com"

[model]
model = "deepseek-reasoner"

[log]
level = "info"
"@ | Out-File -FilePath $configFile -Force
    Write-Host "Created default configuration file at $configFile"
}

# Test installation
Write-Host "Testing installation..."
$testOutput = & "$binDir\claude-code.exe" --help
if ($LASTEXITCODE -eq 0) {
    Write-Host "==========================================="
    Write-Host "Installation successful!"
    Write-Host "==========================================="
    Write-Host "Executable installed at: $binDir\claude-code.exe"
    Write-Host ""
    Write-Host "To configure API key, run:"
    Write-Host "  $binDir\claude-code.exe config set api_key "your-api-key""
    Write-Host ""
    Write-Host "To test the installation, run:"
    Write-Host "  $binDir\claude-code.exe query --prompt "Hello!""
    Write-Host ""
    Write-Host "Note: For easier access, consider adding $binDir to your PATH."
} else {
    Write-Host "Error: Installation failed. Please check the output above." -ForegroundColor Red
    exit 1
}

# Return to original directory
Set-Location -Path $sourceDir
