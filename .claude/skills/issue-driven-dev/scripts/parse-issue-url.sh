#!/bin/bash
# Parse GitHub Issue URL and extract owner, repo, and issue number.
#
# Usage:
#   parse-issue-url.sh <issue-url>           # JSON output
#   parse-issue-url.sh <issue-url> owner     # Extract owner only
#   parse-issue-url.sh <issue-url> repo      # Extract repo only
#   parse-issue-url.sh <issue-url> number    # Extract issue number only
#
# Examples:
#   parse-issue-url.sh https://github.com/noppomario/uti-dev/issues/64
#   # -> {"owner":"noppomario","repo":"uti-dev","number":"64"}
#
#   parse-issue-url.sh https://github.com/noppomario/uti-dev/issues/64 owner
#   # -> noppomario

set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "ERROR: Issue URL required" >&2
    echo "Usage: parse-issue-url.sh <issue-url> [owner|repo|number]" >&2
    exit 1
fi

URL="$1"
FIELD="${2:-}"

# Parse URL with regex
if [[ "$URL" =~ ^https://github\.com/([^/]+)/([^/]+)/issues/([0-9]+) ]]; then
    OWNER="${BASH_REMATCH[1]}"
    REPO="${BASH_REMATCH[2]}"
    NUMBER="${BASH_REMATCH[3]}"
else
    echo "ERROR: Invalid GitHub Issue URL format" >&2
    echo "Expected: https://github.com/{owner}/{repo}/issues/{number}" >&2
    exit 1
fi

case "$FIELD" in
    owner)
        echo "$OWNER"
        ;;
    repo)
        echo "$REPO"
        ;;
    number)
        echo "$NUMBER"
        ;;
    "")
        echo "{\"owner\":\"$OWNER\",\"repo\":\"$REPO\",\"number\":\"$NUMBER\"}"
        ;;
    *)
        echo "ERROR: Unknown field '$FIELD'. Use: owner, repo, or number" >&2
        exit 1
        ;;
esac
