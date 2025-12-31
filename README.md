# uti

A desktop utility that toggles window visibility with double Ctrl press.

## Quick Start

Install with one command:

```bash
curl -fsSL https://raw.githubusercontent.com/noppomario/uti/main/install.sh | bash
```

After installation, log out and log back in, then run `uti`.

## Features

- **Double Ctrl Toggle**: Press Ctrl twice quickly (within 300ms) to show/hide
  the window
- **Clipboard History**: Stores clipboard items for quick access
- **System Tray**: Runs in the background with tray icon control
- **Auto-start**: Optional auto-start on login
- **Self-update**: Update via `uti update` command or tray menu

## Usage

### Basic Operation

1. Press **Ctrl twice quickly** to toggle window visibility
2. Click on clipboard items to copy them back
3. Use arrow keys to navigate, Enter to select

### System Tray

Right-click the tray icon for options:

- **Show/Hide**: Toggle window visibility
- **Auto-start**: Enable/disable start on login
- **Check for Updates...**: Check for new versions
- **GitHub**: Open project page
- **Quit**: Exit application

### Updating

Check for and install updates:

```bash
uti update
```

Check only (without installing):

```bash
uti update --check
```

## Configuration

Configuration file: `~/.config/uti/config.json`

```json
{
  "theme": "dark",
  "clipboardHistoryLimit": 50,
  "showTooltip": true,
  "tooltipDelay": 500
}
```

| Option                  | Type    | Default | Description                              |
|-------------------------|---------|---------|------------------------------------------|
| `theme`                 | string  | `dark`  | UI theme: `dark` or `light`              |
| `clipboardHistoryLimit` | number  | `50`    | Max clipboard items to store             |
| `showTooltip`           | boolean | `true`  | Show tooltip on hover                    |
| `tooltipDelay`          | number  | `500`   | Tooltip delay in milliseconds            |

## System Requirements

- **OS**: Linux (Fedora 43+ recommended)
- **Desktop**: GNOME (Wayland/X11), KDE, XFCE
- **Architecture**: x86_64

### GNOME Users

GNOME 43+ requires the AppIndicator extension for system tray:

1. Install Extension Manager (if not installed):
   - Open GNOME Software
   - Search "Extension Manager" and install

2. Install AppIndicator extension:
   - Open Extension Manager
   - Search "AppIndicator and KStatusNotifierItem Support"
   - Click Install

3. Log out and back in

## Troubleshooting

### Daemon Won't Start

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
systemctl --user status double-ctrl.service
journalctl --user -u double-ctrl.service -n 50
```

### Tray Icon Not Visible (GNOME)

Install AppIndicator extension via Extension Manager (see GNOME Users section
above).

## Uninstallation

```bash
sudo dnf remove uti double-ctrl
```

This automatically stops and disables the daemon service.

To remove yourself from the input group (optional):

```bash
sudo gpasswd -d $USER input
```

## Development

For development setup and contribution guidelines, see
[DEVELOPMENT.md](./DEVELOPMENT.md).

## Known Limitations

- **Window appears in dock (Wayland)**: On Wayland, the window appears in the
  dock when visible. This is a Tauri limitation
  ([#9829](https://github.com/tauri-apps/tauri/issues/9829)).
- **Window position**: Window always appears at screen center (Wayland does
  not support cursor-relative positioning).
