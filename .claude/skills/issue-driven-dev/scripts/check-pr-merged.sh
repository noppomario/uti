#!/bin/bash
# Check if a GitHub PR is merged.
#
# Usage:
#   check-pr-merged.sh <pr-url>
#
# Output:
#   merged    - PR has been merged
#   open      - PR is still open
#   closed    - PR is closed but not merged
#   not_found - PR not found
#
# Example:
#   check-pr-merged.sh https://github.com/noppomario/uti/pull/100

set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "ERROR: PR URL required" >&2
    echo "Usage: check-pr-merged.sh <pr-url>" >&2
    exit 1
fi

PR_URL="$1"

# Get PR state and mergedAt
PR_INFO=$(gh pr view "$PR_URL" --json state,mergedAt 2>/dev/null || echo '{"error":true}')

if echo "$PR_INFO" | jq -e '.error' >/dev/null 2>&1; then
    echo "not_found"
    exit 0
fi

STATE=$(echo "$PR_INFO" | jq -r '.state')
MERGED_AT=$(echo "$PR_INFO" | jq -r '.mergedAt')

if [[ "$MERGED_AT" != "null" && -n "$MERGED_AT" ]]; then
    echo "merged"
elif [[ "$STATE" == "OPEN" ]]; then
    echo "open"
else
    echo "closed"
fi
