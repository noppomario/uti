# Plan: GNOME-native rewrite (Tauri -> GTK4 + libadwaita)

**Status**: Draft
**Date**: 2026-11-14
**Scope**: uti app, uti-daemon, uti for GNOME extension, packaging

Items marked **[UNVERIFIED]** are assumptions that must be confirmed by a
PoC or primary sources before relying on them.

---

## 1. Goals and non-goals

### Goals

- Replace Tauri + React with a native Rust GUI targeting **GNOME/Wayland only**
  (minimum GNOME 50 / libadwaita 1.9; Fedora 44 first).
- Redesign component responsibilities so that each platform constraint is solved
  by the component that can actually solve it (daemon / extension / app).
- Fix correctness and security issues found in the current code (section 5).
- Keep pure logic in a framework-independent crate so the UI can be swapped
  later without rewriting business logic.

### Non-goals

- Windows / macOS / KDE / wlroots support (YAGNI). The `uti-core` split keeps
  this possible later.
- Feature additions beyond parity, except where they fall out of the redesign.

---

## 2. Framework decision (summary for ADR-030)

### Decision

- **Toolkit**: gtk4-rs + libadwaita, no Relm4 (decided in Phase 3: logic
  lives in pure `state/` modules, so Relm4's component model added nothing
  but a 0.x dependency and a bundled tokio runtime). Same approach as GNOME's
  own Rust apps.
- **Main popup window**: libadwaita (`AdwApplicationWindow`, `AdwHeaderBar`,
  `AdwToggleGroup` tabs, `AdwStatusPage` empty states). Colors follow the
  system (light / dark, accent); the midnight / dark / light themes are
  dropped. `theme.size` stays as a density / window-size setting.
- **Settings window and dialogs**: libadwaita
  (`AdwPreferencesDialog`, `AdwSwitchRow`, `AdwSpinRow`, `AdwComboRow`,
  `AdwEntryRow`, `AdwAlertDialog`, `AdwAboutDialog`). Settings is an independent
  window, not transient for the popup (Phase 0).
- **D-Bus**: app uses gio via `GApplication` (see 3.1); daemon uses tokio + zbus 6.

### Versions checked (crates.io, 2026-11)

| Crate | Version | Released |
| ----- | ------- | -------- |
| gtk4 | 0.11.5 | 2026-09-20 |
| libadwaita | 0.9.2 | 2026-07-07 |
| zbus (daemon only) | 6.3.0 | - |
| ksni (non-GNOME tray, optional) | 0.3.7 | 2026-10-20 |

- Build with gtk4 feature `gnome_50` and libadwaita `v1_9`. Fedora 44 ships
  GNOME Shell 50.6, libadwaita 1.9.4, gtk4 4.22.5.
- Font rendering is set to `Manual` (desktop hinting): GTK's `Automatic`
  mode made digit and capital tops look cut off at scale 1.

### Alternatives rejected

| Option | Reason |
| ------ | ------ |
| GPUI | Official `gpui` crate stuck at 0.2.2; ecosystem depends on a third-party snapshot (`gpui-pre`). `PlatformWindow`/app-level `hide()` and `activate()` are **not implemented on Linux** (`crates/gpui_linux/src/linux/platform.rs`, logs "hide is not implemented on Linux"). No tray, no clipboard monitoring. Requires Vulkan/GL. |
| Slint 1.18 | Good cross-platform option with built-in tray, but non-native look on GNOME and licensing needs review. Re-evaluate if Windows/macOS become concrete goals. |
| iced / egui | Long release cadence or poor GNOME integration. |
| Keep Tauri | Cross-platform strength is not needed; webview adds attack surface (`execute_command` + `csp: null`) and bundle weight. |

### What no framework solves on GNOME/Wayland

- Clipboard **monitoring**: Mutter implements neither `ext-data-control-v1`
  nor `wlr-data-control`; the extension's `Meta.Selection` watch works
  (Phase 0, C1).
- Window **positioning** and **always-on-top**: not available to clients.
- **Double modifier tap**: GlobalShortcuts portal is combination-based; no
  double-tap concept **[UNVERIFIED]** for modifier-only triggers.

These stay in the daemon and the GNOME extension (section 3).

---

## 3. Target architecture

```text
uti-daemon (evdev)  --signal Triggered-->  uti app  <--method calls-->  GNOME extension
  double-Ctrl only                          owns state                  clipboard watch
                                            (GTK4/adw)                   position / above
                                                                        panel menu
                                                                        paste (virtual kbd)
```

**Principle**: the app is the single owner of state. Other components are
services the app calls via D-Bus methods. No more peer-to-peer signal races.

### 3.1 D-Bus interfaces (new)

