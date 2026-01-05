# Architectural Decision Records (ADR)

This document records important architectural and technical decisions made for the project.

## ADR-001: Use Biome instead of ESLint + Prettier

**Date**: 2025-12-27
**Status**: Accepted
**Decision Makers**: Project team

### Context

Need a linter and formatter for TypeScript/React code. Traditional choice is ESLint + Prettier.

### Decision

Use Biome (Rust-based, single tool) instead of ESLint + Prettier.

### Rationale

- **Performance**: 25-100x faster than ESLint + Prettier
- **Simplicity**: Single tool instead of two
- **Modern**: Built for modern JavaScript/TypeScript
- **Project size**: Small project, simpler tooling is better

### Consequences

**Positive**:

- Much faster linting and formatting
- Simpler configuration (one config file)
- Better VSCode integration

**Negative**:

- Fewer plugins available vs ESLint ecosystem
- Smaller community (but growing rapidly)

---

## ADR-002: Do NOT use oxlint/oxfmt/tsgo

**Date**: 2025-12-27
**Status**: Accepted
**Decision Makers**: Project team

### Context

Zenn article suggested oxlint, oxfmt, and tsgo for maximum performance.

### Decision

Stay with current tooling (Biome + tsc + esbuild).

### Rationale

- **Project scale**: Only 77 lines of TypeScript
- **Stability**: tsgo is still in preview, unstable
- **No bottleneck**: Current tools are already fast enough
- **Simplicity**: Fewer tools = easier maintenance
- **YAGNI**: Don't optimize what's not a problem

### Consequences

**Positive**:

- Stable, proven tools
- Simpler setup
- Lower learning curve

**Negative**:

- Slightly slower than bleeding-edge tools (but not noticeable at this scale)

**Reconsider when**:

- Project grows to 50+ files
- Type checking takes >30 seconds
- CI time becomes a bottleneck
- tsgo reaches stable release

---

## ADR-003: No complex folder structure

**Date**: 2025-12-27
**Status**: Accepted
**Decision Makers**: Project team

### Context

Typical React projects use feature-based or layered folder structures.

### Decision

Keep flat structure in `src/` directory.

### Current Structure

```text
src/
├── App.tsx
├── main.tsx
└── index.css
```

### Rationale

- **YAGNI**: Only 3 files, no need for structure
- **Clarity**: Everything is immediately visible
- **Refactorable**: Easy to reorganize when needed

### Consequences

**Positive**:

- Zero cognitive overhead
- No time wasted on "where does this go?"
- Easy to understand for new contributors

**Negative**:

- Will need refactoring if project grows

**Reconsider when**:

- 5+ React components
- Multiple custom hooks
- Shared utilities needed
- Multiple features/pages

---

## ADR-004: D-Bus for daemon-to-app communication

**Date**: 2025-12-27
**Status**: Accepted
**Decision Makers**: Project team

### Context

Need IPC between root daemon (evdev access) and user-level Tauri app.

### Decision

Use D-Bus session bus.

### Rationale

- **Linux standard**: Well-supported on GNOME/Wayland
- **Security**: Proper permission separation
- **Reliability**: Proven technology
- **zbus library**: Excellent Rust support

### Alternatives Considered

1. **Unix socket**: More complex permission handling
2. **TCP localhost**: Overkill for local IPC
3. **Shared memory**: Too complex for simple signals

### Consequences

**Positive**:

- Proper security boundaries
- Standard Linux approach
- Easy to debug with `dbus-monitor`

**Negative**:

- D-Bus not available on Windows
- Adds dependency on system D-Bus daemon

---

## ADR-005: Remove all personal information from commits

**Date**: 2025-12-27
**Status**: Accepted
**Decision Makers**: Project team

### Context

Repository initially contained personal names in code, commits, and documentation.

### Decision

Rewrite git history and remove all personal identifiable information.

### Changes Made

- Removed author names from code
- Changed "うちお (@uchiwo)" to "Anonymous"
- Updated all D-Bus interfaces to use `io.github.noppomario.uti`
- Rewrote commit history

### Rationale

- Privacy concerns
- Professional presentation
- Easier to transfer ownership if needed

