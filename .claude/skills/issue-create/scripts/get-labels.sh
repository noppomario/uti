#!/bin/bash
# Get available labels from a GitHub repository.
#
# Usage:
#   get-labels.sh <owner/repo>
#
# Output:
#   JSON array of labels with name, description, and color
#
# Example:
#   get-labels.sh noppomario/uti-dev

set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "ERROR: Repository required" >&2
    echo "Usage: get-labels.sh <owner/repo>" >&2
    exit 1
fi

REPO="$1"

# Get labels from repository
gh label list -R "$REPO" --json name,description,color