| Owner | Bus name | Interface | Members |
| ----- | -------- | --------- | ------- |
| daemon | `io.github.noppomario.uti.Daemon` (today the daemon owns `io.github.noppomario.uti`; must move off it before the GTK app ships) | `io.github.noppomario.uti.Daemon1` | signal `Triggered()` |
| app | `io.github.noppomario.uti` | GActions via `org.freedesktop.Application` | `toggle`, `show`, `hide`, `settings`, `check-updates`, `quit` (Phase 3 adds stateful `pinned`) |
| extension | `io.github.noppomario.uti.Shell` | `io.github.noppomario.uti.Shell1` | `PositionAtPointer(activate: b)`, `SetAbove(b)`, `Paste(plain: b, submit: b)`, signal `ClipboardChanged(s text)` |

Notes:

- `GApplication` owns `io.github.noppomario.uti` (single instance). The app must
  use its connection (`Application::dbus_connection()`) for all D-Bus calls; a
  separate zbus connection can neither take the name nor pass the caller check.
- Methods that trigger input (`Paste`) must check the caller's unique name
  matches the owner of `io.github.noppomario.uti`. This only proves name
  ownership; optionally also check the peer's `/proc/<pid>/exe`.
- Version interfaces (`...1`) so the extension and app can be updated
  independently.
- Keep the old `DoubleTap` interface emitting for one release for migration,
  then remove.

### 3.2 Toggle sequence (new)

1. daemon emits `Triggered`.
2. app receives it; if pinned, ignore. If visible, hide. Otherwise:
3. app calls `Shell1.PositionAtPointer()`, then `present()`. The extension keeps
   the request pending and moves the window on its actor's `first-frame`
   (the frame rect is 0x0 before that), then `hide_from_window_list()`.
4. No `sleep(50ms)` / `timeout_add(100ms)` coordination.
5. app hides itself on `is-active` = false unless pinned.

### 3.3 Responsibilities

| Concern | Today | New |
| ------- | ----- | --- |
| Double Ctrl | daemon (evdev) | daemon (evdev), fixed detector (5.1) |
| Optional normal shortcut | none | extension `Main.wm.addKeybinding` (lets users avoid `input` group) |
| Clipboard history capture | JS polls `readText()` every 1 s | extension listens to `Meta.Selection` `owner-changed`, reads text via `St.Clipboard`, emits `ClipboardChanged` |
| Paste into target app | daemon uinput Ctrl+Shift+V + Enter | extension Clutter virtual keyboard (`notify_keyval`, layout-independent) after focus leaves uti; keep uinput as fallback |
| Positioning / always-on-top / hide from dash | extension reacts to broadcast signals | extension methods called by app |
| Tray | Tauri SNI + extension acts as `StatusNotifierWatcher` | extension shows its own `PanelMenu.Button`, menu actions call `App1` methods. **Drop SNI on GNOME.** |
| Updates | in-app RPM download + `pkexec` | COPR repo with GPG signing; `dnf upgrade`. Remove updater. |
| Autostart | tauri-plugin-autostart | XDG autostart `.desktop` file, or systemd user unit tied to `graphical-session.target` |

### 3.4 Window identification in the extension

The current `_findUtiWindow` matches the first window with WM_CLASS `uti`,
which may be the settings window or a dialog.

- Main window: app ID `io.github.noppomario.uti`, fixed title (e.g. `uti`).
- Settings / dialogs: distinct titles.
- Extension matches `get_gtk_application_id()` **and** `get_title()`. Both are
  `null` at `window-created`; check at `first-frame`.

---

## 4. Repository layout (new)

```text
crates/
  uti-core/        # pure logic, no GTK: config, clipboard history, snippets,
                   # launcher config + recent files, D-Bus names, atomic writes
app/               # uti binary: GTK4 / libadwaita UI, pure state/ modules
daemon/            # uti-daemon: evdev + Daemon1 signal (+ uinput fallback)
gnome-extension/   # uti for GNOME: Shell1 service, panel menu, clipboard watch
packaging/         # RPM specs, COPR config, desktop file, metainfo, icons
```

Existing Rust modules in `app/src-tauri/src/{config,clipboard,snippets,launcher}`
move into `uti-core` with their tests largely intact.

---

## 5. Defects to fix (found in current code)

### 5.1 Daemon

1. **False double-tap**: `daemon/src/main.rs` (~L202-215) compares only
   Ctrl *release* times (non-Ctrl keys are skipped), and the last-release
   timestamp is shared across all devices. `Ctrl+C` then `Ctrl+V` within
   300 ms triggers.
   Replace with a state machine: a "tap" is press -> release of Ctrl with no
   other key pressed in between and press duration below a threshold; two taps
   within the interval trigger. Implement as a pure `DoubleTapDetector` with
   injected timestamps and test it (current tests only assert constants).
