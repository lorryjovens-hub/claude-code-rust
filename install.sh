#!/usr/bin/env bash
# Claude Code Rust - 通用安装程序
# 自动检测操作系统并运行相应的安装脚本

set -e

REPO_URL="https://github.com/lorryjovens-hub/claude-code-rust"
INSTALL_URL="$REPO_URL/raw/master"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印函数
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

# 检测已有版本
check_existing() {
    if command -v claude-code-rs &> /dev/null; then
        EXISTING_VERSION=$(claude-code-rs --version 2>/dev/null | grep -oP 'v\d+\.\d+\.\d+' || echo "unknown")
        print_warning "已安装版本: $EXISTING_VERSION"
        echo -e "${YELLOW}继续安装将覆盖现有版本${NC}"
        read -p "继续? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_error "取消安装"
            exit 1
        fi
    fi
}

# 检测依赖
check_dependencies() {
    echo ""
    print_header "检查依赖"
    
    if ! command -v curl &> /dev/null; then
        print_error "需要安装 curl"
        exit 1
    fi
    print_success "curl 已安装"
}

# 选择安装路径
choose_install_path() {
    echo ""
    print_header "选择安装路径"
    
    # 检查常见安装路径
    INSTALL_PATHS=()
    
    if [[ -w "/usr/local/bin" ]]; then
        INSTALL_PATHS+=("/usr/local/bin")
    fi
    
    if [[ -w "$HOME/.local/bin" ]]; then
        INSTALL_PATHS+=("$HOME/.local/bin")
    fi
    
    INSTALL_PATHS+=("$HOME/bin")
    INSTALL_PATHS+=("$(pwd)")
    
    if [[ ${#INSTALL_PATHS[@]} -eq 1 ]]; then
        INSTALL_PATH="${INSTALL_PATHS[0]}"
    else
        echo "选择安装路径:"
        for i in "${!INSTALL_PATHS[@]}"; do
            echo "  $((i+1))) ${INSTALL_PATHS[$i]}"
        done
        echo "  c) 自定义路径"
        
        read -p "选择 (1-${#INSTALL_PATHS[@]} 或 c): " choice
        
        if [[ $choice == "c" || $choice == "C" ]]; then
            read -p "输入安装路径: " INSTALL_PATH
        else
            INSTALL_PATH="${INSTALL_PATHS[$((choice-1))]}"
        fi
    fi
    
    mkdir -p "$INSTALL_PATH"
    print_success "安装路径: $INSTALL_PATH"
}

# 获取最新版本
get_latest_version() {
    echo ""
    print_header "获取最新版本"
    
    LATEST_VERSION=$(curl -s https://api.github.com/repos/lorryjovens-hub/claude-code-rust/releases/latest \
        | grep '"tag_name"' \
        | cut -d'"' -f4)
    
    if [[ -z "$LATEST_VERSION" ]]; then
        print_warning "无法获取最新版本，使用默认版本: v0.1.0"
        LATEST_VERSION="v0.1.0"
    fi
    
    print_success "最新版本: $LATEST_VERSION"
}

# 下载二进制
download_binary() {
    echo ""
    print_header "下载二进制文件"
    
    BINARY_NAME="claude-code-rust-${OS}-${ARCH}"
    DOWNLOAD_URL="https://github.com/lorryjovens-hub/claude-code-rust/releases/download/${LATEST_VERSION}/${BINARY_NAME}"
    TEMP_FILE="/tmp/${BINARY_NAME}.tmp"
    
    print_warning "下载中: $BINARY_NAME"
    
    if ! curl -fsSL -o "$TEMP_FILE" "$DOWNLOAD_URL"; then
        print_error "下载失败: $DOWNLOAD_URL"
        print_error "请手动从以下地址下载:"
        print_error "https://github.com/lorryjovens-hub/claude-code-rust/releases"
        exit 1
    fi
    
    print_success "下载完成"
}

# 安装二进制
install_binary() {
    echo ""
    print_header "安装二进制"
    
    INSTALL_FILE="$INSTALL_PATH/claude-code-rs"
    
    mv "$TEMP_FILE" "$INSTALL_FILE"
    chmod +x "$INSTALL_FILE"
    
    print_success "已安装: $INSTALL_FILE"
}

# 检查 PATH
check_path() {
    echo ""
    print_header "检查 PATH 配置"
    
    if [[ "$INSTALL_PATH" != "/usr/local/bin" ]] && [[ "$INSTALL_PATH" != "/usr/bin" ]]; then
        if ! echo "$PATH" | grep -q "$INSTALL_PATH"; then
            print_warning "安装路径不在 PATH 中"
            echo ""
            echo "请将以下行添加到你的 shell 配置文件中:"
            echo "  export PATH=\"$INSTALL_PATH:\$PATH\""
            echo ""
            echo "配置文件位置:"
            if [[ -f "$HOME/.bashrc" ]]; then
                echo "  1) Bash: $HOME/.bashrc"
            fi
            if [[ -f "$HOME/.zshrc" ]]; then
                echo "  2) Zsh: $HOME/.zshrc"
            fi
            echo ""
            read -p "现在添加到 ~/.bashrc? (y/N) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                echo "export PATH=\"$INSTALL_PATH:\$PATH\"" >> ~/.bashrc
                print_success "已添加到 ~/.bashrc"
            fi
        fi
    fi
}

# 验证安装
verify_installation() {
    echo ""
    print_header "验证安装"
    
    if command -v claude-code-rs &> /dev/null; then
        VERSION=$(claude-code-rs --version)
        print_success "安装成功!"
        echo ""
        echo "版本信息:"
        echo "  $VERSION"
        echo ""
        echo "快速开始:"
        echo "  claude-code-rs --help       显示帮助信息"
        echo "  claude-code-rs --version    显示版本"
        echo "  claude-code-rs              启动 REPL 模式"
        echo ""
        print_success "准备好开始使用了!"
    else
        print_error "验证失败!"
        print_error "请手动将 $INSTALL_FILE 添加到 PATH"
        exit 1
    fi
}

# 主流程
main() {
    clear
    print_header "Claude Code Rust 安装程序"
    echo ""
    echo "这个脚本将在你的系统上安装 Claude Code Rust"
    echo ""
    
    detect_os
    check_existing
    check_dependencies
    choose_install_path
    get_latest_version
    download_binary
    install_binary
    check_path
    verify_installation
    
    echo ""
    print_header "感谢使用 Claude Code Rust!"
    echo ""
}

# 处理中断
trap 'print_error "中止安装"; rm -f /tmp/claude-code-rust*.tmp; exit 1' INT TERM

# 运行主程序
main
