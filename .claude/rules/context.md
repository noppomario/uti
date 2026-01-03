# uti - Project Context

## Overview

Desktop utility for Linux that toggles window visibility with double Ctrl press.

**For development environment setup**, see [DEVELOPMENT.md](../../../docs/DEVELOPMENT.md)

**For architecture and platform configurations**, see [ARCHITECTURE.md](../../../docs/ARCHITECTURE.md)

## Technology Stack

### Frontend

- **Framework**: React 19
- **Language**: TypeScript 5.7
- **Styling**: Tailwind CSS v4
- **Build**: Vite 6 + esbuild
- **Linter/Formatter**: Biome 2.3 (Rust-based, 25-100x faster than ESLint+Prettier)

### Backend

- **Framework**: Tauri 2
- **Language**: Rust 2021
- **IPC**: D-Bus session bus
- **Tray**: StatusNotifierItem protocol
- **Application ID**: `io.github.noppomario.uti`

### Daemon (uti-daemon)

- **Language**: Rust 2021
- **Input**: evdev (keyboard monitoring)
- **IPC**: D-Bus (`io.github.noppomario.uti.DoubleTap` interface)
- **Permissions**: Requires `input` group for `/dev/input/*` access

### GNOME Extension (uti for GNOME)

- **Language**: JavaScript (GJS)
- **UUID**: `uti@noppomario.github.io`
- **Role**: StatusNotifierHost + window positioning
- **Settings**: GSettings (`org.gnome.shell.extensions.uti`)

## Key Technical Decisions

### Why Biome instead of ESLint + Prettier?

- Rust-based, significantly faster
- Single tool for linting + formatting
- Better integration with modern tooling

### Why NOT oxlint/oxfmt/tsgo?

- Project is too small (77 lines of TS)
- tsgo is still in preview (unstable)
- Current tools are fast enough
- Stability over bleeding-edge performance

### Why NOT complex folder structure?

- YAGNI principle - don't add complexity until needed
- Current scale: 21 TypeScript files (~2,900 lines), 6 components, 3 hooks
- Already organized with `components/` and `hooks/` directories
- Will add feature-based structure when 4+ distinct features exist

### Why custom GNOME extension instead of AppIndicator?

- **Tray**: Extension acts as StatusNotifierHost, displaying Tauri's tray
  directly without requiring AppIndicator extension
- **Positioning**: Wayland doesn't allow apps to query cursor position;
  GNOME extension can use `global.get_pointer()` and `Meta.Window.move_frame()`
- **Single dependency**: One extension handles both tray and positioning

## Repository Structure

```text
uti/
├── .vscode/                    # VSCode configuration
├── app/                        # Tauri frontend
│   ├── src/
│   │   ├── components/         # React components
│   │   │   ├── ClipboardHistory.tsx  # Clipboard history list
│   │   │   ├── Launcher.tsx    # Launcher command list
│   │   │   ├── JumpList.tsx    # Recent files list
│   │   │   ├── TabBar.tsx      # Tab navigation
│   │   │   └── ListItem.tsx    # Shared list item component
│   │   ├── hooks/              # Custom React hooks
│   │   │   ├── useClipboard.ts # Clipboard monitoring
│   │   │   ├── useLauncher.ts  # Launcher config loading
│   │   │   └── useListKeyboardNavigation.ts  # Keyboard nav
│   │   ├── App.tsx             # Main component
│   │   ├── main.tsx            # Entry point + theme init
│   │   ├── config.ts           # Configuration loader
│   │   └── index.css           # Tailwind v4 theme
│   └── src-tauri/              # Rust backend
│       └── src/
│           ├── main.rs         # D-Bus listener + window control + tray
│           ├── lib.rs          # Library exports
│           ├── clipboard/      # Clipboard module
│           │   ├── mod.rs      # ClipboardItem definition
│           │   └── store.rs    # LRU clipboard storage
│           ├── launcher/       # Launcher module
│           │   ├── mod.rs      # LauncherConfig, HistorySource
│           │   ├── store.rs    # Config file loading
│           │   └── recent_files.rs  # XBEL/VSCode history
│           └── updater.rs      # Self-update functionality
├── daemon/
│   ├── src/main.rs             # evdev keyboard monitor + D-Bus sender
│   ├── systemd/                # systemd user service
│   └── uti-daemon.spec         # RPM spec
├── gnome-extension/            # GNOME Shell extension
│   ├── extension.js            # StatusNotifierHost + positioning
│   ├── metadata.json           # Extension metadata
│   └── schemas/                # GSettings schema
└── README.md                   # English documentation
```