### Consequences

**Positive**:

- Clean, professional codebase
- Privacy protected

**Negative**:

- Lost original commit history
- Force-pushed to remote

---

## ADR-006: English-only for all code and documentation

**Date**: 2025-12-27
**Status**: Accepted
**Decision Makers**: Project team

### Context

Original code contained Japanese comments and documentation.

### Decision

Translate everything to English.

### Rationale

- **International collaboration**: English is lingua franca of programming
- **Tool compatibility**: Better IDE/AI support
- **Best practice**: Standard in open source

### Consequences

**Positive**:

- Accessible to international contributors
- Better tooling support
- Professional standard

**Negative**:

- None (Japanese UI text can be added later for localization)

---

## ADR-007: Target Linux only (for now)

**Date**: 2025-12-27
**Status**: Accepted
**Decision Makers**: Project team

### Context

Tauri supports Windows, macOS, and Linux. evdev is Linux-only.

### Decision

Focus on Linux (Fedora 43 GNOME/Wayland) for initial release.

### Rationale

- **evdev**: Linux-specific API
- **D-Bus**: More common on Linux
- **Focus**: Better to do one platform well
- **Future**: Can add Windows support later with different backend

### Windows Support Plan (Future)

- Replace evdev with Windows low-level keyboard hooks
- Replace D-Bus with TCP localhost or named pipes
- Use conditional compilation (`#[cfg(target_os)]`)

### Consequences

**Positive**:

- Simpler initial implementation
- Can optimize for Linux

**Negative**:

- Limited audience
- Future refactoring needed for Windows

---

## ADR-008: Monorepo task architecture with workspace-first design

**Date**: 2025-12-27
**Status**: Accepted
**Decision Makers**: Project team

### Context

Project has three workspaces (frontend, tauri, daemon). Need standardized task
execution across workspaces with CI/local parity.

### Decision

Use **workspace-first naming** pattern `[workspace]:[task]`:

- Workspaces: `frontend`, `tauri`, `daemon`, `app`, `rust`, `all`
- Tasks: `format`, `lint`, `typecheck`, `build`, `test`, `all`

Root `package.json` as task runner with `npm-run-all` for parallel execution.
Rust workspace (`Cargo.toml`) for unified Rust management.

### Rationale

- Workspace-centric thinking: `bun run daemon:all`
- Selective execution: Only run tasks for modified workspace
- CI/local parity: Same `ci:local` command everywhere
- Parallel execution: Independent workspaces run concurrently

### Consequences

**Positive**:

- Clear task naming
- Fast selective execution
- CI and local perfectly aligned

**Negative**:

- Requires bun/npm installed
- Learning curve for naming convention

---

## ADR-009: Pre-commit quality gate with ci:local requirement

**Date**: 2025-12-27
**Status**: Accepted
**Decision Makers**: Project team

### Context

Need to prevent low-quality commits while maintaining fast workflow.

### Decision

Two-layer quality gate:

1. **Automatic**: Pre-commit hook formats and lints staged files
2. **Manual**: Developers run `bun run ci:local` before significant commits

### Rationale

- Automatic formatting prevents unformatted code
- Full `ci:local` check too slow for every commit
- Trust developers for comprehensive checks
- CI still catches everything if forgotten

### Consequences

**Positive**:

- Fast commits for small changes
- Automatic formatting
- Reduced CI failures

**Negative**:

- Developers might forget `ci:local`
- Requires discipline

**Reconsider when**:

- CI failure rate exceeds 20%

---

## ADR-010: Adopt Vitest for testing with TDD methodology

**Date**: 2025-12-27
**Status**: Accepted
**Decision Makers**: Project team

### Context

Project had no automated tests. Need to establish testing infrastructure with
TDD methodology for future development.

Two main options considered:

1. **Bun native test runner** - Zero additional dependencies, ultra-fast
2. **Vitest** - Mature ecosystem, Vite integration

### Decision

Use **Vitest** for frontend testing, standard `cargo test` for Rust.
Require **Test-Driven Development (TDD)** for all new code.

### Rationale

#### Why Vitest over Bun Test (2025 analysis)

**Test Isolation (Critical for TDD)**:

