#!/usr/bin/env bash
# Run the GTK app against a copy of ~/.config/uti so a released uti keeps its
# data (the GTK app migrates clipboard history to $XDG_DATA_HOME on start).
# Usage: app/run-dev.sh [--reset] [app args...]
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
sandbox="$root/target/uti-dev"
if [[ "${1:-}" == "--reset" ]]; then
  rm -rf "$sandbox"
  shift
fi
if [[ ! -d "$sandbox" ]]; then
  mkdir -p "$sandbox/config" "$sandbox/data"
  cp -r "$HOME/.config/uti" "$sandbox/config/" 2>/dev/null || true
fi

cargo build -p uti
UTI_DEV_SANDBOX=1 XDG_CONFIG_HOME="$sandbox/config" XDG_DATA_HOME="$sandbox/data" \
  exec "$root/target/debug/uti" "$@"
