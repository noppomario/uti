# uti - Project Memory

This document serves as the central project memory for Claude Code.
All team members should read this when starting development.

**Note:** Detailed rules are automatically loaded from `.claude/rules/`:
- `context.md` - Technical stack, architecture, constraints
- `decisions.md` - Architectural Decision Records (ADRs)
- `conventions.md` - Code standards, naming, Git conventions

---

## Quick Reference

### Development Commands
```bash
# Frontend development
cd app
bun install
bun run tauri:dev

# Daemon development (requires root)
cd daemon
sudo cargo run

# Code quality
bun run check:fix      # Biome lint + format
bun run type-check     # TypeScript type checking
cargo fmt              # Rust formatting
cargo clippy           # Rust linting (when available)
```

### Key Technologies
- **Frontend**: React 19 + TypeScript + Tailwind v4 + Biome
- **Backend**: Tauri 2 + Rust
- **Daemon**: Rust + evdev + D-Bus
- **Target**: Linux (Fedora 43 GNOME/Wayland)

### Important Reminders
- ✅ All code and documentation in **English only**
- ✅ Use **TSDoc** for TypeScript, **Rust doc comments** for Rust
- ✅ Format on save is enabled
- ✅ No personal information in commits
- ✅ Work on `claude/*` branches
- ✅ Commit messages in English, follow conventional commits

### Project Scale
- **Current**: 3 TypeScript files, ~77 lines
- **Philosophy**: Keep it simple (YAGNI)
- **Refactor when**: 5+ components or multiple features

---

*Last updated: 2025-12-27*
