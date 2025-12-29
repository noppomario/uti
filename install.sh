#!/bin/bash
set -e

echo "=== uti Installation Script ==="
echo ""

# Check for root permissions
if [ "$EUID" -ne 0 ]; then
    echo "Error: This script must be run with root privileges"
    echo "Usage: sudo ./install.sh"
    exit 1
fi

# Check if daemon binary exists
if [ ! -f "daemon/target/release/double-ctrl" ]; then
    echo "Error: Daemon binary not found"
    echo ""
    echo "Please build the daemon first:"
    echo "  cd daemon"
    echo "  cargo build --release"
    echo "  cd .."
    echo ""
    exit 1
fi

# Install the daemon
echo "[1/3] Installing daemon..."
cp daemon/target/release/double-ctrl /usr/local/bin/
chmod +x /usr/local/bin/double-ctrl

# Install systemd service
echo "[2/3] Setting up systemd service..."
cp daemon/systemd/double-ctrl.service /etc/systemd/system/
systemctl daemon-reload

# Enable the service
echo "[3/3] Enabling service..."
systemctl enable double-ctrl.service
systemctl start double-ctrl.service

echo ""
echo "✓ Installation complete!"
echo ""
echo "Service status:"
systemctl status double-ctrl.service --no-pager
echo ""
echo "Next steps:"
echo "  1. cd app"
echo "  2. bun install"
echo "  3. bun run tauri:dev"
echo ""
