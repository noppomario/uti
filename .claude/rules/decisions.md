# Architectural Decision Records (ADR)

This document records active architectural and technical decisions that are
frequently referenced during development.

## Archived Decisions

Archived ADRs (001-017, 019-027) are stored in
`.claude/archive/adr-archive.md`. These decisions are finalized and
rarely need reference during development.

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

## ADR-028: Scalable project memory with ADR archiving

**Date**: 2026-01-11
**Status**: Accepted
**Decision Makers**: Project team

### Context

`.claude/rules/` files are automatically loaded at the start of every Claude Code
session. With 27 ADRs (1,724 lines in `decisions.md`), context window usage was
becoming a concern. Most ADRs document completed decisions that rarely need
reference during development.

### Decision

Archive finalized ADRs to `.claude/archive/adr-archive.md`:

1. **Archive location**: `.claude/archive/` (outside `.claude/rules/`, not auto-loaded)
2. **Active ADRs**: Keep only frequently-referenced decisions in `decisions.md`
3. **Criteria for active**: GNOME extension, daemon lifecycle, workflow, recent changes
4. **Reference pattern**: Link to archive from `decisions.md` header

### Active ADRs (4)

| ADR | Topic | Reason |
| --- | ----- | ------ |
| 016 | GNOME extension | Extension development reference |
| 018 | Daemon lifecycle | systemd configuration reference |
| 023 | Window pinning | GNOME extension feature |
| 027 | Settings UI | Latest feature |

### Archived ADRs (23)

001-015, 017, 019-022, 024-026 - All finalized decisions with stable implementations.

### Rationale

**Context efficiency**:

- Reduces auto-loaded content by ~1,300 lines (54%)
- Preserves decision history without consuming session context
- Archive accessible when needed via explicit read

**Clear separation**:

- Active: Decisions that inform current development
- Archived: Historical record of completed decisions

### Alternatives Considered

1. **Delete old ADRs**
   - Pro: Maximum reduction
   - Con: Loses valuable decision history

2. **GitHub Wiki**
   - Pro: Completely external
   - Con: Harder to reference, version control separate

3. **Summarize ADRs**
   - Pro: Keeps essence, reduces size
   - Con: Loses detailed rationale

### Consequences

**Positive**:

- 54% reduction in auto-loaded rules content
- Decision history preserved and accessible
- Clear guideline for future ADR archiving

**Negative**:

- Must manually read archive when referencing old decisions
- Two files to maintain instead of one

**Reconsider when**:

- Archive grows beyond 50 ADRs (consider further organization)
- Claude Code supports selective rule loading

---

## ADR-029: Trunk-based development on main

**Date**: 2026-11-14
**Status**: Accepted (supersedes ADR-021, ADR-022)
**Decision Makers**: Project team

### Context

