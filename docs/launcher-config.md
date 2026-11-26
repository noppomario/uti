# Launcher Configuration Guide

This document explains how to configure `~/.config/uti/launcher.json`.

## File Location

```text
~/.config/uti/launcher.json
```

## Basic Structure

`items` is a tree. Folders open beside their row (hover or →), like classic cascading menus.

```json
{
  "items": [
    { "type": "folder", "name": "Dev", "children": [
      { "type": "command", "name": "Terminal", "command": "ptyxis", "args": ["--new-window"] }
    ]},
    { "type": "separator" },
    { "type": "command", "name": "Files", "command": "nautilus", "historySource": { ... } }
  ]
}
```

## Item Types

| `type`      | Fields                                                  | Row                                     |
| ----------- | ------------------------------------------------------- | --------------------------------------- |
| `folder`    | `name`, `children` (items)                              | Opens its children                      |
| `command`   | `name`, `command`, `args` (default `[]`), `historySource` | Launches; recent files open as children |
| `separator` | -                                                       | A line                                  |

Every item may have an `id`; uti generates one when it is missing. Items of an unknown `type`
are skipped.

A file with the older `"commands": [...]` array (commands only) is still read as `command` items.

## History Source Types

### 1. System XBEL (`recently-used`)

Uses the freedesktop standard `~/.local/share/recently-used.xbel` file.
**Requires `appName`** to filter entries for a specific application.

```json
{
  "historySource": {
    "type": "recently-used",
    "appName": "org.gnome.Nautilus"
  }
}
```

To find available app names:

```bash
grep -oP 'application name="\K[^"]+' ~/.local/share/recently-used.xbel | sort -u
```

### 2. Per-App XBEL (`recently-used` with `path`)

Some applications store their own XBEL file separately.
When `path` is specified, `appName` becomes **optional**.

```json
{
  "historySource": {
    "type": "recently-used",
    "path": "~/.local/share/org.gnome.TextEditor/recently-used.xbel"
  }
}
```

You can still add `appName` for filtering if needed:

```json
{
  "historySource": {
    "type": "recently-used",
    "appName": "gnome-text-editor",
    "path": "~/.local/share/org.gnome.TextEditor/recently-used.xbel"
  }
}
```

### 3. VSCode (`vscode`)

Reads recent files from VSCode's SQLite database.

```json
{
  "historySource": {
    "type": "vscode",
    "path": "~/.config/Code/User/globalStorage/state.vscdb"
  }
}
```

Common paths:

| VSCode Version  | Path                                                                           |
| --------------- | ------------------------------------------------------------------------------ |
| VSCode          | `~/.config/Code/User/globalStorage/state.vscdb`                                |
| VSCode Insiders | `~/.config/Code - Insiders/User/globalStorage/state.vscdb`                     |
| VSCodium        | `~/.config/VSCodium/User/globalStorage/state.vscdb`                            |
| Flatpak VSCode  | `~/.var/app/com.visualstudio.code/config/Code/User/globalStorage/state.vscdb`  |

### 4. No History Source

For applications without recent files (e.g., terminals):

```json
{
  "type": "command",
  "name": "Terminal",
  "command": "gnome-terminal",
  "args": ["--new-window"]
}
```

Commands without `historySource` have no children (no `›` marker).

## Complete Example

See [launcher.example.json](launcher.example.json) for a complete working example.

## Tips

### Finding XBEL Paths

Some GNOME apps store their own XBEL files:

```bash
find ~/.local/share -name "recently-used.xbel" 2>/dev/null
```

### Finding App Names in XBEL

```bash
# List all app names in system XBEL
grep -oP 'application name="\K[^"]+' ~/.local/share/recently-used.xbel | sort -u

# Check if a specific app exists
grep "org.gnome.Nautilus" ~/.local/share/recently-used.xbel
```

### Args Behavior

The `args` array is used for:

1. **Direct launch** (clicking the app item): `command + args`
2. **File launch** (clicking one of its recent files): `command + args + filePath`

If you want different behavior for file launch, consider removing `--new-window`
from `args` to let the app decide how to open the file.

## Supported Apps

Recent files are supported for:

| Type               | Apps                                        | Notes                                          |
| ------------------ | ------------------------------------------- | ---------------------------------------------- |
| **GTK/GNOME apps** | Nautilus, gnome-text-editor, Firefox, etc.  | Writes to `~/.local/share/recently-used.xbel`  |
| **VSCode**         | Code, Code Insiders, VSCodium               | Uses SQLite database                           |

**Not supported:**

- KDE apps (use different recent files mechanism)
- Most Electron apps (except VSCode)
- Flatpak apps (sandboxed, different file paths)

## Troubleshooting

### Recent files are empty

1. Check if `appName` matches exactly (case-sensitive)
2. For per-app XBEL, verify the file exists at the specified `path`
3. For VSCode, ensure the `state.vscdb` file exists

### App launches but doesn't show the file

This is application-dependent behavior. Some apps require specific arguments
to open files in existing windows vs new windows.
