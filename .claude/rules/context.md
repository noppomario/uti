# uti - Project Context

## Overview

Desktop utility for Linux that toggles window visibility with double Ctrl press.

**For development environment setup**, see [DEVELOPMENT.md](../../../DEVELOPMENT.md)

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
- **Application ID**: `io.github.noppomario.uti`

### Daemon

- **Language**: Rust 2021
- **Input**: evdev (keyboard monitoring)
- **IPC**: D-Bus (`io.github.noppomario.uti.DoubleTap` interface)
- **Permissions**: Requires root/input group for `/dev/input/*` access

## Architecture

```text
[double-ctrl daemon]  ← root/input permissions
  ↓ Monitor Ctrl via evdev
  ↓ Detect double press within 300ms
  ↓ Send D-Bus Signal
[Tauri app]           ← Normal user permissions
  ↓ Receive D-Bus signal
  ↓ Toggle window visibility
```

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
- Current scale: 3 files, 1 component
- Will refactor when we have 5+ components

## Repository Structure

```text
uti/
├── .vscode/              # VSCode configuration
├── app/                  # Tauri frontend
│   ├── src/
│   │   ├── App.tsx      # Main component (56 lines)
│   │   ├── main.tsx     # Entry point (20 lines)
│   │   └── index.css    # Tailwind imports
│   └── src-tauri/       # Rust backend
│       └── src/main.rs  # D-Bus listener + window control
├── daemon/
│   └── src/main.rs      # evdev keyboard monitor + D-Bus sender
└── README.md            # English documentation
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
- GitHub repo: `noppomario/uti`

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
- systemd service for daemon
- No packaging yet

### Future

- .rpm package for Fedora
- .deb package for Ubuntu/Debian
- AUR package for Arch Linux
- Automated releases via GitHub Actions

## Known Limitations

1. Single keyboard device detection (first found)
2. Fixed 300ms double-tap interval
3. No configuration file support
4. Linux-only (Wayland/X11)
5. No system tray icon
6. Window position/size not persisted

## Roadmap

See TODO in README.md for planned features.
