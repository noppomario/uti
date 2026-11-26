#!/usr/bin/env bash
# End-to-end checks of the GTK app in a headless GNOME Shell on a private
# session bus: a scenario (gjs) moves the pointer and types through Mutter's
# RemoteDesktop API and asserts on the app's `debug-state`.
# Usage: tools/e2e/run.sh [scenario.js] (default: every *.js here but lib.js)
# Env: E2E_SIZE (theme.size, default normal), E2E_KEEP=1 keeps target/e2e.
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
here="$root/tools/e2e"
work="$root/target/e2e"

if [[ -z "${E2E_INNER:-}" ]]; then
  command -v gnome-shell >/dev/null || { echo "gnome-shell is required" >&2; exit 1; }
  command -v gjs >/dev/null || { echo "gjs is required" >&2; exit 1; }
  cargo build -p uti
  rm -rf "$work"
  mkdir -p "$work/config/uti" "$work/data"
  cp "$here/fixtures/recently-used.xbel" "$work/data/"
  sed "s|@DATA@|$work/data|g" "$here/fixtures/launcher.json" >"$work/config/uti/launcher.json"
  printf '{"theme": {"size": "%s"}}\n' "${E2E_SIZE:-normal}" >"$work/config/uti/config.json"
  exec env E2E_INNER=1 dbus-run-session -- "$0" "$@"
fi

display=wayland-uti-e2e
gnome-shell --headless --wayland --virtual-monitor 1400x900 --wayland-display="$display" \
  >"$work/shell.log" 2>&1 &
shell=$!
trap 'kill "${app:-}" "$shell" 2>/dev/null || true' EXIT
for _ in $(seq 60); do
  [[ -S "$XDG_RUNTIME_DIR/$display" ]] && break
  sleep 0.5
done
sleep 3

WAYLAND_DISPLAY=$display GDK_BACKEND=wayland GTK_A11Y=none UTI_DEV_SANDBOX=1 UTI_DEBUG_KEYS=1 \
  UTI_DEBUG_SNAPSHOT="$work/last.png" XDG_CONFIG_HOME="$work/config" XDG_DATA_HOME="$work/data" \
  "$root/target/debug/uti" --minimized >"$work/uti.log" 2>&1 &
app=$!

scenarios=("$@")
if [[ ${#scenarios[@]} -eq 0 ]]; then
  mapfile -t scenarios < <(find "$here" -maxdepth 1 -name '*.js' ! -name lib.js | sort)
fi
status=0
for scenario in "${scenarios[@]}"; do
  echo "== $(basename "$scenario")"
  E2E_WORK="$work" E2E_LOG="$work/uti.log" gjs -m "$scenario" || status=1
done
exit $status
