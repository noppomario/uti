# uti - Project Memory

This document serves as the central project memory for Claude Code.
All team members should read this when starting development.

**Note:** Detailed rules are automatically loaded from `.claude/rules/`:

- `context.md` - Technical stack, architecture, constraints
- `decisions.md` - Active ADRs (archived ADRs in `.claude/archive/`)
- `conventions.md` - Code standards, naming, Git conventions

---

## Quick Reference

### Development Commands

```bash
# Monorepo task runner (from root)
bun run app:all         # App: lint → test
bun run daemon:all      # Daemon: format → lint → build → test
bun run ci:local        # CI equivalent: lint → test → build
bun run all:test        # Run all tests

# Individual tasks (pattern: [workspace]:[task])
bun run app:lint        # Lint app only
bun run daemon:build    # Build daemon only
bun run rust:format     # Format all Rust code

# Test-Driven Development (TDD)
cargo test -p uti               # App tests (app/src/state/)
cargo test --workspace          # All Rust tests

# Development
bun run app:run                 # Run app in a config sandbox (app/run-dev.sh)
cd daemon && sudo cargo run     # Run daemon
```

### Key Technologies

- **App**: Rust + gtk4-rs 0.11 + libadwaita 0.9
- **Testing**: `cargo test` (TDD required)
- **Daemon**: Rust + evdev + D-Bus
- **Target**: Linux (Fedora 44 GNOME 50/Wayland)

### Important Reminders

- ✅ **Terminal output**: ASCII only (Unicode has ambiguous width issues)
- ✅ All code and documentation in **English only**
- ✅ **TDD required**: Write tests BEFORE implementation (Red-Green-Refactor)
- ✅ Use **Rust doc comments** for Rust
- ✅ Format on save is enabled
- ✅ No personal information in commits
- ✅ Trunk-based: work on short-lived `claude/*` branches, create PRs to `main`
- ✅ Releases are tags (`vX.Y.Z`) created from `main` by `release.yml`
- ✅ Commit messages in English, follow conventional commits
- ✅ **Run `bun run ci:local` before committing** (includes tests)

### Project Scale

- **Current**: 12 Rust files in `app/src/`
- **Philosophy**: Keep it simple (YAGNI)
- **Structure**: `app/src/` (UI, D-Bus, settings) and `app/src/state/` (pure, unit-tested logic)

---

**Last updated:** 2026-11-15
