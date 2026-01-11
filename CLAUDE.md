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
bun run frontend:all    # Frontend: format → lint → typecheck → build → test
bun run daemon:all      # Daemon: format → lint → build → test
bun run ci:local        # CI equivalent: lint → typecheck → build → test
bun run all:test        # Run all tests (parallel)

# Individual tasks (pattern: [workspace]:[task])
bun run frontend:lint   # Lint frontend only
bun run daemon:build    # Build daemon only
bun run rust:format     # Format all Rust code

# Test-Driven Development (TDD)
cd app && bun run test:watch    # Frontend watch mode (recommended for TDD)
cd app && bun run test:coverage # Coverage report
cargo test --workspace          # All Rust tests

# Traditional development
cd app && bun run tauri:dev    # Tauri dev server
cd daemon && sudo cargo run    # Run daemon
```

### Key Technologies

- **Frontend**: React 19 + TypeScript + Tailwind v4 + Biome
- **Testing**: Vitest + React Testing Library (TDD required)
- **Backend**: Tauri 2 + Rust
- **Daemon**: Rust + evdev + D-Bus
- **Target**: Linux (Fedora 43 GNOME/Wayland)

### Important Reminders

- ✅ **Terminal output**: ASCII only (Unicode has ambiguous width issues)
- ✅ All code and documentation in **English only**
- ✅ **TDD required**: Write tests BEFORE implementation (Red-Green-Refactor)
- ✅ Use **TSDoc** for TypeScript, **Rust doc comments** for Rust
- ✅ Format on save is enabled
- ✅ No personal information in commits
- ✅ Work on `claude/*` branches, create PRs to `develop` (not `main`)
- ✅ `main` is for stable releases only, `develop` is for latest features
- ✅ Commit messages in English, follow conventional commits
- ✅ **Run `bun run ci:local` before committing** (includes tests)

### Project Scale

- **Current**: 49 TypeScript files, ~6,600 lines
- **Philosophy**: Keep it simple (YAGNI)
- **Structure**: `components/`, `hooks/`, `settings/` directories

---

**Last updated:** 2026-01-11
