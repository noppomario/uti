<div align="center">

# uti

<img src="app/src-tauri/icons/icon.png" alt="uti icon" width="128" style="border-radius: 16px; box-shadow: 0 4px 8px rgba(0,0,0,0.3);">

[![Release](https://img.shields.io/github/v/release/noppomario/uti)](https://github.com/noppomario/uti/releases)
[![CI](https://github.com/noppomario/uti/actions/workflows/ci.yml/badge.svg)](https://github.com/noppomario/uti/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux-lightgrey.svg)](https://github.com/noppomario/uti)

[![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-24C8D8?logo=tauri&logoColor=white)](https://tauri.app/)
[![React](https://img.shields.io/badge/React_19-61DAFB?logo=react&logoColor=black)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Tailwind CSS](https://img.shields.io/badge/Tailwind_v4-06B6D4?logo=tailwindcss&logoColor=white)](https://tailwindcss.com/)
[![Bun](https://img.shields.io/badge/Bun-000000?logo=bun&logoColor=white)](https://bun.sh/)
[![Biome](https://img.shields.io/badge/Biome-60A5FA?logo=biome&logoColor=white)](https://biomejs.dev/)
[![Vite](https://img.shields.io/badge/Vite-646CFF?logo=vite&logoColor=white)](https://vitejs.dev/)

> A modern clipboard manager and app launcher for Linux with double-Ctrl toggle

</div>

## ✨ Features

- 🎹 **Double Ctrl Toggle**: Press Ctrl twice quickly (within 300ms) to show/hide the window
- 📋 **Clipboard History**: Stores clipboard items for quick access
- 🚀 **App Launcher**: Quick-launch configured applications with jump lists (recent files)
- 🔍 **Search**: Filter clipboard history or search system applications in real-time
- 🖥️ **System Tray**: Runs in the background with tray icon control
- 📍 **Cursor Positioning**: Window appears at cursor location on GNOME
- 🔄 **Auto-start & Self-update**: Optional auto-start on login, update via CLI or tray menu

<p align="center">
  <img src="docs/assets/screenshot_1.png" alt="Clipboard tab" width="32%">
  <img src="docs/assets/screenshot_2.png" alt="Launcher tab" width="32%">
  <img src="docs/assets/screenshot_3.png" alt="Jump list expanded" width="32%">
</p>

## 📋 System Requirements

- **OS**: Linux with systemd (Fedora 43+ recommended)
- **Desktop**: GNOME 45+ on Wayland (recommended), other Wayland/X11 environments (limited)
- **Architecture**: x86_64

## 🚀 Quick Start

Install with one command:

```bash
curl -fsSL https://raw.githubusercontent.com/noppomario/uti/main/install.sh | bash
```

After installation, **log out and log back in** (required for input group), then run `uti`.

<details>
<summary><strong>GNOME Users</strong></summary>

On GNOME, the installer automatically installs the "uti for GNOME" extension. After logging back in, enable it:

```bash
gnome-extensions enable uti@noppomario.github.io
```

> **Note**: GNOME Shell only detects newly installed local extensions after a logout/login. The extension cannot be enabled until you log back in.

This extension provides:

- **Panel icon with full menu** (same menu as system tray)
- **Cursor-relative window positioning** (window appears at cursor location)

The extension displays Tauri's tray icon directly, so no additional extensions (like AppIndicator) are required.

Without the extension, uti still works but the window appears at screen center.

</details>

<details>
<summary>Using AppIndicator instead?</summary>

If you already have AppIndicator extension installed and prefer to use it, disable the extension's tray icon:

```bash
gsettings set org.gnome.shell.extensions.uti enable-tray-icon false
```

</details>

## 🎯 Why uti?

**Solving the challenges of Wayland environments with modern tooling.**

- 🔒 **Wayland Limitations**: Wayland restricts global keyboard shortcuts. Launching apps with double Ctrl press cannot be achieved through standard desktop settings. Additionally, Wayland prevents apps from querying global cursor position or setting their own window position, requiring compositor (GNOME Shell) integration for cursor-relative window positioning.

- 🔍 **No Existing Tools**: No clipboard manager was found that works natively on Wayland and can be triggered by double Ctrl press.

- ⚡ **Daemon Architecture**: A dedicated Rust daemon monitors keyboard input via evdev, enabling flexible shortcuts independent of the desktop environment.

- 🦀 **Modern Stack**: Built entirely with Rust backend, React 19 frontend, and tooling like Bun, Biome, and Vite for maximum developer experience.

- 🧩 **GNOME Integration**: Custom extension displays window at cursor position and provides native panel icon—features impossible for regular Wayland apps.

## 🛠️ Tech Stack

| Layer | Technology |
| ----- | ---------- |
| 🦀 Backend | Rust + Tauri 2 |
| ⚛️ Frontend | React 19 + TypeScript 5.7 |
| 🎨 Styling | Tailwind CSS v4 |
| 📦 Bundler | Vite 6 + Bun |
| 🔧 Linting | Biome (25-100x faster than ESLint) |
| 🎹 Daemon | Rust + evdev + D-Bus |

## 📖 Usage

### Basic Operation

1. Press **Ctrl twice quickly** to toggle window visibility
2. Use **←/→** to switch between Clipboard and Launcher tabs
3. Use **↑/↓** to navigate items, **Enter** to select
4. Press **1-9** to quickly select an item by its number
5. Press **Ctrl+F** to focus the search bar
6. **Type to search**: Filter clipboard history or search system applications
7. **Escape** clears search and returns focus to the list
8. In Launcher tab, press **→** to expand jump list (recent files)

### System Tray

Right-click the tray icon for options:

- **Show/Hide**: Toggle window visibility
- **Auto-start**: Enable/disable start on login
- **Check for Updates...**: Check for new versions
- **GitHub**: Open project page
- **Quit**: Exit application

<img src="docs/assets/screenshot_4.png" alt="System tray menu" width="280">

### Updating

Check for and install updates:

```bash
uti update
```

Check only (without installing):

```bash
uti update --check
```

## ⚙️ Configuration

Configuration file: `~/.config/uti/config.json`

```json
{
  "theme": {
    "color": "dark",
    "size": "normal"
  },
  "clipboardHistoryLimit": 50
}
```

| Option                  | Type   | Default                            | Description                  |
| ----------------------- | ------ | ---------------------------------- | ---------------------------- |
| `theme.color`           | string | `dark`                             | `midnight`, `dark`, `light`  |
| `theme.size`            | string | `normal`                           | `minimal`, `normal`, `wide`  |
| `clipboardHistoryLimit` | number | `50`                               | Max clipboard items to store |

### Launcher Configuration

Launcher commands: `~/.config/uti/launcher.json`

```json
{
  "commands": [
    {
      "id": "nautilus",
      "name": "Files",
      "command": "nautilus",
      "historySource": { "type": "recently-used", "appName": "org.gnome.Nautilus" }
    },
    {
      "id": "vscode",
      "name": "VSCode",
      "command": "code",
      "historySource": { "type": "vscode", "path": "~/.config/Code/User/globalStorage/state.vscdb" }
    }
  ]
}
```

For detailed configuration options, see [Launcher Configuration Guide](docs/launcher-config.md).

## 🔧 Troubleshooting

<details>
<summary><strong>Daemon Won't Start</strong></summary>

Check if you're in the input group:

```bash
groups  # Should include "input"
```

If not, add yourself and log out/in:

```bash
sudo usermod -aG input $USER
```

Check daemon status:

```bash
systemctl --user status uti-daemon.service
journalctl --user -u uti-daemon.service -n 50
```

</details>

<details>
<summary><strong>Tray Icon Not Visible (GNOME)</strong></summary>

Enable uti for GNOME (see GNOME Users section above).

</details>

## 🗑️ Uninstallation

```bash
sudo dnf remove uti uti-daemon
```

This automatically stops and disables the daemon service.

To remove the GNOME extension (optional):

```bash
rm -rf ~/.local/share/gnome-shell/extensions/uti@noppomario.github.io
```

To remove yourself from the input group (optional):

```bash
sudo gpasswd -d $USER input
```

## ⚠️ Known Limitations

- **Window appears in dock (Wayland)**: On Wayland, the window appears in the dock when visible. This is a Tauri limitation ([#9829](https://github.com/tauri-apps/tauri/issues/9829)).
- **Window position (non-GNOME)**: On non-GNOME Wayland environments (KDE, Sway, etc.), window always appears at screen center. Enable uti for GNOME for cursor positioning.
- **Jump list app support**: Only apps that write to `recently-used.xbel` (GTK/GNOME apps) and VSCode are supported. KDE apps and most Electron apps are not supported.

## 🏗️ Architecture

| Component | Role |
| --------- | ---- |
| **uti-daemon** | Detects double Ctrl press via evdev |
| **uti** | Clipboard manager + App launcher UI (Tauri) |
| **uti for GNOME** | GNOME Shell extension for tray icon + cursor positioning (optional) |

For details, see [Architecture Documentation](docs/ARCHITECTURE.md).

## 📦 What Gets Installed

When you install uti, the following changes are made to your system:

| Component | Location | Description |
| --------- | -------- | ----------- |
| **uti** | `/usr/bin/uti` | Main application (RPM package) |
| **uti-daemon** | `/usr/bin/uti-daemon` | Keyboard daemon (RPM package) |
| **User service** | `~/.config/systemd/user/` | Daemon autostart service |
| **Config** | `~/.config/uti/` | User configuration, clipboard history, launcher config |
| **Input group** | `/etc/group` | Your user is added to the `input` group |
| **uti for GNOME** | `~/.local/share/gnome-shell/extensions/` | GNOME Shell extension (GNOME only) |

### ⚠️ About the Input Group

The daemon needs to read keyboard events from `/dev/input/*` devices. This requires membership in the **input group**. The installer automatically adds your user to this group.

**Security note**: Members of the input group can read all input devices (keyboard, mouse). This is necessary for the double-Ctrl detection to work.

## 👨‍💻 Development

For development setup, see [DEVELOPMENT.md](docs/DEVELOPMENT.md).

## 📄 License

[MIT](LICENSE)