The `develop` / `main` split (ADR-021) added sync PRs after every release and
drifted when a change landed on `main` directly (#117). With a single
maintainer it gave no benefit: stable versions are already identified by tags.

### Decision

- `main` is the only long-lived branch. Work on short-lived `claude/*`
  branches and open PRs to `main`.
- The release skill opens `release/vX.Y.Z` from `main`; merging it makes
  `release.yml` tag the release (amended 2026-11-15, no PAT).
- Remove `develop`, `sync-develop.yml` and Dependabot `target-branch`.

### Consequences

**Positive**: no sync PRs, Dependabot config applies immediately, one diff base.

**Negative**: `main` may contain unreleased work; users install tagged
releases (COPR, ADR-032), not the branch.

---

## ADR-030: Native GTK4 UI with libadwaita

**Date**: 2026-11-14
**Status**: Accepted (supersedes ADR-027; shipped in v0.3.0)
**Decision Makers**: Project team

### Context

uti targets GNOME/Wayland only. Tauri + React added a webview, a JS toolchain
and IPC surface (arbitrary command execution, `csp: null`), while the hard
platform problems (positioning, always-on-top, clipboard monitoring, double
modifier tap) are solved outside the UI framework anyway.

### Decision

gtk4-rs (`gnome_50`) + libadwaita (`v1_9`), no Relm4, for the popup and the
settings window. UI logic lives in pure `app/src/state/` modules; widgets only
apply state. Colors follow the system; `theme.size` remains as density.

### Alternatives Considered

1. GPUI: no window hide/activate on Linux, unofficial crate snapshots.
2. Slint: cross-platform with tray, but non-native look and licensing review.
3. Relm4: adds a 0.x dependency and tokio; the `state/` split already gives
   testable updates.
4. Keep Tauri: cross-platform not needed; webview attack surface.

### Consequences

**Positive**: one language for app and daemon, native look (dark mode, accent,
IME, HiDPI), no JS toolchain.

**Negative**: GNOME-only; custom color themes dropped; widgets are not
unit-tested.

**Reconsider when**: Windows/macOS/KDE support becomes a concrete goal
(re-evaluate Slint).

---

## ADR-031: App owns state; extension and daemon are D-Bus services

**Date**: 2026-11-14
**Status**: Accepted (supersedes ADR-016, ADR-023)
**Decision Makers**: Project team

### Context

The app, daemon and extension reacted to the same broadcast signals, which
raced (positioning before the window existed, sleeps as coordination). The
extension also hosted a StatusNotifierWatcher for Tauri's tray icon.

### Decision

- The app is the single owner of state and calls the others.
- Daemon (`io.github.noppomario.uti.Daemon`, `Daemon1`): only signal
  `Triggered` on double Ctrl.
- Extension (`io.github.noppomario.uti.Shell`, `Shell1`): methods
  `PositionAtPointer`, `SetAbove`, `Paste`, signal `ClipboardChanged`
  (`Meta.Selection` watch). Its own panel menu calls the app's GActions.
- App: `GApplication` owns `io.github.noppomario.uti`; actions `toggle`,
  `show`, `hide`, `settings`, `check-updates`, `quit`.
- Interfaces are versioned (`...1`); `Shell1` methods check that the caller
  owns the app name.

### Consequences

**Positive**: no signal races or sleeps; no SNI on GNOME; clipboard capture
without polling.

**Negative**: positioning, always-on-top and the panel menu require the
extension (pin falls back to "no auto-hide"); extension follows GNOME Shell
versions.

**Reconsider when**: Wayland gets standard positioning / always-on-top /
data-control protocols in Mutter.

---

## ADR-032: Distribute via COPR from a single spec

**Date**: 2026-11-15
**Status**: Accepted (first release: v0.4.0)
**Decision Makers**: Project team

### Context

RPMs attached to GitHub Releases were installed by `install.sh` and an in-app
updater (downloads to `/tmp`, `pkexec`, no signature check). The extension was
unzipped into the user's home.

### Decision

- COPR `noppomario/uti` (SCM, `make_srpm`). `packaging/uti.spec` builds
  `uti`, `uti-daemon` and `gnome-shell-extension-uti` (system-wide) from a
  vendored tarball, offline and `--locked`, with Fedora's rust.
- `release.yml` triggers COPR via the `COPR_WEBHOOK_URL` webhook;
  `.copr/Makefile` builds the latest `vX.Y.Z` tag, never the branch tip.
  A tag ruleset blocks moving or deleting `v*` tags.
- GitHub Releases carry notes only. Updates: `dnf upgrade`. No updater.
- deb/Ubuntu only on request (Ubuntu lacks libadwaita 1.9).

### Alternatives Considered

1. Keep Release assets: unsigned, needs an updater.
2. `copr-cli` in CI: API token expires every 180 days.
3. Fedora official packages: package review and crate packaging overhead.

### Consequences

**Positive**: signed repo, standard updates, CI `RPM build` job checks the
spec on every packaging change.

**Negative**: Fedora-only; a leaked webhook can only rebuild the latest tag.

---

## ADR-033: Tooling pinned in the repository (mise, just, lefthook)

**Date**: 2026-11-15
**Status**: Accepted
**Decision Makers**: Project team

### Context

After the rewrite only Rust remained, yet bun/npm still ran tasks, husky and
lint-staged. Lint tools were unpinned, so local results could differ from CI.

### Decision

- Rust via rustup, pinned by `rust-toolchain.toml` (Fedora 44's version).
- Other tools pinned in `mise.toml` + `mise.lock` (aqua backend, checksums);
  no node.
- `just` recipes are the only entry points; CI runs the same recipes, and
  `just ci` equals CI. Recipes fail when a tool is missing.
- lefthook: pre-commit formats staged files, commit-msg checks Conventional
  Commits, pre-push runs `just lint-rust test`.
- CI: SHA-pinned actions, least-privilege tokens, no PAT, jobs skipped by
  changed files, one required check (`Merge gate`). Dependabot monthly with a
  7-day cooldown, merged by hand. CodeQL off PRs.

### Consequences

**Positive**: reproducible local checks; no JS toolchain; faster PR CI.

**Negative**: contributors need mise and rustup; mise tools are updated by
hand (`mise outdated`).

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
