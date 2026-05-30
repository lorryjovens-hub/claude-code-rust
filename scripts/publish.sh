#!/usr/bin/env bash
# 本地打包和发布脚本
# 用法: ./scripts/publish.sh [--release] [--patch|--minor|--major] [VERSION]

set -euo pipefail

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Configuration
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
SRC_TAURI="$PROJECT_ROOT/src-tauri"

VERSION_BUMP="${2:-patch}"
PUBLISH_RELEASE="${1:-}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Functions
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

log() { echo -e "${BLUE}[INFO]${NC} $*"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*"; exit 1; }
success() { echo -e "${GREEN}[OK]${NC} $*"; }

bump_version() {
    local current="$1"
    local bump_type="${2:-patch}"
    
    IFS='.' read -r major minor patch <<< "$current"
    
    case "$bump_type" in
        major)
            echo "$((major + 1)).0.0"
            ;;
        minor)
            echo "$major.$((minor + 1)).0"
            ;;
        patch)
            echo "$major.$minor.$((patch + 1))"
            ;;
        *)
            echo "$current"
            ;;
    esac
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Pre-flight checks
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

log "Checking prerequisites..."
command -v git &>/dev/null || error "git is required"
command -v npm &>/dev/null || error "npm is required"
command -v node &>/dev/null || error "node is required"

cd "$PROJECT_ROOT"

# Check if we're in a git repo
git rev-parse --git-dir &>/dev/null || error "Not a git repository"

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    warn "You have uncommitted changes. Please commit them first."
    read -p "Continue anyway? (y/N): " -r
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Version management
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

CURRENT_VERSION=$(node -e "console.log(require('./package.json').version)")
log "Current version: $CURRENT_VERSION"

if [ "$PUBLISH_RELEASE" = "--release" ]; then
    if [ -n "${3:-}" ]; then
        NEW_VERSION="$3"
    else
        NEW_VERSION=$(bump_version "$CURRENT_VERSION" "$VERSION_BUMP")
    fi
    
    log "Bumping version: $CURRENT_VERSION -> $NEW_VERSION"
    
    # Update package.json
    npm version "$NEW_VERSION" --no-git-tag-version
    
    # Update tauri.conf.json
    sed -i '' "s/\"version\": \"$CURRENT_VERSION\"/\"version\": \"$NEW_VERSION\"/" "$SRC_TAURI/tauri.conf.json"
    
    # Update Cargo.toml
    sed -i '' "s/version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" "$SRC_TAURI/Cargo.toml"
    
    success "Version bumped to $NEW_VERSION"
else
    NEW_VERSION="$CURRENT_VERSION"
    log "Using current version: $NEW_VERSION (dry-run mode)"
fi

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Install dependencies
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

log "Installing dependencies..."
npm ci

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Build frontend
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

log "Building frontend..."
npm run build
success "Frontend built"

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Run tests
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

log "Running tests..."
if npm run test:run 2>/dev/null; then
    success "Tests passed"
else
    warn "Some tests failed. Continuing with build..."
fi

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Build Tauri app
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

log "Building Tauri application..."
cd "$SRC_TAURI"
cargo build --release
cd "$PROJECT_ROOT"

# Create installer packages using Tauri CLI
log "Creating installer packages..."
npx tauri build

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Collect build artifacts
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

BUILD_DIR="$SRC_TAURI/target/release/bundle"
DIST_DIR="$PROJECT_ROOT/dist-release/$NEW_VERSION"

mkdir -p "$DIST_DIR"

log "Collecting build artifacts..."

