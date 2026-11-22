#!/usr/bin/env bash
# Wait until the release PR for vX.Y.Z is merged and the COPR build of that
# version finishes. Exit 0 on success, 1 on failure, 2 on timeout.
# Usage: .claude/skills/release/wait-copr.sh X.Y.Z [timeout_minutes]
set -euo pipefail

version="$1"
deadline=$(($(date +%s) + ${2:-120} * 60))
repo="noppomario/uti"
copr="https://copr.fedorainfracloud.org/api_3/build/list?ownername=noppomario&projectname=uti&limit=5"

while (($(date +%s) < deadline)); do
  pr_state=$(gh pr view "release/v$version" --repo "$repo" --json state --jq .state 2>/dev/null || echo UNKNOWN)
  if [[ "$pr_state" == "CLOSED" ]]; then
    echo "release PR closed without merging"
    exit 1
  fi
  if [[ "$pr_state" == "MERGED" ]]; then
    build=$(curl -fsS "$copr" | python3 -c '
import json, sys
v = sys.argv[1]
for b in json.load(sys.stdin)["items"]:
    if ((b.get("source_package") or {}).get("version") or "").startswith(v + "-"):
        print(b["id"], b["state"])
        break
' "$version" || true)
    read -r id state <<<"${build:-none none}"
    case "$state" in
      succeeded)
        echo "COPR build $id succeeded"
        exit 0
        ;;
      failed | canceled)
        echo "COPR build $id $state: https://copr.fedorainfracloud.org/coprs/build/$id/"
        exit 1
        ;;
    esac
  fi
  sleep 60
done
echo "timed out (PR: $pr_state)"
exit 2
