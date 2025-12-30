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

### 3. Build and Install Daemon

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

## Troubleshooting

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

- [ ] Robust device detection (multiple keyboard support)
- [ ] Configuration file support (interval adjustment, etc.)
- [ ] System tray icon support
- [ ] Window position/size persistence
- [ ] Windows support

## License

MIT License

## Author

Anonymous