- Vitest: Complete isolation between test suites (default)
- Bun Test: Shared global state, mocks leak between tests

**Ecosystem Maturity**:

- Vitest: Full IDE integration, browser mode, sharding, type testing
- Bun Test: Limited IDE support, experimental coverage

**Vite Ecosystem Consistency**:

- Already using Vite (Tauri recommended)
- Vitest shares `vite.config.ts` configuration
- Same plugin ecosystem

**Production Readiness**:

- Vitest: Battle-tested, recommended by Evan You (Vite creator)
- Bun Test: Production-ready for internal tools, but isolated testing issues remain

#### Why NOT Bun Test

- Global state leakage breaks TDD reliability
- No IDE integration for watch mode debugging
- Coverage reporting still basic (CLI-only output)

#### Oxc Ecosystem Consideration

- oxfmt/oxlint (Dec 2025 alpha) are faster than Biome (2-3x)
- Decision: Keep Biome for now due to maturity and CST-based editor support
- Reconsider when oxfmt reaches stable release

### Alternatives Considered

1. **Bun native test runner**
   - Pro: 5x faster than Vitest, zero dependencies
   - Con: Test isolation issues, limited tooling

2. **Jest**
   - Pro: Most popular, huge ecosystem
   - Con: 8-20x slower than alternatives, outdated architecture

### Consequences

**Positive**:

- Reliable TDD workflow with isolated tests
- Excellent watch mode for red-green-refactor cycle
- Coverage reports integrated with CI
- Consistent with Vite build tooling
- Strong TypeScript/React support

**Negative**:

- 3 additional dependencies vs Bun native
- Slightly slower than Bun (though fast enough in practice)

**Migration Path**:

- If Bun test fixes isolation issues in future, migration is easy (Jest-compatible API)

### Implementation Details

**Dependencies**:

```bash
bun add -d vitest @testing-library/react @testing-library/dom happy-dom
```

**Coverage Targets**:

- Minimum: 70% line coverage
- Target: 80%+ overall
- Critical paths (daemon keyboard logic): 90%+

**TDD Enforcement**:

- Documented in conventions.md
- Required for all new features/bug fixes
- CI runs tests on every commit
- Watch mode recommended for development

### Reconsider When

- Bun test implements proper test isolation
- Project scales beyond 10,000+ tests (speed becomes critical)
- Vitest maintenance stalls

---

---

## ADR-011: Externalize configuration to JSON file

**Date**: 2025-12-30
**Status**: Accepted
**Decision Makers**: Project team

### Context

Application settings (theme, clipboard limit) were hardcoded in source files.
Users need to customize settings without recompiling.

### Decision

Use `~/.config/uti/config.json` for persistent user configuration with:

- `theme.color`: "midnight", "dark", or "light"
- `theme.size`: "minimal", "normal", or "wide"
- `clipboardHistoryLimit`: Number of items to store

### Rationale

**Separation of concerns**:

- Code defines behavior
- Config defines preferences
- Clear boundary between application logic and user settings

**Linux standards**:

- Follow XDG Base Directory spec (`~/.config/`)
- JSON format for human readability
- Auto-created with defaults on first run

**Auto-migration**:

- `clipboard.json` max_items syncs with `clipboardHistoryLimit` on startup
- No manual intervention required for existing users

### Alternatives Considered

1. **Environment variables**
   - Pro: Simple
   - Con: Not persistent, hard to edit

2. **TOML/YAML format**
   - Pro: More human-friendly
   - Con: Requires additional parser, JSON is standard

3. **Single combined file**
   - Pro: One file for everything
   - Con: Mixing config and data (clipboard history)

### Consequences

**Positive**:

- Users can customize settings without code changes
- Clear separation: `config.json` (settings) vs `clipboard.json` (data)
- Auto-migration ensures smooth upgrades

**Negative**:

- Two JSON files instead of one
- Potential for config/data desync (mitigated by auto-migration)

**Reconsider when**:

- Need for more complex config (nested settings, profiles)
- Performance issues with JSON parsing (unlikely at this scale)

---

## ADR-012: Centralize theme management with Tailwind CSS v4 @theme

