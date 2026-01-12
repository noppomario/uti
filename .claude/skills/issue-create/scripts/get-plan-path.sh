#!/bin/bash
# Get the most recently modified plan file path.
#
# Usage:
#   get-plan-path.sh
#
# Output:
#   /home/user/.claude/plans/xxx.md
#
# Returns the most recently modified .md file in ~/.claude/plans/

set -euo pipefail

PLANS_DIR="$HOME/.claude/plans"

if [[ ! -d "$PLANS_DIR" ]]; then
    echo "ERROR: Plans directory not found: $PLANS_DIR" >&2
    exit 1
fi

# Find most recently modified .md file
LATEST=$(ls -t "$PLANS_DIR"/*.md 2>/dev/null | head -1)

if [[ -z "$LATEST" ]]; then
    echo "ERROR: No plan files found in $PLANS_DIR" >&2
    exit 1
fi

echo "$LATEST"
