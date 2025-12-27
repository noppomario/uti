# uti - Project Context

## Overview
Desktop utility for Linux that toggles window visibility with double Ctrl press.

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

```
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

```
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

### Current Status
- No automated tests yet
- Manual testing only
- Code quality enforced by Biome linter

### Future Considerations
- Add integration tests when features grow
- CI/CD setup when team expands
- End-to-end testing for D-Bus communication

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