2. **No hotplug**: devices are enumerated once (`find_keyboard_devices`,
   ~L119). Keyboards connected later (Bluetooth) are ignored; unplug exits the
   process. Watch udev (or inotify on `/dev/input`) and add/remove device tasks.
3. **Unauthenticated input injection**: `TypeText` match rule (~L66) accepts
   any sender on the session bus. Any process can make the daemon send
   paste + Enter. Move to a method with caller check, or remove when the
   extension handles paste.
4. **Layout assumption**: `daemon/src/uinput.rs` emits keycodes for
   Ctrl+Shift+V + Enter (~L86, L102). Wrong on non-QWERTY layouts; Enter is
   always sent. Make paste combo and "submit" configurable.
5. Bump `zbus` 4 -> 6, `evdev` 0.12 -> 0.13.

### 5.2 App (carry over into rewrite)

1. **Zombie processes**: `execute_command` uses `Command::spawn()` without
   waiting (`app/src-tauri/src/launcher/commands.rs` ~L22). Use
   `gio::DesktopAppInfo::launch` / `gio::AppInfo::launch_default_for_uri`
   (also provides xdg-activation tokens on Wayland) or reap children.
2. **Desktop entry search**: `launcher/desktop.rs` rescans all `.desktop`
   files on every keystroke, ignores `XDG_DATA_DIRS` (Flatpak exports) and
   localized `Name[xx]`. Use `gio::AppInfo::all()` once, cache, refresh on
   `gio::AppInfoMonitor` change.
3. **Arbitrary command execution from the webview** with `csp: null`
   (`tauri.conf.json`). Disappears with the native rewrite; keep launcher
   commands sourced only from `launcher.json`.
4. **Non-atomic writes**: `clipboard/store.rs`, `snippets/store.rs` and
   `config/` use `fs::write` directly.
   Write to temp file + `rename`, mode `0600`.
5. **Data location**: clipboard history is data, not config. Move to
   `$XDG_DATA_HOME/uti/` (or `$XDG_STATE_HOME`) with migration from
   `~/.config/uti/clipboard.json`.
6. **Sensitive content**: skip clipboard entries offered with
   `x-kde-passwordManagerHint` (password managers); visible via
   `Meta.Selection.get_mimetypes()` (Phase 0, C2).
7. **Index-based removal** (`remove_clipboard_item`,
   `clipboard/commands.rs` ~L37): use stable IDs.
8. Remove dead stub `paste_item`.
9. Remove the 50 ms `sleep` in `toggle_window` (`window/commands.rs` ~L35)
   via the new sequence.

### 5.3 Updater / distribution

1. RPMs downloaded to predictable `/tmp/<name>` paths (`updater/core.rs`
   ~L259, L303) and installed with `pkexec` without signature or checksum
   verification. Replace with a signed COPR repository and drop the updater.
2. GNOME extension installed by unzipping into the user dir; prefer shipping
   it in the RPM under `/usr/share/gnome-shell/extensions/` and/or
   extensions.gnome.org.

### 5.4 GNOME extension

1. **Takes over `org.kde.StatusNotifierWatcher`** (`extension.js` ~L117)
   but only displays uti's item: conflicts with the AppIndicator extension
   and hides other apps' tray icons. Remove the watcher entirely (3.3).
2. **Wrong window may be moved** (`_findUtiWindow`, ~L653). See 3.4.
3. **Moves/activates the window on every `Triggered`** (`_onTriggered`,
   ~L634) even when the app is hiding it or pinned. Fixed by method-based flow.
4. **Synchronous D-Bus calls block the shell**: `new SNIProxy(...)` (~L273,
   L521) and `Gio.bus_get_sync` (~L568). Use async variants
   (extensions.gnome.org review guideline).
5. `SetAlwaysOnTop` accepts any sender (~L585). Fixed by `Shell1.SetAbove`
   with caller check.
6. `metadata.json` `shell-version` is `45`-`49`; the installed extension is
   **OUT OF DATE** on GNOME 50. Set to `["50"]` (add 51 after testing).

---

## 6. Migration phases

Each phase is shippable on its own. Follow TDD for every phase.

### Phase 0: PoC (throwaway, 1-2 days)

Done 2026-11-14: no blocker. Results and design changes in
[phase0-findings.md](phase0-findings.md) (already applied to this plan).
Open: clipboard watch and paste in a GTK editor / terminal, real-session run
of the extension.

### Phase 1: Component redesign (still on Tauri)

Valuable even if the rewrite is delayed.

