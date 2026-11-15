# Architecture

This document describes the architecture of uti across different platforms and configurations.

## Overview

uti is a clipboard manager and app launcher that is invoked by pressing Ctrl twice quickly. Due to platform security restrictions (especially on Wayland), a multi-component architecture is required on Linux.

```mermaid
flowchart LR
    KB[Keyboard] -->|evdev read| DAEMON[uti-daemon]
    DAEMON -->|Daemon1.Triggered| APP[uti]
    APP -->|Shell1 methods| EXT[uti for GNOME]
    EXT -->|ActivateAction| APP
    EXT -->|position / above / paste| WIN[Window]
    APP --> WIN

    style KB fill:#e0e0e0,stroke:#666
    style WIN fill:#e0e0e0,stroke:#666
    style DAEMON fill:#4a9eff,stroke:#2171c7,color:#fff
    style APP fill:#4a9eff,stroke:#2171c7,color:#fff
    style EXT fill:#4a9eff,stroke:#2171c7,color:#fff
```

The app owns state; the daemon and the extension are services it calls.

## Requirements

- **Linux** with D-Bus session bus and glibc
- **systemd** for daemon service management
- User in `input` group for keyboard access

**Optional**: "uti for GNOME" extension enables the panel menu, cursor positioning, always-on-top, hiding from dash / Alt+Tab, and paste on **GNOME/Wayland**. Without it, window appears at screen center and pin button only disables auto-hide.

---

## Components

### uti-daemon

Rust daemon that monitors keyboard input, detects double Ctrl press, and provides
virtual keyboard for auto-paste functionality.

| Property | Value |
| -------- | ----- |
| Language | Rust |
| Input | evdev (`/dev/input/event*`, hotplug via inotify) |
| Output | `Daemon1.Triggered`, uinput (`/dev/uinput`) |
| Permissions | `input` group membership |
| Service | `uti-daemon.service` (systemd user) |

A tap is a short Ctrl press with no other key in between; two taps within
300 ms trigger (`daemon/src/detector.rs`).

### uti (GTK App)

Main application with Clipboard, Snippets and Launcher tabs, plus a settings window.

| Property | Value |
| -------- | ----- |
| Toolkit | Rust + gtk4-rs 0.11 + libadwaita 0.9 (`app/`) |
| Structure | `app.rs` controller (`Msg` loop), `ui.rs`, `dbus.rs`, `settings.rs`, `state/` (pure UI logic), `config/` `clipboard/` `snippets/` `launcher/` (data files) |
| IPC | D-Bus: owns `io.github.noppomario.uti`, serves `ActivateAction`, calls `Shell1` / `Daemon1` |
| Config | `~/.config/uti/config.json` |
| Launcher | `~/.config/uti/launcher.json` |
| Clipboard | `~/.local/share/uti/clipboard.json` (moved from `~/.config/uti/` on first start) |
| Snippets | `~/.config/uti/snippets.json` |

**Snippets Feature:**

- Pin clipboard items via star icon for quick access
- Items immediately appear in Snippets tab
- Pinned items removed from Clipboard on window close
- Manual editing via JSON file supported

**Launcher Features:**

- Configurable application commands and installed app search (`gio::AppInfo`) with keyboard navigation
- Jump lists showing recent files from system history (recently-used.xbel)
- VSCode recent files support via SQLite database

### uti for GNOME

GNOME Shell extension that provides:

1. **Panel menu** - Show/Hide, Settings, Check for Updates, GitHub, Quit (calls the app's `ActivateAction`)
2. **Cursor positioning** - Moves the main window to the pointer on its first frame
3. **Always-on-top** - `Meta.Window.make_above()` (Mutter ignores app-level requests on Wayland)
4. **Hide from dash / Alt+Tab** - `Meta.Window.hide_from_window_list()`
5. **Paste** - Clutter virtual keyboard after focus leaves uti
6. **Clipboard watch** - `Meta.Selection` owner changes, emitted as `ClipboardChanged`

| Property | Value |
| -------- | ----- |
| UUID | `uti@noppomario.github.io` |
| Settings | GSettings (`org.gnome.shell.extensions.uti`) |
| Shell versions | 50, 51 |

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
    participant Mutter

    Daemon->>App: Daemon1.Triggered
    App->>Ext: Shell1.PositionAtPointer
    Ext->>Ext: Keep request pending
    App->>App: Show window
    Mutter->>Ext: first-frame of main window
    Ext->>Mutter: move_frame + hide_from_window_list
```

---

## D-Bus Interfaces

| Owner | Bus name | Object path | Interface | Members |
| ----- | -------- | ----------- | --------- | ------- |
| uti-daemon | `io.github.noppomario.uti.Daemon` | `/io/github/noppomario/uti/Daemon` | `io.github.noppomario.uti.Daemon1` | signal `Triggered()`, method `Paste(b plain, b submit)` |
| uti | `io.github.noppomario.uti` | `/io/github/noppomario/uti` | `org.freedesktop.Application` | `ActivateAction(s, av, a{sv})`: `toggle`, `show`, `hide`, `settings`, `check-updates`, `quit` |
| GNOME Extension | `io.github.noppomario.uti.Shell` | `/io/github/noppomario/uti/Shell` | `io.github.noppomario.uti.Shell1` | `PositionAtPointer(b activate)`, `SetAbove(b)`, `Paste(b plain, b submit)`, signal `ClipboardChanged(s)` |

`Daemon1.Paste` and all `Shell1` methods reject callers that do not own
`io.github.noppomario.uti` (`AccessDenied`). Nothing in the app calls `Paste`
since the Prompt tab was removed.

---

## File Locations

| File | Purpose |
| ---- | ------- |
| `/usr/bin/uti` | Main application |
| `/usr/bin/uti-daemon` | Keyboard daemon |
| `/usr/lib/udev/rules.d/99-uti-uinput.rules` | uinput access for auto-paste |
| `/usr/lib/systemd/user/uti-daemon.service` | Daemon service (enabled per user) |
| `~/.config/uti/config.json` | User configuration |
| `~/.config/uti/launcher.json` | Launcher commands |
| `~/.local/share/uti/clipboard.json` | Clipboard history |
| `~/.config/uti/snippets.json` | Pinned snippets |
| `/usr/share/gnome-shell/extensions/uti@noppomario.github.io/` | GNOME extension |
| `/usr/share/glib-2.0/schemas/org.gnome.shell.extensions.uti.gschema.xml` | Extension settings schema |
