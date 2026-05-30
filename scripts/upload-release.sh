#!/usr/bin/env bash
# Upload build artifacts to GitHub Release
# Prerequisites: gh CLI installed, artifacts in ./dist-release/VERSION/

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log() { echo -e "${BLUE}[INFO]${NC} $*"; }
error() { echo -e "${RED}[ERROR]${NC} $*"; exit 1; }
success() { echo -e "${GREEN}[OK]${NC} $*"; }

# Check prerequisites
command -v gh &>/dev/null || error "GitHub CLI (gh) is required"

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    # Get latest version from dist-release
    VERSION=$(ls -d dist-release/*/ 2>/dev/null | tail -1 | tr -d '/')
    VERSION=$(basename "$VERSION")
    
    if [ -z "$VERSION" ]; then
        error "No release found. Please specify version: $0 v3.2.0"
    fi
    
    log "Using latest release: $VERSION"
fi

# Remove 'v' prefix if present for directory lookup
VERSION_NUM="${VERSION#v}"
DIST_DIR="dist-release/$VERSION_NUM"

if [ ! -d "$DIST_DIR" ]; then
    error "Directory not found: $DIST_DIR"
fi

# Check if release exists
if gh release view "$VERSION" &>/dev/null; then
    log "Release $VERSION already exists. Uploading missing artifacts..."
else
    log "Creating release $VERSION..."
    
    NOTES_FILE="$DIST_DIR/RELEASE_NOTES.md"
    if [ -f "$NOTES_FILE" ]; then
        gh release create "$VERSION" \
            --title "$VERSION" \
            --notes-file "$NOTES_FILE" \
            --draft \
            "$DIST_DIR"/*
    else
        gh release create "$VERSION" \
            --title "$VERSION" \
            --generate-notes \
            --draft \
            "$DIST_DIR"/*
    fi
fi

success "Release $VERSION uploaded to GitHub!"
log "View release at: $(gh release view $VERSION --json url -q .url)"
log "Don't forget to publish the draft release!"
