#!/bin/bash
# Claude Code Rust 安装脚本 - 完整版 (Linux/macOS)
# 用法: bash install-unix.sh

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_header() {
    echo -e "${BLUE}═══════════════════════════════════════${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}═══════════════════════════════════════${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# 检测操作系统
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        OS="linux"
        if [[ $(uname -m) == "aarch64" ]]; then
            ARCH="aarch64"
        else
            ARCH="x86_64"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS="macos"
        if [[ $(uname -m) == "arm64" ]]; then
            ARCH="aarch64"
        else
            ARCH="x86_64"
        fi
    else
        print_error "不支持的操作系统: $OSTYPE"
        exit 1
    fi
    
    print_success "检测到操作系统: $OS ($ARCH)"
}

# 选择安装路径
choose_install_path() {
    echo ""
    print_header "选择安装路径"
    
    INSTALL_PATHS=()
    [[ -w "/usr/local/bin" ]] && INSTALL_PATHS+=("/usr/local/bin")
    [[ -w "$HOME/.local/bin" ]] && INSTALL_PATHS+=("$HOME/.local/bin")
    INSTALL_PATHS+=("$HOME/bin")
    
    if [[ ${#INSTALL_PATHS[@]} -eq 1 ]]; then
        INSTALL_PATH="${INSTALL_PATHS[0]}"
    else
        echo "选择安装路径:"
        for i in "${!INSTALL_PATHS[@]}"; do
            echo "  $((i+1))) ${INSTALL_PATHS[$i]}"
        done
        
        read -p "选择 (1-${#INSTALL_PATHS[@]}): " choice
        INSTALL_PATH="${INSTALL_PATHS[$((choice-1))]}"
    fi
    
    mkdir -p "$INSTALL_PATH"
    print_success "安装路径: $INSTALL_PATH"
}

# 获取最新版本
get_latest_version() {
    echo ""
    print_header "获取最新版本"
    
    LATEST_VERSION=$(curl -s https://api.github.com/repos/lorryjovens-hub/claude-code-rust/releases/latest | grep '"tag_name"' | cut -d'"' -f4)
    [[ -z "$LATEST_VERSION" ]] && LATEST_VERSION="v0.1.0"
    print_success "最新版本: $LATEST_VERSION"
}

# 下载并安装
download_and_install() {
    echo ""
    print_header "下载并安装"
    
    BINARY_NAME="claude-code-rust-${OS}-${ARCH}"
    DOWNLOAD_URL="https://github.com/lorryjovens-hub/claude-code-rust/releases/download/${LATEST_VERSION}/${BINARY_NAME}"
    TEMP_FILE="/tmp/${BINARY_NAME}.tmp"
    INSTALL_FILE="$INSTALL_PATH/claude-code-rs"
    
    print_warning "下载中: $BINARY_NAME"
    
    if ! curl -fsSL -o "$TEMP_FILE" "$DOWNLOAD_URL"; then
        print_error "下载失败"
        exit 1
    fi
    
    mv "$TEMP_FILE" "$INSTALL_FILE"
    chmod +x "$INSTALL_FILE"
    print_success "已安装: $INSTALL_FILE"
}

# 验证安装
verify_installation() {
    echo ""
    print_header "验证安装"
    
    if command -v claude-code-rs &> /dev/null; then
        VERSION=$(claude-code-rs --version)
        print_success "安装成功!"
        
        echo ""
        echo "版本信息: $VERSION"
        echo ""
        echo "快速命令:"
        echo "  claude-code-rs --help"
        echo "  claude-code-rs --version"
        echo "  claude-code-rs"
        echo ""
        print_success "准备好开始使用了!"
    else
        print_error "验证失败!"
        exit 1
    fi
}

main() {
    clear
    print_header "Claude Code Rust 安装程序"
    echo ""
    
    detect_os
    choose_install_path
    get_latest_version
    download_and_install
    verify_installation
}

trap 'print_error "中止安装"; rm -f /tmp/claude-code-rust*.tmp; exit 1' INT TERM

main
