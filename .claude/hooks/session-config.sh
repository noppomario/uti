#!/bin/bash
#===============================================================================
# Repository-specific configuration for Claude Code session setup
#
# This file should be modified per repository.
# session-setup.sh should NOT be modified unless fixing bugs.
#===============================================================================

# Whether to install gh CLI if not available
CONFIG_INSTALL_GH=true

# Additional setup commands (optional)
# Called at the end of setup if defined
custom_setup() {
    : # Add custom commands here, e.g.:
    # bun install
    # source ~/.nvm/nvm.sh && nvm use 20
}
