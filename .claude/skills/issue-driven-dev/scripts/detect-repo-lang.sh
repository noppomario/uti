#!/bin/bash
# Detect language convention for a repository based on context.
#
# Usage:
#   detect-repo-lang.sh <owner/repo> <context> [issue-url]
#
# Context:
#   issue-comment - Detect from issue body (requires issue-url)
#   pr            - Detect from recent PR titles
#   commit        - Detect from recent commit messages
#
# Output:
#   en  - English
#   ja  - Japanese
#
# Examples:
#   detect-repo-lang.sh noppomario/uti pr
#   detect-repo-lang.sh noppomario/uti-dev issue-comment https://github.com/noppomario/uti-dev/issues/64

set -euo pipefail

if [[ $# -lt 2 ]]; then
    echo "ERROR: Repository and context required" >&2
    echo "Usage: detect-repo-lang.sh <owner/repo> <context> [issue-url]" >&2
    exit 1
fi

REPO="$1"
CONTEXT="$2"
ISSUE_URL="${3:-}"

# Check if text contains Japanese characters
has_japanese() {
    echo "$1" | grep -qP '[\p{Hiragana}\p{Katakana}\p{Han}]'
}

# Count lines with Japanese characters
count_japanese_lines() {
    local text="$1"
    local total=0
    local japanese=0

    while IFS= read -r line; do
        if [[ -n "$line" ]]; then
            total=$((total + 1))
            if has_japanese "$line"; then
                japanese=$((japanese + 1))
            fi
        fi
    done <<< "$text"

    if [[ $total -eq 0 ]]; then
        echo "0"
    else
        echo "$((japanese * 100 / total))"
    fi
}

case "$CONTEXT" in
    issue-comment)
        if [[ -z "$ISSUE_URL" ]]; then
            echo "ERROR: issue-url required for issue-comment context" >&2
            exit 1
        fi
        # Get issue body
        TEXT=$(gh issue view "$ISSUE_URL" --json body --jq '.body' 2>/dev/null || echo "")
        ;;
    pr)
        # Get recent PR titles
        TEXT=$(gh pr list -R "$REPO" --limit 10 --json title --jq '.[].title' 2>/dev/null || echo "")
        ;;
    commit)
        # Get recent commit messages
        OWNER=$(echo "$REPO" | cut -d'/' -f1)
        REPO_NAME=$(echo "$REPO" | cut -d'/' -f2)
        TEXT=$(gh api "repos/$OWNER/$REPO_NAME/commits" --jq '.[0:20] | .[].commit.message' 2>/dev/null || echo "")
        ;;
    *)
        echo "ERROR: Unknown context '$CONTEXT'. Use: issue-comment, pr, or commit" >&2
        exit 1
        ;;
esac

if [[ -z "$TEXT" ]]; then
    # Default to English if no text found
    echo "en"
    exit 0
fi

# Calculate percentage of Japanese content
JAPANESE_PCT=$(count_japanese_lines "$TEXT")

if [[ $JAPANESE_PCT -ge 50 ]]; then
    echo "ja"
else
    echo "en"
fi
