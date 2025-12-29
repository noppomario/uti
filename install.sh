#!/bin/bash
set -e

echo "=== uti Installation Script ==="
echo ""

# Check if daemon binary exists
if [ ! -f "target/release/double-ctrl" ]; then
    echo "Error: Daemon binary not found"
    echo ""
    echo "Please build the daemon first:"
    echo "  cargo build --release"
    echo ""
    exit 1
fi

# Check if user is in input group
if ! groups | grep -q "\binput\b"; then
    echo "Error: Current user is not in the 'input' group"
    echo ""
    echo "Please add your user to the input group:"
    echo "  sudo usermod -aG input $USER"
    echo ""
    echo "Then log out and log back in for the change to take effect."
    exit 1
fi

# Install the daemon (requires sudo for /usr/local/bin)
echo "[1/3] Installing daemon to /usr/local/bin..."
sudo cp target/release/double-ctrl /usr/local/bin/
sudo chmod +x /usr/local/bin/double-ctrl

# Install systemd user service
echo "[2/3] Setting up systemd user service..."
mkdir -p ~/.config/systemd/user
cp daemon/systemd/double-ctrl.service ~/.config/systemd/user/double-ctrl.service
systemctl --user daemon-reload

# Enable the service
echo "[3/3] Enabling user service..."
systemctl --user enable double-ctrl.service
systemctl --user start double-ctrl.service

echo ""
echo "✓ Installation complete!"
echo ""
echo "Service status:"
systemctl --user status double-ctrl.service --no-pager
echo ""
echo "Next steps:"
echo "  1. cd app"
echo "  2. bun install"
echo "  3. bun run tauri:dev"
echo ""
echo "To check daemon logs:"
echo "  journalctl --user -u double-ctrl.service -f"
echo ""
