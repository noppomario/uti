# Architecture Decision Records

Why things are the way they are. Not auto-loaded; current facts live in
[ARCHITECTURE](../ARCHITECTURE.md) and [DEVELOPMENT](../DEVELOPMENT.md).
ADR-001 to ADR-029 mostly describe the Tauri-era app; ADR-030 and later describe the current one.

| ADR | Title | Date | Status |
| --- | --- | --- | --- |
| [001](records.md#adr-001) | Use Biome instead of ESLint + Prettier | 2025-12-27 | Accepted |
| [002](records.md#adr-002) | Do NOT use oxlint/oxfmt/tsgo | 2025-12-27 | Accepted |
| [003](records.md#adr-003) | No complex folder structure | 2025-12-27 | Accepted |
| [004](records.md#adr-004) | D-Bus for daemon-to-app communication | 2025-12-27 | Accepted |
| [005](records.md#adr-005) | Remove all personal information from commits | 2025-12-27 | Accepted |
| [006](records.md#adr-006) | English-only for all code and documentation | 2025-12-27 | Accepted |
| [007](records.md#adr-007) | Target Linux only (for now) | 2025-12-27 | Accepted |
| [008](records.md#adr-008) | Monorepo task architecture with workspace-first design | 2025-12-27 | Accepted |
| [009](records.md#adr-009) | Pre-commit quality gate with ci:local requirement | 2025-12-27 | Accepted |
| [010](records.md#adr-010) | Adopt Vitest for testing with TDD methodology | 2025-12-27 | Accepted |
| [011](records.md#adr-011) | Externalize configuration to JSON file | 2025-12-30 | Accepted |
| [012](records.md#adr-012) | Centralize theme management with Tailwind CSS v4 @theme | 2025-12-30 | Accepted (Updated 2026-01-04) |
| [013](records.md#adr-013) | Class-based dark mode only (no system theme detection) | 2025-12-30 | Accepted |
| [014](records.md#adr-014) | System tray implementation with AppIndicator | 2025-12-30 | Accepted |
| [015](records.md#adr-015) | Separate RPM packaging from development workflow | 2025-12-31 | Accepted |
| [016](records.md#adr-016) | Custom GNOME extension for tray and positioning | 2026-01-01 | Superseded by ADR-031 (2026-11) |
| [017](records.md#adr-017) | Unified component naming (uti-daemon) | 2026-01-01 | Accepted |
| [018](records.md#adr-018) | Daemon lifecycle bound to graphical session | 2026-01-01 | Accepted |
| [019](records.md#adr-019) | Launcher feature with system history integration | 2026-01-02 | Accepted |
| [020](records.md#adr-020) | markdownlint configuration rationale | 2026-01-03 | Accepted |
| [021](records.md#adr-021) | Adopt develop branch workflow (Git-flow simplified) | 2026-01-03 | Superseded by ADR-029 |
| [022](records.md#adr-022) | PR-based sync-develop workflow | 2026-01-03 | Superseded by ADR-029 |
| [023](records.md#adr-023) | Window pinning with GNOME extension for always-on-top | 2026-01-04 | Superseded by ADR-031 (2026-11) |
| [024](records.md#adr-024) | Snippets feature with delayed pin mechanism | 2026-01-05 | Accepted |
| [025](records.md#adr-025) | ENODEV handling for sleep/resume recovery | 2026-01-06 | Updated (2026-01-11) |
| [026](records.md#adr-026) | Prompt feature with uinput auto-paste | 2026-01-10 | Accepted |
| [027](records.md#adr-027) | Settings UI with declarative schema and i18n | 2026-01-11 | Superseded by ADR-030 (2026-11) |
| [028](records.md#adr-028) | Scalable project memory with ADR archiving | 2026-01-11 | Superseded (2026-11-15): all ADRs live in `docs/adr/` and are not auto-loaded |
| [029](records.md#adr-029) | D-Bus connection retry for Tauri app | 2026-01-12 | Accepted |
| [030](records.md#adr-030) | Native GTK4 UI with libadwaita | 2026-11-14 | Accepted (supersedes ADR-027; shipped in v0.3.0) |
| [031](records.md#adr-031) | App owns state; extension and daemon are D-Bus services | 2026-11-14 | Accepted (supersedes ADR-016, ADR-023) |
| [032](records.md#adr-032) | Distribute via COPR from a single spec | 2026-11-15 | Accepted (first release: v0.4.0) |
| [033](records.md#adr-033) | Tooling pinned in the repository (mise, just, lefthook) | 2026-11-15 | Accepted |
| [034](records.md#adr-034) | Trunk-based development on main | 2026-11-14 | Accepted (supersedes ADR-021, ADR-022) |
