#!/bin/bash
set -e

echo "=== uti Uninstallation Script ==="
echo ""

# Check for root permissions
if [ "$EUID" -ne 0 ]; then
    echo "Error: This script must be run with root privileges"
    echo "Usage: sudo ./uninstall.sh"
    exit 1
fi

# Stop and disable the service
echo "[1/3] Stopping service..."
systemctl stop double-ctrl.service || true
systemctl disable double-ctrl.service || true

# Remove systemd service
echo "[2/3] Removing systemd service..."
rm -f /etc/systemd/system/double-ctrl.service
systemctl daemon-reload

# Remove the daemon
echo "[3/3] Removing daemon..."
rm -f /usr/local/bin/double-ctrl

echo ""
echo "✓ Uninstallation complete!"
echo ""
