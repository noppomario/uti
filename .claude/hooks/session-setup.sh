#!/bin/bash
#===============================================================================
# Claude Code on Web: Session setup script
#
# DO NOT MODIFY this file unless fixing bugs.
# Repository-specific settings go in session-config.sh
#===============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_PREFIX="[session-setup]"

log() { echo "$LOG_PREFIX $1" >&2; }

#---------------------------------------
# Load configuration
#---------------------------------------
CONFIG_FILE="$SCRIPT_DIR/session-config.sh"
if [ -f "$CONFIG_FILE" ]; then
    source "$CONFIG_FILE"
else
    log "WARNING: $CONFIG_FILE not found, using defaults"
    CONFIG_DEFAULT_BRANCH=""
    CONFIG_INSTALL_GH=true
fi

#---------------------------------------
# Environment check
#---------------------------------------
if [ "$CLAUDE_CODE_REMOTE" != "true" ]; then
    log "Not a remote session, skipping setup"
    exit 0
fi

log "=== Session setup starting ==="

#---------------------------------------
# Module: Git branch setup
#---------------------------------------
setup_branch() {
    local target_branch="$1"

    if [ -z "$target_branch" ]; then
        log "Branch switching disabled"
        return 0
    fi

    local current_branch=$(git branch --show-current 2>/dev/null || echo "")

    if [ -z "$current_branch" ]; then
        log "Not a git repository, skipping branch setup"
        return 0
    fi

    if [ "$current_branch" = "$target_branch" ]; then
        log "Already on $target_branch branch"
        return 0
    fi

    # Check for uncommitted changes
    if ! git diff --quiet 2>/dev/null || ! git diff --cached --quiet 2>/dev/null; then
        log "WARNING: Uncommitted changes detected, staying on $current_branch"
        return 0
    fi

    log "Switching from $current_branch to $target_branch..."
    if git checkout "$target_branch" 2>/dev/null; then
        log "Switched to $target_branch"
    else
        log "WARNING: Failed to switch to $target_branch (branch may not exist)"
    fi
}

#---------------------------------------
# Module: gh CLI installation
#---------------------------------------
setup_gh() {
    if [ "$CONFIG_INSTALL_GH" != "true" ]; then
        log "gh CLI installation disabled"
        return 0
    fi

    if command -v gh &>/dev/null; then
        log "gh CLI available: $(gh --version | head -1)"
        return 0
    fi

    local LOCAL_BIN="$HOME/.local/bin"

    if [ -x "$LOCAL_BIN/gh" ]; then
        log "gh found in $LOCAL_BIN"
        persist_path "$LOCAL_BIN"
        return 0
    fi

    log "Installing gh CLI..."

    local TEMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TEMP_DIR"' RETURN

    local ARCH=$(uname -m)
    local GH_ARCH
    case "$ARCH" in
        x86_64) GH_ARCH="amd64" ;;
        aarch64|arm64) GH_ARCH="arm64" ;;
        *) log "Unsupported architecture: $ARCH"; return 0 ;;
    esac

    mkdir -p "$LOCAL_BIN"

    local GH_VERSION=$(curl -sL --connect-timeout 10 \
        https://api.github.com/repos/cli/cli/releases/latest | \
        grep '"tag_name"' | sed -E 's/.*"v([^"]+)".*/\1/')

    if [ -z "$GH_VERSION" ]; then
        log "WARNING: Failed to get gh version"
        return 0
    fi

    local GH_URL="https://github.com/cli/cli/releases/download/v${GH_VERSION}/gh_${GH_VERSION}_linux_${GH_ARCH}.tar.gz"

    log "Downloading gh v${GH_VERSION}..."

    if curl -sL --connect-timeout 10 --max-time 60 "$GH_URL" -o "$TEMP_DIR/gh.tar.gz" && \
       tar -xzf "$TEMP_DIR/gh.tar.gz" -C "$TEMP_DIR" && \
       mv "$TEMP_DIR/gh_${GH_VERSION}_linux_${GH_ARCH}/bin/gh" "$LOCAL_BIN/gh" && \
       chmod +x "$LOCAL_BIN/gh"; then
        persist_path "$LOCAL_BIN"
        log "gh installed: $($LOCAL_BIN/gh --version | head -1)"
    else
        log "WARNING: Failed to install gh CLI"
    fi
}

#---------------------------------------
# Utility: Persist PATH
#---------------------------------------
persist_path() {
    local dir="$1"
    export PATH="$dir:$PATH"

    if [ -n "$CLAUDE_ENV_FILE" ]; then
        if ! grep -q "$dir" "$CLAUDE_ENV_FILE" 2>/dev/null; then
            echo "export PATH=\"$dir:\$PATH\"" >> "$CLAUDE_ENV_FILE"
            log "PATH persisted to CLAUDE_ENV_FILE"
        fi
    fi
}

#---------------------------------------
# Main execution
#---------------------------------------
setup_branch "$CONFIG_DEFAULT_BRANCH"
setup_gh

# Run custom setup if defined
if type custom_setup &>/dev/null; then
    log "Running custom setup..."
    custom_setup
fi

log "=== Session setup complete ==="
exit 0
