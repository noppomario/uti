# Architectural Decision Records (ADR)

This document records active architectural and technical decisions that are
frequently referenced during development.

## Archived Decisions

Archived ADRs (001-015, 017, 019-022, 024-026) are stored in
`.claude/archive/adr-archive.md`. These decisions are finalized and
rarely need reference during development.

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

## ADR-027: Settings UI with declarative schema and i18n

**Date**: 2026-01-11
**Status**: Accepted
**Decision Makers**: Project team

### Context

Users needed a way to configure application settings (theme, clipboard limit,
language) without editing JSON files directly. The UI should be extensible
for future settings and support multiple languages.

### Decision

Implement a settings window with:

1. **Declarative schema** (`settings/schema.ts`) for automatic UI generation
2. **react-i18next** for settings-only internationalization (en/ja)
3. **Separate window** opened from tray menu
4. **Rust module refactoring** (config/, tray/, settings/)

### Schema Design

```typescript
{
  id: 'appearance',
  titleKey: 'sections.appearance',
  fields: [
    {
      key: 'themeColor',
      labelKey: 'fields.themeColor.label',
      type: 'select',
      configPath: 'theme.color',
      options: [...]
    }
  ]
}
```

Adding a new setting requires only:

1. Add field to schema
2. Add translation keys
3. Update Rust config struct

### Rationale

**Declarative over imperative**:

- Single source of truth for settings structure
- Automatic form generation reduces boilerplate
- Easy to add/remove settings without touching components

**Settings-only i18n**:

- Main UI is simple (tab names, status), doesn't need i18n
- Settings has more complex labels and descriptions
- Keeps main bundle size small

**Separate window**:

- Settings context differs from main window
- Can have different window size/style
- Independent lifecycle (persist while main hides)

**Rust module refactoring**:

- main.rs reduced from 1120 to ~480 lines
- Clear separation: config/, tray/, settings/
- Easier to test individual modules

### Alternatives Considered

1. **Hardcoded settings form**
   - Pro: Simpler initial implementation
   - Con: Tedious to add new settings

2. **JSON schema with react-hook-form**
   - Pro: More flexible validation
   - Con: Overkill for simple settings

3. **Full app i18n (main + settings)**
   - Pro: Consistent localization
   - Con: Unnecessary for minimal main UI

### Consequences

**Positive**:

- Easy to add new settings
- Bilingual settings (en/ja)
- Cleaner Rust code organization
- Schema-driven validation

**Negative**:

- Schema adds abstraction layer
- i18n adds bundle size to settings only
- Separate window is different from main UI flow

**Reconsider when**:

- Main UI needs i18n support
- Settings become complex enough for tabs/navigation
- Need for settings sync across devices

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
