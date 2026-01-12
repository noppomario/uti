#!/bin/bash
# Update GitHub Project item status for an Issue.
#
# Usage:
#   project-status.sh <issue-url> <status>
#
# Status values are normalized (case-insensitive):
#   inprogress, in-progress, InProgress -> "In Progress"
#   todo, Todo                          -> "Todo"
#   done, Done, complete                -> "Done"
#   now, Now                            -> "Now"
#   next, Next                          -> "Next"
#   backlog, Backlog                    -> "Backlog"
#   later, Later                        -> "Later"
#
# Output:
#   OK: Now -> In Progress (uti開発)
#   WARN: Issue is not linked to any project
#   ERROR: Invalid status "Working". Available: ...

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ $# -lt 2 ]]; then
    echo "ERROR: Issue URL and status required" >&2
    echo "Usage: project-status.sh <issue-url> <status>" >&2
    exit 1
fi

ISSUE_URL="$1"
INPUT_STATUS="$2"

# Parse issue URL
PARSED=$("$SCRIPT_DIR/parse-issue-url.sh" "$ISSUE_URL")
OWNER=$(echo "$PARSED" | jq -r '.owner')
REPO=$(echo "$PARSED" | jq -r '.repo')
NUMBER=$(echo "$PARSED" | jq -r '.number')

# Normalize status value
normalize_status() {
    local input="$1"
    local lower=$(echo "$input" | tr '[:upper:]' '[:lower:]')

    case "$lower" in
        inprogress|in-progress|in_progress)
            echo "In Progress"
            ;;
        todo)
            echo "Todo"
            ;;
        done|complete)
            echo "Done"
            ;;
        now)
            echo "Now"
            ;;
        next)
            echo "Next"
            ;;
        backlog)
            echo "Backlog"
            ;;
        later)
            echo "Later"
            ;;
        *)
            # Return as-is if not recognized (will be validated later)
            echo "$input"
            ;;
    esac
}

TARGET_STATUS=$(normalize_status "$INPUT_STATUS")

# Get project items for the issue
PROJECT_INFO=$(gh issue view "$ISSUE_URL" --json projectItems 2>/dev/null || echo '{"projectItems":[]}')
PROJECT_ITEMS=$(echo "$PROJECT_INFO" | jq -r '.projectItems')
PROJECT_COUNT=$(echo "$PROJECT_ITEMS" | jq 'length')

if [[ "$PROJECT_COUNT" -eq 0 ]]; then
    echo "WARN: Issue is not linked to any project"
    exit 0
fi

# Get first project info
PROJECT_TITLE=$(echo "$PROJECT_ITEMS" | jq -r '.[0].title')
CURRENT_STATUS=$(echo "$PROJECT_ITEMS" | jq -r '.[0].status.name // "Unknown"')

# Get project list to find project ID and number
PROJECTS=$(gh project list --owner "$OWNER" --format json 2>/dev/null || echo '{"projects":[]}')
PROJECT_DATA=$(echo "$PROJECTS" | jq -r --arg title "$PROJECT_TITLE" '.projects[] | select(.title == $title)')

if [[ -z "$PROJECT_DATA" ]]; then
    echo "ERROR: Project '$PROJECT_TITLE' not found for owner '$OWNER'" >&2
    exit 1
fi

PROJECT_ID=$(echo "$PROJECT_DATA" | jq -r '.id')
PROJECT_NUM=$(echo "$PROJECT_DATA" | jq -r '.number')

# Get project item ID using item-list
ITEM_DATA=$(gh project item-list "$PROJECT_NUM" --owner "$OWNER" --format json 2>/dev/null | \
    jq -r --arg num "$NUMBER" --arg repo "$OWNER/$REPO" \
    '.items[] | select(.content.number == ($num | tonumber) and .content.repository == $repo)')

if [[ -z "$ITEM_DATA" ]]; then
    echo "ERROR: Issue #$NUMBER not found in project '$PROJECT_TITLE'" >&2
    exit 1
fi

ITEM_ID=$(echo "$ITEM_DATA" | jq -r '.id')

# Get Status field info
FIELD_DATA=$(gh project field-list "$PROJECT_NUM" --owner "$OWNER" --format json 2>/dev/null | \
    jq -r '.fields[] | select(.name == "Status")')

if [[ -z "$FIELD_DATA" ]]; then
    echo "ERROR: Status field not found in project '$PROJECT_TITLE'" >&2
    exit 1
fi

FIELD_ID=$(echo "$FIELD_DATA" | jq -r '.id')
AVAILABLE_OPTIONS=$(echo "$FIELD_DATA" | jq -r '.options[].name' | tr '\n' ', ' | sed 's/,$//')

# Find option ID for target status
OPTION_ID=$(echo "$FIELD_DATA" | jq -r --arg status "$TARGET_STATUS" \
    '.options[] | select(.name == $status) | .id')

if [[ -z "$OPTION_ID" ]]; then
    echo "ERROR: Invalid status \"$INPUT_STATUS\". Available: $AVAILABLE_OPTIONS" >&2
    exit 1
fi

# Update status
gh project item-edit \
    --project-id "$PROJECT_ID" \
    --id "$ITEM_ID" \
    --field-id "$FIELD_ID" \
    --single-select-option-id "$OPTION_ID" >/dev/null 2>&1

echo "OK: $CURRENT_STATUS -> $TARGET_STATUS ($PROJECT_TITLE)"
