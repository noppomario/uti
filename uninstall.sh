#!/bin/bash
set -e

echo "=== uti Uninstallation Script ==="
echo ""

# Stop and disable the user service
echo "[1/3] Stopping user service..."
systemctl --user stop double-ctrl.service || true
systemctl --user disable double-ctrl.service || true

# Remove systemd user service
echo "[2/3] Removing systemd user service..."
rm -f ~/.config/systemd/user/double-ctrl.service
systemctl --user daemon-reload

# Remove the daemon (requires sudo)
echo "[3/3] Removing daemon from /usr/local/bin..."
sudo rm -f /usr/local/bin/double-ctrl

echo ""
echo "✓ Uninstallation complete!"
echo ""
echo "Note: Your user is still in the 'input' group."
echo "To remove (optional):"
echo "  sudo gpasswd -d $USER input"
echo ""
