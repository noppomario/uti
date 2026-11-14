# Phase 0 findings (PoC on GNOME 50)

**Date**: 2026-11-14
**Environment**: Fedora 44, GNOME Shell 50.6 (Mutter 18), gtk4 4.22.5, libadwaita 1.9.4, `jp` layout + ibus/mozc
**PoC code**: branch `poc/phase0` (`poc/app`, `poc/extension`, not for merge)

## Results

| # | Item | Result | Notes |
| --- | --- | --- | --- |
| A1 | `hide()` / `present()` toggle | Pass | Real session, triggered by daemon while another app had focus: `present()` 3-4 ms after the trigger, `is-active=true` ~70 ms later. No extension `activate()` needed. |
| A2 | Undecorated, rounded corners, shadow | Pass | Transparent `window` background + CSS `border-radius` / `box-shadow` on an inner box. |
| A3 | Startup latency | Pass | ~130 ms from exec to first map (debug build). |
| A4 | Hidden from dash / Alt+Tab | Pass | Extension calls `Meta.Window.hide_from_window_list()` (Mutter 18); works on a normal Wayland client, `skip_taskbar` becomes true. |
| B1 | Position at pointer | Pass, order fixed | "present then position" fails: frame rect is 0x0 at that point. Working order: app calls `PositionAtPointer` first, extension keeps it pending, app presents, extension moves on the actor's `first-frame` (31-74 ms later). No visible flicker. |
| B2 | Identify main window | Pass | `get_gtk_application_id()` + `get_title()`; both are `null` at `window-created`, valid at `first-frame`. |
| B3 | Always-on-top | Pass | `make_above()` / `unmake_above()`. |
| B4 | Caller check | Pass | Compare `invocation.get_sender()` with `GetNameOwner(app id)`; calls from `gdbus` are rejected with `AccessDenied`. |
| C1 | Clipboard watch | Pass (Firefox only) | `Meta.Selection` `owner-changed` + `St.Clipboard.get_text` while uti is unfocused. GTK editor and terminal not tested. One copy can fire 2 events, plus events with an empty MIME list. |
| C2 | Password hint | Pass | `x-kde-passwordManagerHint` is visible in `get_mimetypes()`; entry skipped. Tested with a self-made provider, not a real password manager. |
| D1 | Paste via Clutter virtual keyboard | Pass (Firefox) | `notify_keyval` (keyval-based, so layout-independent) on `jp` layout. Extension waits for focus to leave uti before sending. |
| E1 | IME | Pass | ibus + mozc in `GtkSearchEntry` and `GtkTextView`. |
| F1 | Nested shell | Pass | `dbus-run-session gnome-shell --devkit --wayland --no-x11 --wayland-display NAME` (needs `mutter-devkit`). ibus is not available inside; dconf is shared with the real session. |

## Design changes

1. **App D-Bus goes through GApplication, not zbus.** GApplication already owns the app id on the bus, so a separate zbus connection cannot take the name. Expose app commands as GActions (`org.freedesktop.Application.ActivateAction`) and call `Shell1` over `Application::dbus_connection()`; otherwise the extension's caller check sees the wrong sender. zbus stays in the daemon only (zbus 6.3).
2. **Toggle order is position-then-present** (B1).
3. **Hide from dash / Alt+Tab** with `hide_from_window_list()` (A4).
4. **Auto-hide on deactivate unless pinned** stays an app concern (`is-active` notify).
5. **Settings is an independent window**, not transient for the popup (the popup auto-hides when settings takes focus). Host `AdwPreferencesDialog` in its own window.
6. **Deduplicate clipboard events** in the app (same text twice in a row, empty MIME list).

## Residual risks

- Caller check only proves "owner of the app id". If uti is not running, another process can own the name first. Hardening option: check `GetConnectionUnixProcessID` -> `/proc/<pid>/exe`.
- C1 / D1 not tested in a GTK editor or a terminal.
- Real-session test of the extension (not nested) not done.

## Also observed

- Defect 5.1.2 (no hotplug) reproduced: the daemon kept watching only the internal keyboard for 5 days after an external keyboard was connected.

## Verdict

No blocker. Proceed to Phase 1 with the design changes above.
