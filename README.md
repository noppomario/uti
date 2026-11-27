<div align="center">

<img src="app/packaging/icons/hicolor/256x256/apps/io.github.noppomario.uti.png" alt="uti icon" width="128" style="border-radius: 16px; box-shadow: 0 4px 8px rgba(0,0,0,0.3);">

# uti

[![Release](https://img.shields.io/github/v/release/noppomario/uti)](https://github.com/noppomario/uti/releases)
[![CI](https://github.com/noppomario/uti/actions/workflows/ci.yml/badge.svg)](https://github.com/noppomario/uti/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux-lightgrey.svg)](https://github.com/noppomario/uti)

[![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![GTK4](https://img.shields.io/badge/GTK4-4A86CF?logo=gtk&logoColor=white)](https://gtk-rs.org/)
[![libadwaita](https://img.shields.io/badge/libadwaita-4A86CF?logo=gnome&logoColor=white)](https://gnome.pages.gitlab.gnome.org/libadwaita/)
[![Biome](https://img.shields.io/badge/Biome-60A5FA?logo=biome&logoColor=white)](https://biomejs.dev/)

> A modern clipboard manager and app launcher for Linux with double-Ctrl toggle

<img src="docs/assets/screenshot_1.png" alt="Clipboard tab" width="200">
<img src="docs/assets/screenshot_2.png" alt="Launcher tab" width="200">
<img src="docs/assets/screenshot_3.png" alt="Jump list expanded" width="200">

</div>

## ✨ Features

- 🎹 **Double Ctrl Toggle**: Press Ctrl twice quickly (within 300ms) to show/hide the window
- 📋 **Clipboard History**: Stores clipboard items for quick access
- ⭐ **Snippets**: Keep frequently used text; add from clipboard history, manage in Settings
- 🚀 **App Launcher**: Cascading menu of installed apps by category, power actions and your own commands, with recent files for each app
- 🔍 **Search**: Filter clipboard history or search system applications in real-time
- 📌 **Window Pinning**: Pin button keeps the window always-on-top with auto-hide disabled
- 🖥️ **Panel Menu**: Show/Hide, Settings and more from the GNOME panel (uti for GNOME)
- 📍 **Cursor Positioning**: Window appears at cursor location on GNOME
- 🎨 **Native Look**: GTK4 / libadwaita; follows the system light/dark style and accent color
- 🔄 **Auto-start**: Optional auto-start on login (Settings)

## 📋 System Requirements

- **OS**: Linux with systemd (Fedora 44+; the app needs libadwaita 1.9+)
- **Desktop**: GNOME 50 or 51 on Wayland (recommended), other Wayland/X11 environments (limited)
- **Architecture**: x86_64

## 🚀 Quick Start

Install with one command:

```bash
curl -fsSL https://raw.githubusercontent.com/noppomario/uti/main/install.sh | bash
```

After installation, **log out and log back in** (required for input group), then run `uti`.

The installer enables the [COPR repository](https://copr.fedorainfracloud.org/coprs/noppomario/uti/), installs the packages, adds you to the `input` group and enables the daemon. To install manually instead:

```bash
sudo dnf copr enable noppomario/uti
sudo dnf install uti uti-daemon gnome-shell-extension-uti
sudo usermod -aG input $USER
systemctl --user enable --now uti-daemon.service
```

<details>
<summary><strong>GNOME Users</strong></summary>

The "uti for GNOME" extension is the `gnome-shell-extension-uti` package. After logging back in, enable it:

```bash
gnome-extensions enable uti@noppomario.github.io
```

> **Note**: GNOME Shell only detects newly installed extensions after a logout/login. The extension cannot be enabled until you log back in.

This extension provides:

- **Panel icon with full menu** (Show/Hide, Settings, Check for Updates, GitHub, Quit)
- **Cursor-relative window positioning** (window appears at cursor location)

Without the extension, uti still works but the window appears at screen center.

</details>

<details>
<summary>Hide the panel button?</summary>

The panel button belongs to uti for GNOME (the app itself shows no tray icon). To hide it and use double Ctrl only:

```bash
gsettings set org.gnome.shell.extensions.uti enable-tray-icon false
```

</details>

## 🎯 Why uti?

**Solving the challenges of Wayland environments with modern tooling.**

- 🔒 **Wayland Limitations**: Wayland restricts global keyboard shortcuts. Launching apps with double Ctrl press cannot be achieved through standard desktop settings. Additionally, Wayland prevents apps from querying global cursor position or setting their own window position, requiring compositor (GNOME Shell) integration for cursor-relative window positioning.

- 🔍 **No Existing Tools**: No clipboard manager was found that works natively on Wayland and can be triggered by double Ctrl press.

- ⚡ **Daemon Architecture**: A dedicated Rust daemon monitors keyboard input via evdev, enabling flexible shortcuts independent of the desktop environment.

- 🦀 **Native Stack**: Written entirely in Rust with GTK4 / libadwaita, so it looks and behaves like a GNOME app.

- 🧩 **GNOME Integration**: Custom extension displays window at cursor position and provides native panel icon—features impossible for regular Wayland apps.

## 🛠️ Tech Stack

| Layer | Technology |
| ----- | ---------- |
| 🦀 App | Rust + gtk4-rs + libadwaita |
| 🎹 Daemon | Rust + evdev + uinput + D-Bus |
| 🧩 GNOME extension | GJS |
| 🔧 Tooling | mise, just, lefthook, clippy, cargo-deny, Biome (extension), rumdl |

## 📖 Usage

### Basic Operation

1. Press **Ctrl twice quickly** to toggle window visibility; it opens on the Clipboard tab. Optional mouse triggers (Settings > Mouse) open the Launcher tab
2. Use **←/→** to switch between Clipboard, Snippets, and Launcher tabs
3. Use **↑/↓** to navigate items, **Enter** to select
4. Press **1-9** to quickly select an item by its number
5. Press **Ctrl+F** to focus the search bar
6. **Type to search**: Filter items or search system applications
7. **Escape** clears search and returns focus to the list
8. **Right-click** a row, click its **⋮** button, or press **Menu** / **Shift+F10** for its actions:
   - Clipboard: Add to Snippets (**S**; the item stays in history), Delete from History (**Delete**)
   - Snippets: Edit (**F2**), New Snippet, Move Up / Down, Delete (**Delete**). Edit and New
     open Settings > Snippets, where snippets can also be added, edited, reordered and deleted
9. In **Launcher tab**, hover a row with **›** or press **→** to open its folder or recent files
10. Click **📌** (pin button) to keep window always-on-top and disable auto-hide
    - While pinned, drag rows to reorder snippets and organize the Launcher: drop on the
      middle of a folder to move into it, or rest on it to open it. Rows inside generated
      menus (apps, power actions) are copied
    - **Note**: Always-on-top requires the "uti for GNOME" extension on GNOME/Wayland. Without the extension, only auto-hide is disabled.

### Panel Menu (GNOME)

Click the uti panel icon for options:

- **Show/Hide**: Toggle window visibility
- **Settings**: Open settings window (includes auto-start)
- **Check for Updates**: Open the Updates page of GNOME Software
- **GitHub**: Open project page
- **Quit**: Exit application

### Updating

Updates come from the COPR repository like any other package:

- **GNOME Software** (Check for Updates in the panel menu or settings): it downloads updates and notifies you; Restart & Update installs them.
- **Terminal**: `sudo dnf upgrade`, then log out and back in. The daemon restarts itself, but the running app and the extension keep the old version until the next login.

## ⚙️ Configuration

Configuration file: `~/.config/uti/config.json`

```json
{
  "theme": {
    "size": "normal",
    "scrollbar": "overlay"
  },
  "clipboardHistoryLimit": 50,
  "language": "en",
  "mouse": {
    "desktopDoubleClick": false,
    "leftRightClick": false
  }
}
```

| Option | Type | Default | Description |
| ------ | ---- | ------- | ----------- |
| `theme.size` | string | `normal` | `minimal` (dense; key hints hidden, search bar shown while searching), `normal`, `wide` |
| `theme.scrollbar` | string | `overlay` | List scrollbar: `overlay` (shown while scrolling), `always`, `hidden` (still scrolls) |
| `clipboardHistoryLimit` | number | `50` | Max clipboard items to store |
| `language` | string | `en` | Settings window language: `en` (English), `ja` (日本語) |
| `mouse.desktopDoubleClick` | boolean | `false` | Show the window on a left double click on the desktop background (needs uti for GNOME; touchpad taps are not detected) |
| `mouse.leftRightClick` | boolean | `false` | Show the window on a right click while holding left, anywhere. Mice with real buttons only; uti-daemon grabs them and re-emits their events, and the desktop's left long-press menu is disabled |

Colors follow the system style; `theme.color` and `theme.accentColor` are ignored.

### Snippets Data

Snippets are stored in `~/.config/uti/snippets.json`:

```json
{
  "items": [
    { "id": "my-email", "label": "My Email", "value": "user@example.com" }
  ]
}
```

- `id`: Unique identifier (any string; auto-generated UUID when added via UI)
- `label`: Optional display name (shows `value` if omitted)
- `value`: The actual text to copy

To add snippets manually, edit this file with any unique `id` string.

### Launcher Configuration

Launcher items: `~/.config/uti/launcher.json`. Without it, the Launcher shows installed apps
by category and power actions. Right-click an item (or press F2 / Del) to edit it from the popup.

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
<summary><strong>Panel Icon Not Visible (GNOME)</strong></summary>

Enable uti for GNOME (see GNOME Users section above).

</details>

## 🗑️ Uninstallation

```bash
systemctl --user disable --now uti-daemon.service
sudo dnf remove uti uti-daemon gnome-shell-extension-uti
```

Package removal cannot disable a per-user service, so disable it first.

To remove the COPR repository (optional):

```bash
sudo dnf copr remove noppomario/uti
```

To remove yourself from the input group (optional):

```bash
sudo gpasswd -d $USER input
```

## ⚠️ Known Limitations

- **Window appears in dock (non-GNOME)**: Hiding the window from the dash / Alt+Tab requires uti for GNOME.
- **Window position (non-GNOME)**: On non-GNOME Wayland environments (KDE, Sway, etc.), window always appears at screen center. Enable uti for GNOME for cursor positioning.
- **Recent files support**: Only apps that write to `recently-used.xbel` (GTK/GNOME apps) and VSCode are supported. KDE apps and most Electron apps are not supported.

## 🏗️ Architecture

| Component | Role |
| --------- | ---- |
| **uti-daemon** | Detects double Ctrl press via evdev |
| **uti** | Clipboard manager + App launcher UI (GTK4 / libadwaita) |
| **uti for GNOME** | GNOME Shell extension for panel menu, cursor positioning, and always-on-top control (optional) |

For details, see [Architecture Documentation](docs/ARCHITECTURE.md).

## 📦 What Gets Installed

When you install uti, the following changes are made to your system:

| Component | Location | Description |
| --------- | -------- | ----------- |
| **uti** | `/usr/bin/uti` | Main application (`uti` package) |
| **uti-daemon** | `/usr/bin/uti-daemon` | Keyboard daemon (`uti-daemon` package) |
| **User service** | `/usr/lib/systemd/user/uti-daemon.service` | Daemon autostart service (enabled per user) |
| **Config** | `~/.config/uti/` | User configuration, snippets, launcher config |
| **Data** | `~/.local/share/uti/` | Clipboard history |
| **Input group** | `/etc/group` | Your user is added to the `input` group |
| **udev rule** | `/usr/lib/udev/rules.d/99-uti-uinput.rules` | Enables uinput access for auto-paste |
| **uti for GNOME** | `/usr/share/gnome-shell/extensions/uti@noppomario.github.io/` | GNOME Shell extension, system-wide (`gnome-shell-extension-uti` package) |
| **COPR repo** | `/etc/yum.repos.d/` | `noppomario/uti` repository for `dnf upgrade` |

### ⚠️ About the Input Group

The daemon needs to read keyboard events from `/dev/input/*` devices and write to `/dev/uinput` for auto-paste. This requires membership in the **input group**. The installer automatically adds your user to this group.

**Security note**: Members of the input group can read all input devices (keyboard, mouse) and create virtual input devices. This is necessary for double-Ctrl detection and auto-paste to work.

## 👨‍💻 Development

For development setup, see [DEVELOPMENT.md](docs/DEVELOPMENT.md).

## 📄 License

[MIT](LICENSE)