- [x] New D-Bus interfaces (3.1); legacy `DoubleTap.Triggered` kept one release.
- [x] Daemon: `DoubleTapDetector` state machine + tests; hotplug; zbus 6 (#183).
- [x] Extension: `shell-version` `["50"]` (5.4.6), drop SNI watcher, own
      panel menu, async D-Bus, method-based positioning, clipboard watch (#184).
- [x] App (Tauri): wired to the new contract, SNI tray removed (#185).
- [x] App: consume `ClipboardChanged` instead of polling (done in Phase 3).

### Phase 2: Extract `uti-core`

- [x] Move config / clipboard / snippets / launcher logic and tests (#195).
      Shared D-Bus names live in `uti_core::dbus`; no separate `uti-dbus` crate.
- [x] Atomic writes, `0600`, XDG data dir migration, stable IDs.

### Phase 3: GTK4 / libadwaita app

- [x] Main window: tab bar, search, clipboard / snippets / launcher tabs,
      keyboard navigation (`state::nav`), jump list, pin button, key-hint
      footer. The Prompt tab is retired (kept in code, not wired).
- [x] Look: libadwaita, system colors and accent; density from `theme.size`.
- [x] Settings window with libadwaita rows; i18n from JSON (en / ja).
- [x] Launcher via `gio::AppInfo` (5.2.1, 5.2.2).
- [x] Remove React frontend, Vite, Biome, Vitest; bun kept as task runner
      (`package.json` scripts updated).

### Phase 4: Distribution

- [x] COPR repo `noppomario/uti` with RPMs (app, daemon, extension) from one
      spec (`packaging/uti.spec`); `release.yml` triggers the build. deb on request.
- [x] Remove `updater.rs`, `uti update` CLI and tray "Check for Updates".
      "Check for Updates" now opens Releases.
- [x] Desktop file, AppStream metainfo, icons.
- [x] Update `install.sh` to add the COPR repo.

### Phase 5: Docs and memory

- [ ] ADR-030 (framework), ADR-031 (D-Bus redesign / extension as shell
      service), ADR-032 (COPR distribution). Archive superseded ADRs.
- [ ] Update `CLAUDE.md`, `.claude/rules/context.md`, `docs/ARCHITECTURE.md`,
      `docs/DEVELOPMENT.md` (testing strategy changes: Rust unit tests for
      `update()` logic replace React Testing Library).

---

## 7. Testing strategy

- `uti-core`: unit tests, target 90%+ for `DoubleTapDetector` and stores.
- GTK app: test `state/` (window, list, nav) as pure state transitions; keep
  widgets thin. `UTI_DEBUG_SNAPSHOT` / `UTI_DEBUG_KEYS` render the window to
  PNG and inject keys over D-Bus for manual checks.
- D-Bus: integration tests on a private session bus (`dbus-run-session`).
- Extension: manual test checklist per GNOME version (nested shell:
  `dbus-run-session gnome-shell --devkit --wayland --no-x11 --wayland-display NAME`,
  needs `mutter-devkit`; no ibus inside, dconf shared with the real session).

---

## 8. Risks

| Risk | Mitigation |
| ---- | ---------- |
| Extension API changes every GNOME release | Keep extension small; versioned D-Bus interface; per-release test checklist |
| Clutter virtual keyboard paste not viable | Keep uinput fallback in daemon (with caller check) |
| Loss of frontend test coverage during rewrite | Move logic to Rust first (Phase 2) so tests exist before UI rewrite |
| Users relying on `input` group | Offer normal shortcut via extension as alternative |

---

## 9. Draft ADR-030

```markdown
## ADR-030: Native GTK4 UI with libadwaita

**Date**: 2026-11-14
**Status**: Proposed
**Decision Makers**: Project team

### Context

uti targets GNOME/Wayland only. Tauri + React adds a webview, a JS toolchain
and IPC surface (arbitrary command execution, csp: null) while the hard
platform problems (positioning, always-on-top, clipboard monitoring, double
modifier tap) are solved outside the UI framework anyway.

### Decision

Rewrite the app with gtk4-rs + libadwaita (no Relm4) for both the popup and
the settings window. Follow the system style instead of custom themes.

### Rationale

- Native hide/show, IME, HiDPI, accent color and dark mode on GNOME.
- System libraries already on Fedora; smaller package.
- Pure `state/` modules keep state transitions testable in Rust.

### Alternatives Considered

1. GPUI - no window hide/activate on Linux, unofficial crate snapshots.
2. Slint - cross-platform with built-in tray; non-native look; licensing.
3. Keep Tauri - cross-platform not needed; webview attack surface.

### Consequences

**Positive**: single language (Rust + GJS extension), native look, simpler
build, no webview.

**Negative**: full UI rewrite; React tests discarded; custom color themes
dropped; GNOME-only.

**Reconsider when**: Windows/macOS/KDE support becomes a concrete goal
(re-evaluate Slint).
```
