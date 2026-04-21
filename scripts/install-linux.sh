#!/bin/bash

# Claude Code Rust - Linux Installation Script
# This script installs Claude Code Rust CLI tool on Linux

# Default installation directory
INSTALL_DIR="$HOME/.claude-code"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --install-dir)
            INSTALL_DIR="$2"
            shift # past argument
            shift # past value
            ;;
        *)
            echo "Unknown argument: $1"
            exit 1
            ;;
    esac
done

echo "==========================================="
echo "Claude Code Rust - Linux Installation"
echo "==========================================="
echo

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust is not installed. Please install Rust first from https://rustup.rs/"
    exit 1
fi

# Check if Git is installed
if ! command -v git &> /dev/null; then
    echo "Error: Git is not installed. Please install Git first."
    exit 1
fi

# Set installation directory
BIN_DIR="$INSTALL_DIR/bin"

echo "Installing Claude Code Rust to: $INSTALL_DIR"
echo

# Create directories
mkdir -p "$INSTALL_DIR"
mkdir -p "$BIN_DIR"

# Clone repository
echo "Cloning repository..."
if [ -d "$INSTALL_DIR/claude-code-rust" ]; then
    rm -rf "$INSTALL_DIR/claude-code-rust"
fi

git clone https://github.com/lorryjovens-hub/claude-code-rust "$INSTALL_DIR/claude-code-rust"
if [ $? -ne 0 ]; then
    echo "Error: Failed to clone repository"
    exit 1
fi

# Build project
echo "Building project..."
cd "$INSTALL_DIR/claude-code-rust"
cargo build --release
if [ $? -ne 0 ]; then
    echo "Error: Failed to build project"
    exit 1
fi

# Copy executable
echo "Copying executable..."
cp "$INSTALL_DIR/claude-code-rust/target/release/claude-code" "$BIN_DIR/claude-code"
chmod +x "$BIN_DIR/claude-code"

# Add to PATH
echo "Adding to PATH..."
if ! grep -q "$BIN_DIR" ~/.bashrc && ! grep -q "$BIN_DIR" ~/.zshrc; then
    if [ -f ~/.bashrc ]; then
        echo "export PATH=\"$BIN_DIR:\$PATH\"" >> ~/.bashrc
        echo "Added $BIN_DIR to ~/.bashrc"
    elif [ -f ~/.zshrc ]; then
        echo "export PATH=\"$BIN_DIR:\$PATH\"" >> ~/.zshrc
        echo "Added $BIN_DIR to ~/.zshrc"
    else
        echo "Warning: Could not find ~/.bashrc or ~/.zshrc. Please add $BIN_DIR to your PATH manually."
    fi
else
    echo "$BIN_DIR is already in PATH"
fi

# Create configuration directory
CONFIG_DIR="$HOME/.config/claude-code"
mkdir -p "$CONFIG_DIR"

# Create default config file
CONFIG_FILE="$CONFIG_DIR/config.toml"
if [ ! -f "$CONFIG_FILE" ]; then
    cat > "$CONFIG_FILE" << EOF
[api]
api_key = ""
base_url = "https://api.deepseek.com"

[model]
model = "deepseek-reasoner"

[log]
level = "info"
EOF
    echo "Created default configuration file at $CONFIG_FILE"
fi

# Test installation
echo "Testing installation..."
"$BIN_DIR/claude-code" --help
if [ $? -eq 0 ]; then
    echo "==========================================="
    echo "Installation successful!"
    echo "==========================================="
    echo "You can now use 'claude-code' command from any terminal."
    echo ""
    echo "To configure API key, run:"
    echo "  claude-code config set api_key \"your-api-key\""
    echo ""
    echo "To test the installation, run:"
    echo "  claude-code query --prompt \"Hello!\""
    echo ""
    echo "Note: You may need to restart your terminal for the PATH changes to take effect."
else
    echo "Error: Installation failed. Please check the output above."
    exit 1
fi

# Return to original directory
cd - > /dev/null
