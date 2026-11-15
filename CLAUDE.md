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
# Task runner: just (tools pinned in mise.toml; run `mise install` and `just setup` once)
just ci                 # Same checks as CI: lint + deny + test
just fmt                # cargo fmt + biome + rumdl --fix
just lint               # lint-rust, lint-md, lint-ext, lint-gha
just test               # All tests
just deny               # cargo-deny (deny.toml)
just rpm                # SRPM + app/daemon/extension RPMs into dist/

# Test-Driven Development (TDD)
cargo test -p uti               # App tests (app/src/state/)
cargo test --workspace          # All Rust tests

# Development
just run                        # Run app in a config sandbox (app/run-dev.sh)
cd daemon && cargo run          # Run daemon (needs input group)
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
- ✅ lefthook: pre-commit formats staged files, commit-msg checks Conventional Commits, pre-push runs `just lint-rust test`
- ✅ No personal information in commits
- ✅ Trunk-based: work on short-lived `claude/*` branches, create PRs to `main`
- ✅ Releases are tags (`vX.Y.Z`) created from `main` by `release.yml`
- ✅ Commit messages in English, follow conventional commits
- ✅ **Run `just ci` before committing** (includes tests)

### Project Scale

- **Current**: 12 Rust files in `app/src/`
- **Philosophy**: Keep it simple (YAGNI)
- **Structure**: `app/src/` (UI, D-Bus, settings) and `app/src/state/` (pure, unit-tested logic)

---

**Last updated:** 2026-11-15
