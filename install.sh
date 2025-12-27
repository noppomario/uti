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

# Build the daemon
echo "[1/4] Building daemon..."
cd daemon
cargo build --release
cd ..

# Install the daemon
echo "[2/4] Installing daemon..."
cp daemon/target/release/double-ctrl /usr/local/bin/
chmod +x /usr/local/bin/double-ctrl

# Install systemd service
echo "[3/4] Setting up systemd service..."
cp daemon/systemd/double-ctrl.service /etc/systemd/system/
systemctl daemon-reload

# Enable the service
echo "[4/4] Enabling service..."
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