## Important Constraints

### Privacy

- **No personal information** in any commits
- All commits in English
- Author field is empty or anonymous
- Git history has been rewritten to remove personal info

### Naming Conventions

- D-Bus interface: `io.github.noppomario.uti.DoubleTap`
- D-Bus path: `/io/github/noppomario/uti/DoubleTap`
- Application ID: `io.github.noppomario.uti`
- GNOME extension UUID: `uti@noppomario.github.io`
- GitHub repo: `noppomario/uti`

### Configuration Management

**User configuration file**: `~/.config/uti/config.json`

**Format**:

```json
{
  "theme": {
    "color": "dark",
    "size": "normal"
  },
  "clipboardHistoryLimit": 50
}
```

**Validation**:

- `theme.color`: "midnight", "dark", or "light" (default: "dark")
- `theme.size`: "minimal", "normal", or "wide" (default: "normal")
- `clipboardHistoryLimit`: Must be > 0 (default: 50)

**Auto-migration**: On startup, `clipboard.json` max_items syncs with
`clipboardHistoryLimit`

**Implementation**:

- Rust: `AppConfig::load()` at startup
- TypeScript: `getConfig()` async function

## Development Environment

### Required Tools

- Rust (latest stable)
- Bun (latest)
- VSCode with recommended extensions:
  - rust-analyzer
  - tauri-vscode
  - biomejs.biome
  - bradlc.vscode-tailwindcss
  - editorconfig.editorconfig

### Format on Save

Enabled in `.vscode/settings.json` for all file types.

### Build Scripts

```bash
bun run dev         # Development mode
bun run build       # Type check + build (safe)
bun run build:fast  # Skip type check (ultra fast)
bun run check:fix   # Biome lint + format + auto-fix
```

## Testing & Quality

### Test Framework

- **Frontend**: Vitest + React Testing Library + happy-dom
- **Rust**: Standard `cargo test` framework
- **Methodology**: Test-Driven Development (TDD) required for all new code

### TDD Workflow (Red-Green-Refactor)

1. **Red**: Write failing test first
2. **Green**: Implement minimal code to pass
3. **Refactor**: Improve code while keeping tests green

### Test Execution

```bash
# Frontend tests
cd app && bun run test          # Run once
cd app && bun run test:watch    # Watch mode (TDD)
cd app && bun run test:coverage # Coverage report

# Rust tests
cargo test --workspace          # All Rust tests
cd daemon && cargo test         # Daemon only

# All tests (from root)
bun run all:test               # Parallel execution
```

### CI Integration

- Tests run automatically in GitHub Actions
- Coverage reports generated for all commits
- PRs require passing tests

### Test Priorities

1. **Daemon**: Keyboard event detection (300ms double-tap logic)
2. **Frontend**: Window control UI components
3. **Integration**: D-Bus communication end-to-end

## Deployment

### Current

- Manual installation via `install.sh`
- systemd user service for daemon
- RPM packages for Fedora

### Future

- .deb package for Ubuntu/Debian
- AUR package for Arch Linux
- Flatpak

## System Tray

**Status**: Implemented via StatusNotifierItem protocol

- Tray icon with context menu (Show/Hide, Auto-start, Updates, GitHub, Quit)
- Left-click toggles window visibility
- Right-click shows menu

**GNOME support**:

- **uti for GNOME** (recommended): Extension acts as StatusNotifierHost
- **AppIndicator** (alternative): Third-party extension

## Known Limitations

1. Single keyboard device detection (first found)
2. Fixed 300ms double-tap interval
3. Linux-only (Wayland/X11)
4. Window position/size not persisted
5. Window appears in dock/taskbar on Wayland (Tauri limitation, see
   [#9829](https://github.com/tauri-apps/tauri/issues/9829))

## Roadmap

See [ARCHITECTURE.md](../../../docs/ARCHITECTURE.md) for platform support plans.
