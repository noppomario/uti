# Architecture

This document describes the architecture of uti across different platforms and configurations.

## Overview

uti is a clipboard manager and app launcher that is invoked by pressing Ctrl twice quickly. Due to platform security restrictions (especially on Wayland), a multi-component architecture is required on Linux.

```mermaid
flowchart LR
    KB[Keyboard] -->|evdev read| DAEMON[uti-daemon]
    DAEMON -->|D-Bus signal| APP[uti]
    DAEMON -->|D-Bus signal| EXT[uti for GNOME]
    APP --> WIN[Window]
    EXT -->|position| WIN

    style KB fill:#e0e0e0,stroke:#666
    style WIN fill:#e0e0e0,stroke:#666
    style DAEMON fill:#4a9eff,stroke:#2171c7,color:#fff
    style APP fill:#4a9eff,stroke:#2171c7,color:#fff
    style EXT fill:#4a9eff,stroke:#2171c7,color:#fff
```

## Requirements

- **Linux** with D-Bus session bus and glibc
- **systemd** for daemon service management
- User in `input` group for keyboard access

**Optional**: "uti for GNOME" extension enables tray icon, cursor positioning, and always-on-top control on **GNOME/Wayland**. Without it, window appears at screen center and pin button only disables auto-hide.

---

## Components

### uti-daemon

Rust daemon that monitors keyboard input, detects double Ctrl press, and provides
virtual keyboard for auto-paste functionality.

| Property | Value |
| -------- | ----- |
| Language | Rust |
| Input | evdev (`/dev/input/event*`), D-Bus signals |
| Output | D-Bus signal, uinput (`/dev/uinput`) |
| Permissions | `input` group membership |
| Service | `uti-daemon.service` (systemd user) |

**D-Bus Interface:**

- Name: `io.github.noppomario.uti`
- Interface: `io.github.noppomario.uti.DoubleTap`
- Path: `/io/github/noppomario/uti/DoubleTap`
- Signals:
  - `Triggered()` - emitted on double Ctrl press
  - `SetAlwaysOnTop(enabled: bool)` - emitted by uti app when pin state changes

### uti (Tauri App)

Main application with clipboard history and app launcher UI.

| Property | Value |
| -------- | ----- |
| Frontend | React 19 + TypeScript |
| Backend | Rust + Tauri 2 |
| IPC | D-Bus (receive), StatusNotifierItem (tray) |
| Config | `~/.config/uti/config.json` |
| Launcher | `~/.config/uti/launcher.json` |
| Clipboard | `~/.config/uti/clipboard.json` |
| Snippets | `~/.config/uti/snippets.json` |

**Snippets Feature:**

- Pin clipboard items via star icon for quick access
- Items immediately appear in Snippets tab
- Pinned items removed from Clipboard on window close
- Manual editing via JSON file supported

**Launcher Features:**

- Configurable application commands with keyboard navigation
- Jump lists showing recent files from system history (recently-used.xbel)
- VSCode recent files support via SQLite database

### uti for GNOME

GNOME Shell extension that provides:

1. **Tray icon display** - Acts as StatusNotifierHost to show Tauri's tray
2. **Cursor positioning** - Moves window to cursor location on toggle
3. **Always-on-top control** - Handles `SetAlwaysOnTop` signal to set window layer via `Meta.Window.make_above()` (Mutter ignores app-level always-on-top requests on Wayland)

| Property | Value |
| -------- | ----- |
| UUID | `uti@noppomario.github.io` |
| Settings | GSettings (`org.gnome.shell.extensions.uti`) |
| Protocol | StatusNotifierItem/DBusMenu |

---

## Linux: GNOME + Wayland (Primary)

The recommended configuration for GNOME desktop.

### Toggle Window Sequence

```mermaid
sequenceDiagram
    box rgb(74, 158, 255) uti components
        participant Daemon as uti-daemon
        participant App as uti
        participant Ext as GNOME Extension
    end
    participant DBus as D-Bus
    participant Mutter

    Daemon->>DBus: Emit DoubleTap.Triggered
    par Broadcast
        DBus->>App: Signal received
        DBus->>Ext: Signal received
    end
    App->>App: Toggle window visibility
    Ext->>Ext: Get cursor position
    Ext->>Mutter: Move window to cursor
```

### Tray Icon Sequence

```mermaid
sequenceDiagram
    box rgb(74, 158, 255) uti components
        participant App as uti
        participant Ext as GNOME Extension
    end
    participant DBus as D-Bus

    App->>DBus: Register StatusNotifierItem
    Ext->>DBus: Watch for SNI names
    DBus->>Ext: uti SNI appeared
    Ext->>DBus: Get icon and menu
    Ext->>Ext: Display in GNOME Panel
```

### Pin State Toggle Sequence

```mermaid
sequenceDiagram
    participant User
    box rgb(74, 158, 255) uti components
        participant App as uti
        participant Ext as GNOME Extension
    end
    participant DBus as D-Bus
    participant Mutter

    User->>App: Click pin button
    App->>App: Store pin state
    App->>DBus: Emit SetAlwaysOnTop(true)
    DBus->>Ext: Signal received
    Ext->>Mutter: make_above()
```

### Prompt Auto-Paste Sequence

```mermaid
sequenceDiagram
    participant User
    box rgb(74, 158, 255) uti components
        participant App as uti
        participant Daemon as uti-daemon
    end
    participant DBus as D-Bus
    participant Target as Active Window

    User->>App: Ctrl+Enter in Prompt tab
    App->>App: Copy to clipboard
    App->>App: Hide window
    App->>DBus: Emit TypeText
    DBus->>Daemon: Signal received
    Daemon->>Target: Ctrl+V via uinput
```

---

## D-Bus Interfaces

### DoubleTap Interface

Shared interface for daemon-to-app and app-to-extension communication:

```xml
<interface name="io.github.noppomario.uti.DoubleTap">
  <signal name="Triggered"/>
  <signal name="SetAlwaysOnTop">
    <arg name="enabled" type="b"/>
  </signal>
  <signal name="TypeText"/>
</interface>
```

| Signal | Sender | Receiver | Purpose |
| ------ | ------ | -------- | ------- |
| `Triggered` | uti-daemon | uti, GNOME Extension | Double Ctrl press detected |
| `SetAlwaysOnTop` | uti | GNOME Extension | Pin state changed |
| `TypeText` | uti | uti-daemon | Trigger auto-paste via Ctrl+V |

### StatusNotifierItem (App → Extension)

The Tauri app registers as a StatusNotifierItem on the session bus:

- Bus name: `org.kde.StatusNotifierItem-{pid}-1`
- Object path: `/StatusNotifierItem`
- Interfaces: `org.kde.StatusNotifierItem`, `org.freedesktop.DBusMenu`

The extension acts as a StatusNotifierHost, watching for this name and proxying the icon and menu to the GNOME panel.

---

## File Locations

| File | Purpose |
| ---- | ------- |
| `/usr/bin/uti` | Main application |
| `/usr/bin/uti-daemon` | Keyboard daemon |
| `/etc/udev/rules.d/99-uti-uinput.rules` | uinput access for auto-paste |
| `~/.config/systemd/user/uti-daemon.service` | Daemon service |
| `~/.config/uti/config.json` | User configuration |
| `~/.config/uti/launcher.json` | Launcher commands |
| `~/.config/uti/clipboard.json` | Clipboard history |
| `~/.config/uti/snippets.json` | Pinned snippets |
| `~/.local/share/gnome-shell/extensions/uti@noppomario.github.io/` | GNOME extension |
