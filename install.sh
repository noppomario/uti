#!/bin/bash
# uti installer for Fedora: enables the COPR repository and installs uti,
# uti-daemon and uti for GNOME. Updates then come with `dnf upgrade`.
set -euo pipefail

COPR_PROJECT="noppomario/uti"
EXT_UUID="uti@noppomario.github.io"

echo "=== uti Installer ==="

for tool in dnf systemctl; do
    command -v "$tool" > /dev/null || { echo "Error: $tool not found (Fedora required)"; exit 1; }
done

echo "[1/4] Enabling COPR repository $COPR_PROJECT (sudo)..."
sudo dnf -y copr enable "$COPR_PROJECT"

echo "[2/4] Installing packages..."
sudo dnf -y install uti uti-daemon gnome-shell-extension-uti

# A user-local copy (old installer) would shadow the packaged extension.
legacy_ext="$HOME/.local/share/gnome-shell/extensions/$EXT_UUID"
if [ -d "$legacy_ext" ]; then
    rm -rf "$legacy_ext"
    echo "  Removed old extension copy: $legacy_ext"
fi

echo "[3/4] Checking input group membership..."
relogin=false
if id -nG "$USER" | grep -qw input; then
    echo "  Already in input group."
else
    sudo usermod -aG input "$USER"
    echo "  Added $USER to the input group."
    relogin=true
fi

echo "[4/4] Enabling uti-daemon user service..."
systemctl --user daemon-reload
systemctl --user enable --now uti-daemon.service

echo ""
echo "Installation complete."
if [ "$relogin" = true ]; then
    echo "!! Log out and log back in: the daemon needs the input group. !!"
fi
echo "Enable uti for GNOME (after logging back in if it is not found yet):"
echo "  gnome-extensions enable $EXT_UUID"
echo "Update: sudo dnf upgrade uti"
echo "Uninstall: sudo dnf remove uti uti-daemon gnome-shell-extension-uti"
