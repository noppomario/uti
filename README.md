# uti

A desktop utility that toggles window visibility with double Ctrl press

## Overview

**uti** is a desktop utility for Linux that allows you to toggle window
visibility by quickly pressing the Ctrl key twice.

### Technology Stack

- **Frontend**: React 19 + TypeScript + Tailwind CSS v4
- **Backend**: Tauri 2 (Rust)
- **Daemon**: Rust (evdev + zbus)
- **Target OS**: Linux (Fedora 43 GNOME/Wayland)

## Architecture

```text
[double-ctrl daemon]  ← User session service (requires input group)
  ↓ (Monitor Ctrl via evdev)
  ↓ (Detect double press within 300ms)
  ↓ (Send D-Bus Signal)
[Tauri app]           ← Normal user application
  ↓ (Receive D-Bus)
  ↓ (window.hide/show)
```

## Setup

**For detailed development environment setup**, see [DEVELOPMENT.md](./DEVELOPMENT.md)

### Requirements

- Rust (latest stable)
- Bun (latest)
- Linux with evdev support
- D-Bus session bus
- Tauri development dependencies (see [DEVELOPMENT.md](./DEVELOPMENT.md))
- **System tray support** (GNOME users: see below)

### 1. Clone Repository

```bash
git clone https://github.com/noppomario/uti.git
cd uti
```

### 2. Add User to input Group

The daemon requires access to `/dev/input/*` devices to monitor keyboard events.

```bash
# Add current user to input group
sudo usermod -aG input $USER
```

**Important**: Log out and log back in for the group change to take effect.

Verify group membership:

```bash
groups
# Should include "input"
```

### 3. System Tray Setup (GNOME)

GNOME 43+ users: System tray support requires a GNOME extension.

#### Option 1: Using dnf (Fedora/RHEL)

```bash
sudo dnf install gnome-shell-extension-appindicator
gnome-extensions enable appindicatorsupport@rgcjonas.gmail.com
```

#### Option 2: Using GNOME Extensions website

1. Visit [AppIndicator Support](https://extensions.gnome.org/extension/615/appindicator-support/)
2. Toggle the switch to install
3. Restart GNOME Shell (Alt+F2, type `r`, press Enter) or re-login

#### Verify Installation

```bash
gnome-extensions list | grep appindicator
# Should output: appindicatorsupport@rgcjonas.gmail.com
```

**Note**: Other desktop environments (KDE, XFCE, etc.) have built-in
system tray support and don't require additional setup.

### 4. Build and Install Daemon

```bash
# Build daemon
cargo build --release

# Install as user session service
./install.sh
```

To uninstall:

```bash
./uninstall.sh
```

### 4. Build and Run Tauri App

```bash
cd app
bun install
bun run tauri:dev   # Development mode
bun run tauri:build # Release build
```

### 5. Install Release Build (Optional)

```bash
sudo dnf install ./target/release/bundle/rpm/uti-*.x86_64.rpm
```

## Configuration

The application can be configured via `~/.config/uti/config.json`.

### Configuration File

Create `~/.config/uti/config.json` with the following options:

```json
{
  "theme": "dark",
  "clipboardHistoryLimit": 50
}
```

### Configuration Options

| Option                  | Type   | Default  | Description                                     |
|-------------------------|--------|----------|-------------------------------------------------|
| `theme`                 | string | `"dark"` | UI theme mode: `"dark"` or `"light"`            |
| `clipboardHistoryLimit` | number | `50`     | Maximum number of clipboard items to store      |

### Theme Options

- `"dark"` - Use dark mode (default)
- `"light"` - Use light mode

**Note**: If the config file doesn't exist, default values are used.

### Auto-start

Enable auto-start via the system tray menu:

1. Right-click the tray icon
2. Click "Auto-start" to toggle

When enabled, the application will start automatically on login.
The setting creates a `.desktop` file in `~/.config/autostart/`.

## Development

**For complete development setup instructions**, see [DEVELOPMENT.md](./DEVELOPMENT.md)

### Quick Start

```bash
# One-time setup: Add user to input group (if not already done)
sudo usermod -aG input $USER
# Log out and log back in for changes to take effect

# Terminal 1: Run daemon (no sudo needed)
cd daemon
cargo run

# Terminal 2: Run Tauri app
cd app
bun install
bun run tauri:dev
```

### Debugging

#### Monitor D-Bus Messages

```bash
dbus-monitor "interface='io.github.noppomario.uti.DoubleTap'"
```

#### Check Devices

```bash
ls -l /dev/input/event*
```

#### Check Logs

```bash
# User session service logs
journalctl --user -u double-ctrl.service -f

# Show recent logs
journalctl --user -u double-ctrl.service -n 50
```

#### Change Log Level

The daemon uses the following log levels:

- `error`: Only errors
- `warn`: Warnings and errors
- `info`: Important events (default) - Double Ctrl detection, D-Bus signals
- `debug`: Verbose debugging - All key events

To change the log level, edit the systemd service file:

```bash
# Edit service file
mkdir -p ~/.config/systemd/user
cp /usr/lib/systemd/user/double-ctrl.service ~/.config/systemd/user/
nano ~/.config/systemd/user/double-ctrl.service

# Change: Environment="RUST_LOG=info"
# To:     Environment="RUST_LOG=debug"  (for verbose logging)

# Reload and restart
systemctl --user daemon-reload
systemctl --user restart double-ctrl.service
```

## Project Structure

```text
uti/
├── daemon/         # Keyboard monitoring daemon (Rust)
├── app/            # Tauri GUI application (React + Rust)
├── install.sh      # Daemon installation script
└── uninstall.sh    # Daemon uninstallation script
```

## Known Limitations

- **Window in dock/taskbar (Wayland)**: On Wayland, the window appears in the
  dock/taskbar when visible. This is a Tauri limitation
  ([#9829](https://github.com/tauri-apps/tauri/issues/9829)). The `skipTaskbar`
  setting works correctly on X11.
- **Fixed interval**: Double-tap detection interval is fixed at 300ms
- **Linux only**: Windows/macOS not supported

## Troubleshooting

### Tray Icon Not Visible (GNOME)

If the tray icon doesn't appear:

```bash
# Check if AppIndicator extension is installed and enabled
gnome-extensions list | grep appindicator

# If not found, install it:
sudo dnf install gnome-shell-extension-appindicator
gnome-extensions enable appindicatorsupport@rgcjonas.gmail.com

# Restart GNOME Shell (Alt+F2, type 'r', press Enter) or re-login
```

**GTK Warning in logs**: This is expected if the extension is not installed.
The warning will disappear once the extension is enabled.

### Daemon Won't Start

```bash
# Check logs
journalctl --user -u double-ctrl.service -n 50

# Verify user is in input group
groups  # Should include "input"

# Test manual execution (see DEVELOPMENT.md for setup)
cd daemon
cargo run
```

### D-Bus Communication Fails

```bash
# Check Session Bus
echo $DBUS_SESSION_BUS_ADDRESS

# Test manual Signal send
dbus-send --session \
  --type=signal \
  /io/github/noppomario/uti/DoubleTap \
  io.github.noppomario.uti.DoubleTap.Triggered
```

## TODO

- [ ] Windows support

## License

MIT License

## Author

Anonymous