**Date**: 2025-12-30
**Status**: Accepted (Updated 2026-01-04)
**Decision Makers**: Project team

### Context

Color definitions were scattered across 6 locations (components, CSS files).
Tailwind CSS v4 introduced CSS-first configuration with `@theme` directive.

### Decision

Use Tailwind v4 `@theme` directive with CSS variables in `index.css`:

**Color themes** (3 options):

| Name     | Description        | Base Color          |
| -------- | ------------------ | ------------------- |
| Midnight | Slate-based dark   | slate-900 (#0f172a) |
| Dark     | Adwaita-inspired   | neutral (#1c1c1c)   |
| Light    | Default light      | white (#ffffff)     |

**Size themes** (3 options):

| Name    | Window | Font  | Padding |
| ------- | ------ | ----- | ------- |
| Minimal | 250px  | 12px  | 4px     |
| Normal  | 500px  | 14px  | 8px     |
| Wide    | 750px  | 16px  | 12px    |

```css
@theme {
  --color-app-bg: #ffffff;
  --size-font-base: 0.75rem;
}

.theme-dark {
  --color-app-bg: #1c1c1c;
}

.size-normal {
  --size-font-base: 0.875rem;
}
```

Components use semantic tokens: `bg-app-bg`, `text-app-text`

### Rationale

**Centralization**:

- All colors in one file (`index.css`)
- Single source of truth for theme
- Easy to modify entire color scheme

**Tailwind v4 best practices**:

- CSS-first approach (vs v3's JavaScript config)
- Leverages native CSS custom properties
- Better build-time performance

**Semantic naming**:

- `bg-app-bg` vs `bg-slate-900` (implementation detail)
- Easier to understand component intent
- Can change underlying colors without touching components

**Separate color and size**:

- Users can mix any color with any size
- 9 combinations possible (3 colors × 3 sizes)

### Alternatives Considered

1. **Tailwind v3 style (tailwind.config.js)**
   - Pro: Familiar to v3 users
   - Con: Not the v4 recommended approach

2. **Inline CSS variables in components**
   - Pro: Co-located with usage
   - Con: Duplicated definitions, hard to maintain

3. **CSS modules**
   - Pro: Scoped styles
   - Con: More complex, doesn't leverage Tailwind utilities

### Consequences

**Positive**:

- Theme changes in one file
- Follows Tailwind v4 conventions
- Color/size themes via class toggle (`.theme-{color}`, `.size-{size}`)
- Flexible combinations for user preference

**Negative**:

- VSCode CSS language server doesn't recognize `@theme` (shows warnings)
- Requires Tailwind v4 (beta), not stable yet

**Reconsider when**:

- Tailwind v5 changes `@theme` directive significantly
- Need for component-specific color overrides

---

## ADR-013: Class-based dark mode only (no system theme detection)

**Date**: 2025-12-30
**Status**: Accepted
**Decision Makers**: Project team

### Context

Attempted to detect system theme via `prefers-color-scheme` media query.
WebKitGTK + Wayland doesn't properly report system color scheme.

### Decision

Use explicit class-based dark mode controlled by `config.json`:

- `@custom-variant dark (&:where(.dark, .dark *))`
- Theme set by `theme` config option
- No automatic system detection

### Rationale

**Technical limitation**:

- WebKitGTK on Wayland doesn't expose `prefers-color-scheme`
- Tested with standalone HTML - same issue
- X11 may work, but Wayland is primary target (Fedora 43 GNOME)

**User control**:

- Explicit config is predictable
- No "magic" behavior that might fail
- Users can set theme independently of system

**Implementation simplicity**:

- Class-based switching is reliable
- No need for media query listeners
- Works on all platforms

### Alternatives Considered

1. **Media query with fallback**
   - Pro: Respects system settings when available
   - Con: Unreliable on target platform

2. **Tauri API for system theme**
   - Pro: Platform-specific detection
   - Con: Tauri 2.x doesn't expose this API

3. **Manual theme toggle button**
   - Pro: User control
   - Con: Extra UI complexity for MVP

### Consequences

**Positive**:

- Reliable on all platforms
- Clear user control
- Simple implementation

**Negative**:

- No automatic sync with system theme
- Users must manually set preference

**Reconsider when**:

- WebKitGTK fixes `prefers-color-scheme` on Wayland
- Tauri adds system theme detection API
- Significant user demand for auto-detection

---

## ADR-014: System tray implementation with AppIndicator

**Date**: 2025-12-30
**Status**: Accepted
**Decision Makers**: Project team

### Context

Application needed a way for users to control window visibility beyond
double Ctrl press. System tray is standard for background applications.

### Decision

Implement system tray using Tauri 2 `tray-icon` feature with libappindicator
backend for Linux.

### Rationale

**User Experience**:

- Tray icon provides visual indicator that app is running
- Context menu offers familiar interaction pattern
- Left-click toggle is intuitive for quick access

**Technical**:

- Tauri 2 has built-in tray support (no additional dependencies)
- libappindicator is standard on GNOME/Linux
- Reuses existing app icon (no new assets needed)

**Menu Design**:

- Show/Hide: Primary action for window toggle
- Auto-start: Toggle to enable/disable launch on login
- GitHub: Opens repository in browser (external link indicator `↗`)
- Version: Displays app version (disabled, info only)
- Quit: Essential for closing app from tray

### Alternatives Considered

1. **No tray icon**
   - Pro: Simpler implementation
   - Con: Users can't easily tell if app is running

2. **D-Bus only control**
   - Pro: No GUI dependencies
   - Con: Poor user experience, no visual feedback

### Consequences

**Positive**:

- Users can control app from tray
- Standard desktop application behavior
- Works on KDE, XFCE out-of-the-box

**Negative**:

- GNOME 43+ requires AppIndicator extension installation
- Adds setup complexity for GNOME users

**Documentation Added**:

- README: Installation instructions for AppIndicator
- README: Troubleshooting for GTK warnings
- context.md: System tray section with requirements

**Reconsider when**:

- GNOME restores native tray support

---

## ADR-015: Separate RPM packaging from development workflow

**Date**: 2025-12-31
**Status**: Accepted
**Decision Makers**: Project team

### Context

Added `daemon:rpm` script for local RPM package building. Question arose whether
to include it in `daemon:all` for consistency with other `:all` scripts.

### Decision

Keep `daemon:rpm` separate from `daemon:all`. RPM packaging is a release task,
not a development task.

### Script Structure

```text
daemon:format  ─┐
daemon:lint    ─┼─→ daemon:all (development cycle)
daemon:build   ─┤
daemon:test    ─┘

daemon:rpm     ─────→ standalone (release packaging)
```

### Rationale

**Environment dependency**:

- `rpmbuild` is not universally available
- Including in `:all` would fail on macOS, Windows, or Linux without rpm-build
- Development workflow should work everywhere

**Separation of concerns**:

- `:build` = compile source to binary
- `:rpm` = package binary for distribution
- These are distinct lifecycle phases

**CI/CD alignment**:

- GitHub Actions release workflow calls `daemon:rpm` explicitly
- `ci:local` focuses on validation, not packaging
- Packaging happens only during release

**Consistency with Tauri**:

- `tauri:build` produces RPM as side effect (Tauri's design, not ours)
- Daemon follows explicit separation: build binary, then optionally package

### Alternatives Considered

1. **Include in daemon:all**
   - Pro: Consistent naming (`all` means everything)
   - Con: Breaks on systems without rpmbuild

2. **Add daemon:all:full including rpm**
   - Pro: Choice between quick and full
   - Con: Adds complexity, rarely needed

3. **Make rpmbuild optional in daemon:rpm**
   - Pro: Script never fails
   - Con: Silent failure is confusing

### Consequences

**Positive**:

- Development workflow works on any platform
- Clear separation: development vs release
- Explicit `bun run daemon:rpm` when packaging needed

**Negative**:

- `daemon:all` doesn't literally run "all" daemon tasks
- Developers must remember to test RPM before release

**Reconsider when**:

- Need for cross-platform packaging (AppImage, Flatpak)
- Automated nightly builds require packaging

---

## ADR-016: Custom GNOME extension for tray and positioning

**Date**: 2026-01-01
**Status**: Accepted
**Decision Makers**: Project team

### Context

On GNOME/Wayland, two features required privileged shell access:

1. **Tray icon**: GNOME 43+ removed native tray support
2. **Window positioning**: Wayland apps cannot query cursor position

### Decision

Create "uti for GNOME" extension that provides:

1. **StatusNotifierHost**: Displays Tauri's SNI tray icon in GNOME panel
2. **Cursor positioning**: Moves window to cursor on D-Bus signal

### Rationale

**Single dependency**:

- One extension handles both tray and positioning
- No need for third-party AppIndicator extension
- Full control over behavior

**Architecture fit**:

- Extension listens to same D-Bus signal as Tauri app
- D-Bus broadcast enables parallel handling
- Extension uses `Meta.Window.move_frame()` for positioning

### Alternatives Considered

1. **AppIndicator extension + libappindicator**
   - Pro: More widely used
   - Con: Doesn't solve positioning issue

2. **Separate extensions for tray and positioning**
   - Pro: Modular
   - Con: More complexity for users

### Consequences

**Positive**:

- Single installation for full GNOME support
- Custom tray menu matches app exactly
- Cursor positioning works on Wayland

**Negative**:

- Must maintain GNOME extension separately
- Extension API changes with GNOME versions

---

## ADR-017: Unified component naming (uti-daemon)

**Date**: 2026-01-01
**Status**: Accepted
**Decision Makers**: Project team

### Context

Keyboard daemon was named `double-ctrl` in early development, which didn't match the project branding.

### Decision

Rename daemon to `uti-daemon` for consistency with `uti` (Tauri app) and `uti for GNOME` (extension).

### Changes

| Before | After |
| ------ | ----- |
| `double-ctrl.service` | `uti-daemon.service` |
| `double-ctrl.spec` | `uti-daemon.spec` |
| Binary: `double-ctrl` | Binary: `uti-daemon` |

### Rationale

- Unified branding across all components
- Clear association: `uti-daemon` serves `uti` app
- Easier to discover related packages

### Consequences

**Positive**:

- Consistent naming
- Better discoverability

**Negative**:

- Breaking change for existing installations (requires reinstall)

---

## ADR-018: Daemon lifecycle bound to graphical session

**Date**: 2026-01-01
**Status**: Accepted
**Decision Makers**: Project team

### Context

After user re-login, double Ctrl press stopped working. The daemon was holding
a stale D-Bus session connection from the previous login session.

### Decision

Bind daemon lifecycle to `graphical-session.target`:

- `BindsTo=graphical-session.target`: Stop daemon when session ends
- `WantedBy=graphical-session.target`: Start daemon when session starts
- Handle D-Bus errors gracefully without exiting monitor loop

### Rationale

**Session-aware design**:

- D-Bus session bus is per-login session
- Daemon must restart with fresh connection after re-login
- `BindsTo` ensures clean shutdown on logout

**Graceful error handling**:

- D-Bus send errors during logout shouldn't crash the daemon
- Log errors but continue monitoring keyboards

### Consequences

**Positive**:

- Double Ctrl works reliably after re-login
- Clean daemon lifecycle management
- No stale connections

**Negative**:

- Requires `systemctl --user disable && enable` when changing `WantedBy`

---

## ADR-019: Launcher feature with system history integration

**Date**: 2026-01-02
**Status**: Accepted
**Decision Makers**: Project team

### Context

Users wanted quick access to frequently used applications beyond clipboard history.
Need to launch apps with their recent files (jump lists) like Windows taskbar.

### Decision

Add launcher tab with:

1. **JSON configuration** (`~/.config/uti/launcher.json`) for command definitions
2. **System history integration** via recently-used.xbel (freedesktop standard)
3. **VSCode support** via SQLite database parsing

### Rationale

**System API approach**:

- Uses existing freedesktop standard (no custom tracking needed)
- Works with any GTK app that writes to recently-used.xbel
- No file watching or daemon required

**JSON configuration**:

- Simple, human-readable format
- Consistent with existing config.json
- Flexible history source per command

**Tab-based UI**:

- Preserves existing clipboard functionality
- Clear separation between features
- Keyboard navigation (←/→ for tabs, ↑/↓ for items)

### Alternatives Considered

1. **Custom file tracking daemon**
   - Pro: Works with any app
   - Con: Complex, resource-intensive

2. **Desktop file parsing (.desktop)**
   - Pro: Standard app definitions
   - Con: Doesn't provide recent files

3. **Bookmark-based approach**
   - Pro: User curated
   - Con: No automatic recent files

### Consequences

**Positive**:

- Quick app launching with recent files
- Works with existing GTK ecosystem
- No additional daemons or services

**Negative**:

- Limited to apps that write to recently-used.xbel
- VSCode requires custom SQLite parsing
- History accuracy depends on app behavior

**Reconsider when**:

- Need to support non-GTK apps (KDE, Electron)
- recently-used.xbel format changes significantly

---

## ADR-020: markdownlint configuration rationale

**Date**: 2026-01-03
**Status**: Accepted
**Decision Makers**: Project team

### Context

Document the rationale for each rule in `.markdownlint.json`.

### Configuration and Rationale

| Rule  | Setting                   | Reason                                        |
| ----- | ------------------------- | --------------------------------------------- |
| MD013 | `false`                   | Line breaks reduce readability; editors wrap  |
| MD024 | `siblings_only: true`     | ADR template repeats same headings            |
| MD033 | `allowed_elements: [...]` | README needs HTML for styling                 |
| MD041 | `false`                   | README starts with div (badges)               |

---

## ADR-021: Adopt develop branch workflow (Git-flow simplified)

**Date**: 2026-01-03
**Status**: Accepted
**Decision Makers**: Project team

### Context

Features merged to main immediately updated README, but releases happened later.
This caused README to describe features not yet available in the latest release.

### Decision

Adopt a simplified Git-flow with:

- `main`: Stable releases only
- `develop`: Latest development
- `claude/*`: Feature branches (PR to develop)
- `release/vX.Y.Z`: Release preparation (PR to main)

### Rationale

**Separation of concerns**:

- README on main matches latest release
- Users downloading release get accurate documentation
- Developers can preview upcoming features on develop

**Minimal workflow change**:

- Only PR target changes (develop instead of main)
- Release workflow already handles version bumping
- Auto-sync after release keeps develop up-to-date

### Alternatives Considered

1. **Separate docs branch**
   - Pro: Minimal change
   - Con: Complex, docs can desync from code

2. **Version-specific READMEs**
   - Pro: Detailed per-version docs
   - Con: Maintenance burden

3. **Release notes only (no README changes)**
   - Pro: Simple
   - Con: README becomes stale

### Consequences

**Positive**:

- README always matches latest release
- Clear separation: develop = latest, main = stable
- Existing workflow mostly unchanged

**Negative**:

- One extra branch to maintain
- Must remember to PR to develop, not main
- Sync workflow adds complexity

**Reconsider when**:

- Team size grows (may need more branches)
- Release frequency changes significantly

---

## ADR-022: PR-based sync-develop workflow

**Date**: 2026-01-03
**Status**: Accepted
**Decision Makers**: Project team

### Context

sync-develop.yml failed during v0.0.10 release because develop branch has
protection rules requiring CI checks. Direct push was rejected.

### Decision

Change sync-develop.yml to create a PR instead of pushing directly:

1. Create sync branch from main
2. Create PR to develop with auto-merge enabled
3. PR goes through normal CI and merges automatically

### Rationale

**Branch protection compatibility**:

- PRs go through required CI checks
- Auto-merge handles the merge after CI passes
- No need to bypass protection rules

**Consistency**:

- All changes to develop go through PRs
- Audit trail for sync operations

### Consequences

**Positive**:

- Works with branch protection rules
- CI validates sync changes
- Clear PR history

**Negative**:

- Slightly more complex workflow
- Creates additional sync branches (cleaned up by --delete-branch)

---

## ADR-023: Window pinning with GNOME extension for always-on-top

**Date**: 2026-01-04
**Status**: Accepted
**Decision Makers**: Project team

### Context

Window pinning feature requested to keep window visible during frequent
clipboard access. On Wayland/GNOME, `set_always_on_top()` is ignored by Mutter.

### Decision

Two-phase implementation:

1. **Phase 1.5**: Pin state disables auto-hide and ignores double Ctrl toggle
2. **Phase 2**: D-Bus signal `SetAlwaysOnTop(bool)` to GNOME extension, which
   calls `Meta.Window.make_above()`

### Rationale

**Wayland security model**:

- Mutter ignores app-level always-on-top requests
- Only compositor (GNOME Shell) can control window stacking
- Extension is the only way to access `Meta.Window.make_above()`

**Leveraging existing infrastructure**:

- GNOME extension already exists for tray and positioning
- D-Bus signal pattern already established
- No new dependencies

### Alternatives Considered

1. **GTK always-on-top APIs**
   - Pro: No extension needed
   - Con: Doesn't work on Wayland/Mutter

2. **org.gnome.Shell.Eval D-Bus**
   - Pro: No extension changes
   - Con: Disabled for security (GNOME 3.38+)

3. **Pin = auto-hide disable only**
   - Pro: Simpler, works everywhere
   - Con: Window can be covered by other windows

### Consequences

**Positive**:

- Full pinning functionality on GNOME/Wayland
- Consistent with existing architecture
- Graceful degradation (auto-hide still works without extension)

**Negative**:

- GNOME-specific (KDE/Sway need separate implementation)
- Requires extension reload after update

**Reconsider when**:

- Wayland standardizes always-on-top protocol
- KDE/Sway support needed

---

## ADR-024: Snippets feature with delayed pin mechanism

**Date**: 2026-01-05
**Status**: Accepted
**Decision Makers**: Project team

### Context

Users requested ability to save frequently used clipboard items permanently.
The feature should integrate with existing tab-based UI and clipboard workflow.

### Decision

Add Snippets tab with delayed pin mechanism:

1. **Tab placement**: Clipboard → Snippets → Launcher
2. **Storage**: `~/.config/uti/snippets.json` with UUID-based IDs
3. **Pin mechanism**: Delayed move (mark in clipboard → move on window close)
4. **UI**: Same list/search pattern as Clipboard and Launcher

### Rationale

**Delayed pin vs immediate move**:

- Immediate move breaks user flow (item disappears mid-work)
- Delayed move allows accumulating multiple pins before processing
- Visual feedback (pin icon) confirms user intent
- Consistent with "work session" mental model

**Separate storage file**:

- Snippets are user-curated, persist across clipboard clears
- Different lifecycle than transient clipboard history
- Easier backup/migration of permanent items

**Tab ordering**:

- Clipboard is primary use case (default tab)
- Snippets are clipboard-related (placed next to clipboard)
- Launcher is distinct feature (rightmost)

### Alternatives Considered

1. **Immediate move on pin click**
   - Pro: Simpler implementation
   - Con: Disruptive UX, item disappears immediately

2. **Star/favorite within clipboard**
   - Pro: Single list, simpler UI
   - Con: Mixes transient and permanent items, confusing

3. **Context menu for pin**
   - Pro: Cleaner item UI
   - Con: Hidden functionality, extra click required

### Consequences

**Positive**:

- Non-disruptive pin workflow
- Clear separation of transient vs permanent items
- Consistent UI pattern across all tabs
- Keyboard navigation works uniformly

**Negative**:

- Delayed move may confuse users expecting immediate action
- Pin state is lost if app crashes before window close
- Index-based removal requires descending order processing

**Reconsider when**:

- Users request immediate move option
- Need for snippet categories/folders
- Sync across devices requested

---

## Template for Future ADRs

```markdown
## ADR-XXX: [Title]

**Date**: YYYY-MM-DD
**Status**: [Proposed | Accepted | Deprecated | Superseded]
**Decision Makers**: [Who decided]

### Context
[What is the situation and problem?]

### Decision
[What are we doing?]

### Rationale
[Why this decision?]

### Alternatives Considered
1. [Alternative 1]
2. [Alternative 2]

### Consequences
**Positive**:
- [Good outcome 1]

**Negative**:
- [Trade-off 1]

**Reconsider when**:
- [Condition 1]
```
