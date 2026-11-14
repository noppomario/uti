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
- **i18n**: react-i18next (settings window only)

### Backend

- **Framework**: Tauri 2
- **Language**: Rust 2021
- **IPC**: D-Bus session bus (owns `io.github.noppomario.uti`)
- **Tray**: none; the GNOME extension's panel menu calls the app
- **Application ID**: `io.github.noppomario.uti`

### Daemon (uti-daemon)

- **Language**: Rust 2021
- **Input**: evdev (keyboard monitoring)
- **IPC**: D-Bus `io.github.noppomario.uti.Daemon1` (bus `io.github.noppomario.uti.Daemon`)
- **Permissions**: Requires `input` group for `/dev/input/*` access

### GNOME Extension (uti for GNOME)

- **Language**: JavaScript (GJS)
- **UUID**: `uti@noppomario.github.io`
- **Role**: `Shell1` service (positioning, always-on-top, paste, clipboard watch) + panel menu
- **Settings**: GSettings (`org.gnome.shell.extensions.uti`)

## Key Technical Decisions

For detailed rationale, see ADRs in `decisions.md` (active) or
`.claude/archive/adr-archive.md` (archived).

Key archived decisions:

- **ADR-001**: Biome over ESLint + Prettier
- **ADR-004**: D-Bus for IPC
- **ADR-016**: Custom GNOME extension (active, in decisions.md)

## Repository Structure

See [ARCHITECTURE.md](../../../docs/ARCHITECTURE.md) for detailed structure.

**Key directories:**

- `app/` - Tauri frontend (React + TypeScript)
- `app/src-tauri/` - Rust backend
- `daemon/` - uti-daemon (evdev + D-Bus)
- `gnome-extension/` - GNOME Shell extension

## Important Constraints

### Privacy

- **No personal information** in any commits
- All commits in English
- Author field is empty or anonymous
- Git history has been rewritten to remove personal info

### Naming Conventions

- D-Bus interfaces: see `docs/ARCHITECTURE.md` (D-Bus Interfaces)
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
    "size": "normal",
    "accentColor": "#3584e4"
  },
  "clipboardHistoryLimit": 50,
  "language": "en"
}
```

**Validation**:

- `theme.color`: "midnight", "dark", or "light" (default: "dark")
- `theme.size`: "minimal", "normal", or "wide" (default: "normal")
- `theme.accentColor`: Hex color string (optional, uses theme default if not set)
- `clipboardHistoryLimit`: Must be > 0 (default: 50)
- `language`: "en" or "ja" (default: "en")

**Clipboard history**: `~/.local/share/uti/clipboard.json` (`$XDG_DATA_HOME`);
moved from `~/.config/uti/` on first start. All files are written atomically
with mode 0600 (`uti_core::fs::write_atomic`).

**Auto-migration**: On startup, `clipboard.json` max_items syncs with
`clipboardHistoryLimit`

**Implementation**:

- Rust: `AppConfig::load()` at startup
- TypeScript: `getConfig()` async function

**Snippets storage file**: `~/.config/uti/snippets.json`

```json
{
  "items": [
    { "id": "my-email", "label": "Email", "value": "user@example.com" }
  ]
}
```

- `id`: Unique identifier (any string; auto-generated UUID when added via UI)
- `label`: Optional display name (shows value if empty/null)
- `value`: The actual text to copy to clipboard

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

## Panel Menu

The "uti for GNOME" extension shows a panel button (left click toggles) with
Show/Hide, Settings, Check for Updates, GitHub and Quit. The Tauri SNI tray was
removed in Phase 1.

## Known Limitations

1. Fixed 300ms double-tap interval
2. Linux-only, GNOME 50 for the extension
3. Window position/size not persisted
4. Positioning, always-on-top, hiding from dash and the panel menu require
   "uti for GNOME" (without it, pin only disables auto-hide)

## Roadmap

See [ARCHITECTURE.md](../../../docs/ARCHITECTURE.md) for platform support plans.