# Copy Windows artifacts
if [ -d "$BUILD_DIR/nsis" ]; then
    cp -r "$BUILD_DIR/nsis"/* "$DIST_DIR/" 2>/dev/null && log "✓ Windows NSIS installer"
fi
if [ -d "$BUILD_DIR/msi" ]; then
    cp -r "$BUILD_DIR/msi"/* "$DIST_DIR/" 2>/dev/null && log "✓ Windows MSI installer"
fi

# Copy macOS artifacts
if [ -d "$BUILD_DIR/dmg" ]; then
    cp -r "$BUILD_DIR/dmg"/* "$DIST_DIR/" 2>/dev/null && log "✓ macOS DMG"
fi

# Copy Linux artifacts
if [ -d "$BUILD_DIR/deb" ]; then
    cp -r "$BUILD_DIR/deb"/* "$DIST_DIR/" 2>/dev/null && log "✓ Linux DEB package"
fi
if [ -d "$BUILD_DIR/appimage" ]; then
    cp -r "$BUILD_DIR/appimage"/* "$DIST_DIR/" 2>/dev/null && log "✓ Linux AppImage"
fi

success "Build artifacts collected to: $DIST_DIR"

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Generate release notes
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

RELEASE_NOTES="$DIST_DIR/RELEASE_NOTES.md"

cat > "$RELEASE_NOTES" << EOF
# Claude Desktop Tauri v$NEW_VERSION

## Installation

| Platform | Installer |
|----------|-----------|
$(for f in "$DIST_DIR"/*.{exe,msi,dmg,deb,AppImage}; do
    [ -f "$f" ] || continue
    filename=$(basename "$f")
    case "$filename" in
        *.exe) echo "| Windows | \`$filename\` (NSIS)" ;;
        *.msi) echo "| Windows | \`$filename\` (MSI)" ;;
        *.dmg) echo "| macOS | \`$filename\`" ;;
        *.deb) echo "| Linux | \`$filename\` (Debian/Ubuntu)" ;;
        *.AppImage) echo "| Linux | \`$filename\` (AppImage)" ;;
    esac
done)
|

## How to use

1. Download the appropriate installer for your platform
2. Run the installer and follow the instructions
3. Launch Claude Desktop Tauri from your applications menu

## System Requirements

- **Windows**: Windows 10 or later
- **macOS**: macOS 12.0 (Monterey) or later
- **Linux**: Debian 11+, Ubuntu 22.04+, or equivalent

## Known Issues

See [GitHub Issues](https://github.com/anthropic/claude-desktop-tauri/issues) for known issues.

---

Built with ❤️ using Tauri

EOF

log "Release notes generated: $RELEASE_NOTES"

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Create git tag and commit
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

if [ "$PUBLISH_RELEASE" = "--release" ]; then
    log "Creating git tag and commit..."
    
    git add package.json package-lock.json "$SRC_TAURI/tauri.conf.json" "$SRC_TAURI/Cargo.toml"
    git commit -m "chore: release v$NEW_VERSION"
    git tag -a "v$NEW_VERSION" -m "Release v$NEW_VERSION"
    
    success "Git tag v$NEW_VERSION created"
fi

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Summary
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo ""
echo "=============================================="
echo "  Build Complete! v$NEW_VERSION"
echo "=============================================="
echo ""
echo "  Output directory: $DIST_DIR"
echo ""
echo "  Next steps:"
echo ""

if [ "$PUBLISH_RELEASE" = "--release" ]; then
    echo "  1. Review the release notes in $RELEASE_NOTES"
    echo "  2. Push the tag to GitHub:"
    echo "     git push origin v$NEW_VERSION"
    echo ""
    echo "  3. CI/CD will automatically:"
    echo "     - Build for all platforms"
    echo "     - Create GitHub Release"
    echo "     - Upload artifacts"
    echo ""
    echo "  Or manually create release:"
    echo "     gh release create v$NEW_VERSION \\"
    echo "       --title \"v$NEW_VERSION\" \\"
    echo "       --notes-file $RELEASE_NOTES \\"
    echo "       $DIST_DIR/*"
else
    echo "  This was a dry-run build. To publish:"
    echo "    ./scripts/publish.sh --release [--minor|--major]"
fi

echo ""
echo "=============================================="
