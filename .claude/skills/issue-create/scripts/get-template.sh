#!/bin/bash
# Fetch GitHub Issue template and detect its language.
#
# Usage:
#   get-template.sh <owner> <repo> <template-name>
#
# Output format:
#   LANG: ja
#   ---
#   (template content)
#
# Example:
#   get-template.sh noppomario uti-dev feature.md

set -euo pipefail

if [[ $# -lt 3 ]]; then
    echo "ERROR: owner, repo, and template name required" >&2
    echo "Usage: get-template.sh <owner> <repo> <template-name>" >&2
    exit 1
fi

OWNER="$1"
REPO="$2"
TEMPLATE="$3"

# Fetch template content
CONTENT=$(gh api "repos/$OWNER/$REPO/contents/.github/ISSUE_TEMPLATE/$TEMPLATE" \
    --jq '.content' 2>/dev/null | base64 -d 2>/dev/null || echo "")

if [[ -z "$CONTENT" ]]; then
    echo "ERROR: Template '$TEMPLATE' not found in $OWNER/$REPO" >&2
    exit 1
fi

# Detect language by checking for Japanese characters (hiragana, katakana, kanji)
# Check first 50 lines for Japanese characters
FIRST_LINES=$(echo "$CONTENT" | head -50)

if echo "$FIRST_LINES" | grep -qP '[\p{Hiragana}\p{Katakana}\p{Han}]'; then
    LANG="ja"
else
    LANG="en"
fi

echo "LANG: $LANG"
echo "---"
echo "$CONTENT"
