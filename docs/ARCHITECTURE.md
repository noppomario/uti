# Architecture

This document describes the architecture of uti across different platforms and configurations.

## Overview

uti is a clipboard manager that is invoked by pressing Ctrl twice quickly. Due to platform security restrictions (especially on Wayland), a multi-component architecture is required on Linux.

```mermaid
flowchart LR
    KB[Keyboard] -->|evdev| DAEMON[uti-daemon]
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

**Optional**: "uti for GNOME" extension enables tray icon and cursor positioning on **GNOME/Wayland**. Without it, window appears at screen center.

---

## Components

### uti-daemon

Rust daemon that monitors keyboard input and detects double Ctrl press.

| Property | Value |
| -------- | ----- |
| Language | Rust |
| Input | evdev (`/dev/input/event*`) |
| Output | D-Bus signal |
| Permissions | `input` group membership |
| Service | `uti-daemon.service` (systemd user) |

**D-Bus Interface:**

- Name: `io.github.noppomario.uti`
- Interface: `io.github.noppomario.uti.DoubleTap`
- Path: `/io/github/noppomario/uti/DoubleTap`
- Signal: `Triggered()` - emitted on double Ctrl press

### uti (Tauri App)

Main application with clipboard history UI.

| Property | Value |
| -------- | ----- |
| Frontend | React 19 + TypeScript |
| Backend | Rust + Tauri 2 |
| IPC | D-Bus (receive), StatusNotifierItem (tray) |
| Config | `~/.config/uti/config.json` |
| Data | `~/.config/uti/clipboard.json` |

### uti for GNOME

GNOME Shell extension that provides:

1. **Tray icon display** - Acts as StatusNotifierHost to show Tauri's tray
2. **Cursor positioning** - Moves window to cursor location on toggle

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
    participant Daemon as uti-daemon
    participant DBus as D-Bus
    participant App as uti
    participant Ext as GNOME Extension
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
    participant App as uti
    participant DBus as D-Bus
    participant Ext as GNOME Extension

    App->>DBus: Register StatusNotifierItem
    Ext->>DBus: Watch for SNI names
    DBus->>Ext: uti SNI appeared
    Ext->>DBus: Get icon and menu
    Ext->>Ext: Display in GNOME Panel
```

---

## D-Bus Interfaces

### DoubleTap Interface (Daemon → App)

```xml
<interface name="io.github.noppomario.uti.DoubleTap">
  <signal name="Triggered"/>
</interface>
```

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
| `~/.config/systemd/user/uti-daemon.service` | Daemon service |
| `~/.config/uti/config.json` | User configuration |
| `~/.config/uti/clipboard.json` | Clipboard history |
| `~/.local/share/gnome-shell/extensions/uti@noppomario.github.io/` | GNOME extension |
