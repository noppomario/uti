# uti - Project Context

## Overview

Desktop utility for Linux that toggles window visibility with double Ctrl press.

**For development environment setup**, see [DEVELOPMENT.md](../../docs/DEVELOPMENT.md)

**For architecture and platform configurations**, see [ARCHITECTURE.md](../../docs/ARCHITECTURE.md)

## Technology Stack

### App (uti)

- **Toolkit**: gtk4-rs 0.11 (`gnome_50`) + libadwaita 0.9 (`v1_9`), no Relm4
- **Language**: Rust 2021
- **Styling**: `app/data/style.css`; colors follow the system (libadwaita light/dark + accent)
- **i18n**: `app/data/i18n/{en,ja}.json` via `include_str!` (settings window only)
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

See [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) for detailed structure.

**Key directories:**

- `app/` - GTK4 / libadwaita app (`uti` crate and binary)
- `app/src/state/` - Pure, unit-tested UI state logic
- `app/packaging/` - Desktop file, metainfo, icons
- `packaging/` - RPM spec (app, daemon, extension) and `make-srpm.sh`; `.copr/` - COPR hook
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

- `theme.color`, `theme.accentColor`: ignored (colors follow the system)
- `theme.size`: "minimal", "normal", or "wide" (default: "normal"); sets window size/density
- `clipboardHistoryLimit`: Must be > 0 (default: 50)
- `language`: "en" or "ja" (default: "en")

**Clipboard history**: `~/.local/share/uti/clipboard.json` (`$XDG_DATA_HOME`);
moved from `~/.config/uti/` on first start. All files are written atomically
with mode 0600 (`uti_core::fs::write_atomic`).

**Auto-migration**: On startup, `clipboard.json` max_items syncs with
`clipboardHistoryLimit`

**Implementation**:

- Rust: `AppConfig::load()` at startup

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

- Rust via rustup, pinned by `rust-toolchain.toml` (1.99.1 = Fedora 44's rust used by COPR/RPM builds)
- mise: other tools pinned in `mise.toml` + `mise.lock` (just, lefthook, biome, cargo-deny,
  rumdl, git-cliff, actionlint, shellcheck). Update with `mise outdated` / `mise lock`.
- `gtk4-devel`, `libadwaita-devel` (Fedora 44; Ubuntu lacks libadwaita 1.9)

Setup: `mise install && just setup` (installs lefthook hooks).

### Tasks (justfile)

```bash
just fmt            # cargo fmt + biome --write + rumdl --fix
just lint           # lint-rust, lint-md, lint-ext, lint-gha, lint-sh
just ci             # lint + deny + test (same checks as CI)
just run            # Run app in a config sandbox (app/run-dev.sh)
just rpm            # SRPM + RPMs into dist/ (once: sudo dnf builddep packaging/uti.spec)
```

## Testing & Quality

### Test Framework

- **Rust**: Standard `cargo test` framework
- **App**: UI logic is tested as pure state transitions in `app/src/state/`
- **Methodology**: Test-Driven Development (TDD) required for all new code

### TDD Workflow (Red-Green-Refactor)

1. **Red**: Write failing test first
2. **Green**: Implement minimal code to pass
3. **Refactor**: Improve code while keeping tests green

### Test Execution

```bash
cargo test --workspace          # All Rust tests
cargo test -p uti               # App only
cd daemon && cargo test         # Daemon only
cargo tarpaulin                 # Coverage (requires cargo-tarpaulin)
just test                       # All tests
```

### CI Integration

- `ci.yml` jobs: Select jobs (by changed files), Rust lint and test (`fedora:44`), Dependency
  audit (cargo-deny), Docs, workflow and shell lint, Extension lint (only on `gnome-extension/`
  changes), RPM build (packaging-related PRs and pushes to `main`); `Merge gate` is the only
  required check
- `codeql.yml` (Code scanning / CodeQL): pushes to `main` and weekly, not on PRs
- Rust jobs skip for docs-only PRs; weekly scheduled full run
- Actions pinned by commit SHA; tools installed from `mise.lock` via `jdx/mise-action`
- Dependabot (cargo, github-actions): monthly, 7-day cooldown, merged by hand

### Test Priorities

1. **Daemon**: Keyboard event detection (300ms double-tap logic)
2. **App**: Window/list/navigation state (`app/src/state/`)
3. **Integration**: D-Bus communication end-to-end

## Deployment

### Current

- COPR `noppomario/uti` (`fedora-44-x86_64`): `uti`, `uti-daemon`, `gnome-shell-extension-uti`
  from one spec (`packaging/uti.spec`); `release.yml` triggers the build via `COPR_WEBHOOK_URL`
- `install.sh` enables the COPR repo, installs the packages, adds the input group and enables
  the daemon user service
- No updater: `sudo dnf upgrade`; GitHub Releases carry release notes only

### Future

- .deb package for Ubuntu/Debian (on request)
- AUR package for Arch Linux
- Flatpak

## Panel Menu

The "uti for GNOME" extension shows a panel button (left click toggles) with
Show/Hide, Settings, Check for Updates (opens GitHub Releases), GitHub and Quit.
The app has no tray icon.

## Known Limitations

1. Fixed 300ms double-tap interval
2. Linux-only, GNOME 50 for the extension
3. Window position/size not persisted
4. Positioning, always-on-top, hiding from dash and the panel menu require
   "uti for GNOME" (without it, pin only disables auto-hide)

## Roadmap

See [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) for platform support plans.
