# uti

A desktop utility that toggles window visibility with double Ctrl press

## Overview

**uti** is a desktop utility for Linux that allows you to toggle window visibility by quickly pressing the Ctrl key twice.

### Technology Stack

- **Frontend**: React 19 + TypeScript + Tailwind CSS v4
- **Backend**: Tauri 2 (Rust)
- **Daemon**: Rust (evdev + zbus)
- **Target OS**: Linux (Fedora 43 GNOME/Wayland)

## Architecture

```
[double-ctrl daemon]  ← Runs with root/input permissions
  ↓ (Monitor Ctrl via evdev)
  ↓ (Detect double press within 300ms)
  ↓ (Send D-Bus Signal)
[Tauri app]           ← Runs with normal user permissions
  ↓ (Receive D-Bus)
  ↓ (window.hide/show)
```

## Setup

### Requirements

- Rust (latest stable)
- Bun (latest)
- Linux with evdev support
- D-Bus session bus

### 1. Clone Repository

```bash
git clone https://github.com/noppomario/uti.git
cd uti
```

### 2. Build and Install Daemon

```bash
cd daemon
cargo build --release

# Install
sudo cp target/release/double-ctrl /usr/local/bin/
sudo cp systemd/double-ctrl.service /etc/systemd/system/

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable --now double-ctrl.service

# Check status
sudo systemctl status double-ctrl.service
```

### 3. Build and Run Tauri App

```bash
cd ../app
bun install
bun run tauri:dev   # Development mode
bun run tauri:build # Release build
```

## Development

### Daemon Development

```bash
cd daemon
sudo cargo run  # Requires root permissions
```

### App Development

```bash
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
journalctl -u double-ctrl.service -f
```

## Project Structure

```
uti/
├── daemon/                          # Ctrl detection daemon
│   ├── Cargo.toml
│   ├── systemd/
│   │   └── double-ctrl.service
│   └── src/
│       └── main.rs
│
└── app/                             # Tauri GUI app
    ├── package.json
    ├── vite.config.ts
    ├── src/                         # React frontend
    │   ├── App.tsx
    │   ├── main.tsx
    │   └── index.css
    └── src-tauri/                   # Rust backend
        ├── Cargo.toml
        └── src/
            └── main.rs
```

## Troubleshooting

### Daemon Won't Start

```bash
# Check logs
sudo journalctl -u double-ctrl.service -n 50

# Test manual execution
cd daemon
sudo cargo run
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
