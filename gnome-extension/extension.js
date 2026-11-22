/**
 * uti for GNOME
 *
 * Shell-side services for the uti app (GNOME 50):
 * 1. `io.github.noppomario.uti.Shell1` D-Bus service: window positioning,
 *    always-on-top, paste via virtual keyboard, clipboard change signal,
 *    whether the pointer is over the desktop.
 *    Only the current owner of the app bus name may call it.
 * 2. Panel menu button that invokes the app's GActions through
 *    `org.freedesktop.Application.ActivateAction`.
 */

import Clutter from 'gi://Clutter';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import GObject from 'gi://GObject';
import Meta from 'gi://Meta';
import St from 'gi://St';
import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';

const APP_ID = 'io.github.noppomario.uti';
const APP_OBJECT_PATH = '/io/github/noppomario/uti';
const APP_WM_CLASS = 'uti';
const MAIN_WINDOW_TITLE = 'uti';

const SHELL_BUS_NAME = 'io.github.noppomario.uti.Shell';
const SHELL_OBJECT_PATH = '/io/github/noppomario/uti/Shell';

const GITHUB_URL = 'https://github.com/noppomario/uti';
const PASSWORD_HINT_MIME = 'x-kde-passwordManagerHint';

/** How long a PositionAtPointer request waits for the main window to map. */
const PENDING_POSITION_TIMEOUT_MS = 2000;
/** How long Paste waits for focus to leave uti before giving up. */
const PASTE_FOCUS_TIMEOUT_MS = 1000;
/** Delay between the paste shortcut and Return when submitting. */
const SUBMIT_DELAY_MS = 50;

const SHELL_IFACE_XML = `
<node>
  <interface name="io.github.noppomario.uti.Shell1">
    <method name="PositionAtPointer">
      <arg type="b" name="activate" direction="in"/>
    </method>
    <method name="SetAbove">
      <arg type="b" name="above" direction="in"/>
    </method>
    <method name="Paste">
      <arg type="b" name="plain" direction="in"/>
      <arg type="b" name="submit" direction="in"/>
    </method>
    <method name="PointerOnDesktop">
      <arg type="b" name="on_desktop" direction="out"/>
    </method>
    <signal name="ClipboardChanged">
      <arg type="s" name="text"/>
    </signal>
    <signal name="ShellPressed"/>
  </interface>
</node>`;

/**
 * Whether `win` belongs to the uti app (any of its windows).
 *
 * @param {Meta.Window} win
 * @returns {boolean}
 */
function isUtiWindow(win) {
    return (
        win.get_gtk_application_id() === APP_ID ||
        win.get_wm_class()?.toLowerCase() === APP_WM_CLASS
    );
}

/**
 * Whether `win` is the uti main (popup) window.
 *
 * The settings window is titled `uti - Settings` and must not match.
 * Title and app id are only valid from the actor's `first-frame` on.
 *
 * @param {Meta.Window} win
 * @returns {boolean}
 */
function isMainWindow(win) {
    return isUtiWindow(win) && win.get_title() === MAIN_WINDOW_TITLE;
}

/**
 * @returns {Meta.Window | null} The uti main window, if mapped
 */
function findMainWindow() {
    for (const actor of global.get_window_actors()) {
        const win = actor.get_meta_window();
        if (win && isMainWindow(win)) return win;
    }
    return null;
}

/**
 * Whether `actor` is part of uti for GNOME's own panel button or its menu.
 *
 * @param {Clutter.Actor | null} actor
 * @returns {boolean}
 */
function isOwnUi(actor) {
    for (let a = actor; a; a = a.get_parent()) if (a._utiOwnUi) return true;
    return false;
}

/**
 * Whether the pointer is over the desktop: the background, or a window of
 * type DESKTOP. False in the overview, where the background is not clickable.
 *
 * @returns {boolean}
 */
function isPointerOnDesktop() {
    if (Main.overview.visible) return false;
    const [x, y] = global.get_pointer();
    const actor = global.stage.get_actor_at_pos(Clutter.PickMode.REACTIVE, x, y);
    for (let a = actor; a; a = a.get_parent()) {
        if (a instanceof Meta.BackgroundActor) return true;
        if (a instanceof Meta.WindowActor)
            return a.get_meta_window()?.get_window_type() === Meta.WindowType.DESKTOP;
    }
    return false;
}

/**
 * Move `window` next to (x, y), flipping to the other side of the pointer
 * when it does not fit in the work area.
 *
 * @param {Meta.Window} window
 * @param {number} x - Pointer x
 * @param {number} y - Pointer y
 */
function moveWindowToCursor(window, x, y) {
    const rect = window.get_frame_rect();
    const workArea = Main.layoutManager.getWorkAreaForMonitor(window.get_monitor());

    const spaceRight = workArea.x + workArea.width - x;
    const spaceLeft = x - workArea.x;
    const spaceBottom = workArea.y + workArea.height - y;
    const spaceTop = y - workArea.y;

    let newX;
    if (rect.width <= spaceRight) {
        newX = x;
    } else if (rect.width <= spaceLeft) {
        newX = x - rect.width;
    } else {
        newX = Math.max(workArea.x, workArea.x + workArea.width - rect.width);
    }

    let newY;
    if (rect.height <= spaceBottom) {
        newY = y;
    } else if (rect.height <= spaceTop) {
        newY = y - rect.height;
    } else {
        newY = Math.max(workArea.y, workArea.y + workArea.height - rect.height);
    }

    window.move_frame(true, newX, newY);
}

/**
 * Invoke one of the app's GActions asynchronously.
 *
 * @param {string} action - Action name (toggle, show, hide, settings, check-updates, quit)
 */
function activateAppAction(action) {
    Gio.DBus.session.call(
        APP_ID,
        APP_OBJECT_PATH,
        'org.freedesktop.Application',
        'ActivateAction',
        new GLib.Variant('(sava{sv})', [action, [], {}]),
        null,
        Gio.DBusCallFlags.NONE,
        -1,
        null,
        (conn, res) => {
            try {
                conn.call_finish(res);
            } catch (e) {
                console.warn(`[uti] ActivateAction(${action}) failed: ${e.message}`);
            }
        }
    );
}

/**
 * Implementation of `io.github.noppomario.uti.Shell1`.
 */
class ShellService {
    constructor() {
        this._impl = null;
        this._ownerId = 0;
        this._keyboard = null;
        this._pending = null;
        this._pendingTimeoutId = 0;
        this._watchedActors = new Set();
        this._focusWait = null;
        this._submitTimeoutId = 0;
        this._lastClipboardText = null;
        this._pressFilterId = 0;
    }

    enable() {
        this._impl = Gio.DBusExportedObject.wrapJSObject(SHELL_IFACE_XML, this);
        this._impl.export(Gio.DBus.session, SHELL_OBJECT_PATH);
        this._ownerId = Gio.bus_own_name(
            Gio.BusType.SESSION,
            SHELL_BUS_NAME,
            Gio.BusNameOwnerFlags.NONE,
            null,
            null,
            () => console.warn(`[uti] Lost D-Bus name ${SHELL_BUS_NAME}`)
        );

        global.display.connectObject(
            'window-created',
            (_display, win) => this._watchFirstFrame(win),
            this
        );
        global.display
            .get_selection()
            .connectObject(
                'owner-changed',
                (selection, type) => this._onSelectionOwnerChanged(selection, type),
                this
            );

        // Presses handled by the Shell itself (desktop background, panel).
        // Presses on app windows never reach this filter.
        this._pressFilterId = Clutter.Event.add_filter(global.stage, (event, actor) => {
            if (
                event.type() === Clutter.EventType.BUTTON_PRESS &&
                !isOwnUi(actor) &&
                findMainWindow()
            )
                this._impl?.emit_signal('ShellPressed', null);
            return Clutter.EVENT_PROPAGATE;
        });

        this._keyboard = global.stage.context
            .get_backend()
            .get_default_seat()
            .create_virtual_device(Clutter.InputDeviceType.KEYBOARD_DEVICE);
    }

    disable() {
        if (this._pressFilterId) Clutter.Event.remove_filter(this._pressFilterId);
        this._pressFilterId = 0;
        global.display.disconnectObject(this);
        global.display.get_selection().disconnectObject(this);
        for (const actor of this._watchedActors) actor.disconnectObject(this);
        this._watchedActors.clear();

        this._clearPending();
        this._cancelFocusWait();
        if (this._submitTimeoutId) {
            GLib.source_remove(this._submitTimeoutId);
            this._submitTimeoutId = 0;
        }

        if (this._ownerId) {
            Gio.bus_unown_name(this._ownerId);
            this._ownerId = 0;
        }
        this._impl?.unexport();
        this._impl = null;
        this._keyboard = null;
        this._lastClipboardText = null;
    }

    // --- D-Bus methods ---

    PositionAtPointerAsync([activate], invocation) {
        this._checkCaller(invocation, () => {
            const win = findMainWindow();
            const rect = win?.get_frame_rect();
            if (win && rect.width > 0 && rect.height > 0) {
                this._placeMainWindow(win, activate);
            } else {
                // The frame rect is 0x0 until the first frame; move then.
                this._setPending(activate);
                if (win) this._watchFirstFrame(win);
            }
            invocation.return_value(null);
        });
    }

    SetAboveAsync([above], invocation) {
        this._checkCaller(invocation, () => {
            const win = findMainWindow();
            if (win) {
                if (above) win.make_above();
                else win.unmake_above();
            }
            invocation.return_value(null);
        });
    }

    PasteAsync([plain, submit], invocation) {
        this._checkCaller(invocation, () => {
            invocation.return_value(null);
            this._whenFocusLeavesUti(() => this._sendPaste(plain, submit));
        });
    }

    PointerOnDesktopAsync(_params, invocation) {
        this._checkCaller(invocation, () => {
            invocation.return_value(new GLib.Variant('(b)', [isPointerOnDesktop()]));
        });
    }

    // --- Caller check ---

    /**
     * Run `onAllowed` only if the caller owns the app bus name; otherwise
     * reply with AccessDenied.
     */
    _checkCaller(invocation, onAllowed) {
        const sender = invocation.get_sender();
        Gio.DBus.session.call(
            'org.freedesktop.DBus',
            '/org/freedesktop/DBus',
            'org.freedesktop.DBus',
            'GetNameOwner',
            new GLib.Variant('(s)', [APP_ID]),
            new GLib.VariantType('(s)'),
            Gio.DBusCallFlags.NONE,
            -1,
            null,
            (conn, res) => {
                let owner = null;
                try {
                    [owner] = conn.call_finish(res).deepUnpack();
                } catch {
                    // App not running: nobody is allowed.
                }
                if (!this._impl) return;
                if (owner === null || owner !== sender) {
                    invocation.return_dbus_error(
                        'org.freedesktop.DBus.Error.AccessDenied',
                        `Caller ${sender} does not own ${APP_ID}`
                    );
                    return;
                }
                onAllowed();
            }
        );
    }

    // --- Positioning ---

    _setPending(activate) {
        this._clearPending();
        this._pending = { activate };
        this._pendingTimeoutId = GLib.timeout_add(
            GLib.PRIORITY_DEFAULT,
            PENDING_POSITION_TIMEOUT_MS,
            () => {
                this._pendingTimeoutId = 0;
                this._pending = null;
                return GLib.SOURCE_REMOVE;
            }
        );
    }

    _clearPending() {
        if (this._pendingTimeoutId) {
            GLib.source_remove(this._pendingTimeoutId);
            this._pendingTimeoutId = 0;
        }
        this._pending = null;
    }

    /**
     * Decide on the actor's first frame, when title and app id are set and
     * the frame rect has its real size.
     */
    _watchFirstFrame(win) {
        const actor = win.get_compositor_private();
        if (!actor || this._watchedActors.has(actor)) return;

        const forget = () => {
            actor.disconnectObject(this);
            this._watchedActors.delete(actor);
        };
        this._watchedActors.add(actor);
        actor.connectObject(
            'first-frame',
            () => {
                forget();
                this._onFirstFrame(win);
            },
            'destroy',
            forget,
            this
        );
    }

    _onFirstFrame(win) {
        if (!isMainWindow(win)) return;
        if (this._pending) {
            const { activate } = this._pending;
            this._clearPending();
            this._placeMainWindow(win, activate);
        } else {
            win.hide_from_window_list();
        }
    }

    _placeMainWindow(win, activate) {
        const [x, y] = global.get_pointer();
        moveWindowToCursor(win, x, y);
        win.hide_from_window_list();
        if (activate) win.activate(global.get_current_time());
    }

    // --- Paste ---

    _whenFocusLeavesUti(callback) {
        this._cancelFocusWait();
        const focusIsUti = () => {
            const focus = global.display.focus_window;
            return focus !== null && isUtiWindow(focus);
        };
        if (!focusIsUti()) {
            callback();
            return;
        }

        const timeoutId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, PASTE_FOCUS_TIMEOUT_MS, () => {
            this._focusWait.timeoutId = 0;
            this._cancelFocusWait();
            console.warn('[uti] Focus did not leave uti; paste aborted');
            return GLib.SOURCE_REMOVE;
        });
        const signalId = global.display.connect('notify::focus-window', () => {
            if (focusIsUti()) return;
            this._cancelFocusWait();
            callback();
        });
        this._focusWait = { timeoutId, signalId };
    }

    _cancelFocusWait() {
        if (!this._focusWait) return;
        const { timeoutId, signalId } = this._focusWait;
        if (timeoutId) GLib.source_remove(timeoutId);
        global.display.disconnect(signalId);
        this._focusWait = null;
    }

    _sendPaste(plain, submit) {
        if (!this._keyboard) return;
        const keys = plain
            ? [Clutter.KEY_Control_L, Clutter.KEY_Shift_L, Clutter.KEY_v]
            : [Clutter.KEY_Control_L, Clutter.KEY_v];
        this._tapKeys(keys);
        if (!submit) return;

        if (this._submitTimeoutId) GLib.source_remove(this._submitTimeoutId);
        this._submitTimeoutId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, SUBMIT_DELAY_MS, () => {
            this._submitTimeoutId = 0;
            this._tapKeys([Clutter.KEY_Return]);
            return GLib.SOURCE_REMOVE;
        });
    }

    /** Press `keyvals` in order, then release them in reverse order. */
    _tapKeys(keyvals) {
        if (!this._keyboard) return;
        let time = GLib.get_monotonic_time();
        for (const keyval of keyvals)
            this._keyboard.notify_keyval(time++, keyval, Clutter.KeyState.PRESSED);
        for (const keyval of [...keyvals].reverse())
            this._keyboard.notify_keyval(time++, keyval, Clutter.KeyState.RELEASED);
    }

    // --- Clipboard ---

    _onSelectionOwnerChanged(selection, type) {
        if (type !== Meta.SelectionType.SELECTION_CLIPBOARD) return;
        const mimes = selection.get_mimetypes(Meta.SelectionType.SELECTION_CLIPBOARD);
        if (mimes.length === 0 || mimes.includes(PASSWORD_HINT_MIME)) return;

        St.Clipboard.get_default().get_text(St.ClipboardType.CLIPBOARD, (_clipboard, text) => {
            if (!this._impl || !text || text === this._lastClipboardText) return;
            this._lastClipboardText = text;
            this._impl.emit_signal('ClipboardChanged', new GLib.Variant('(s)', [text]));
        });
    }
}

/**
 * Panel button: opens the menu (Show/Hide, Settings, ...).
 */
const UtiIndicator = GObject.registerClass(
    class UtiIndicator extends PanelMenu.Button {
        _init() {
            super._init(0.0, 'uti', false);
            // Clicks on our own UI must not count as clicks outside uti.
            this._utiOwnUi = true;
            this.menu.actor._utiOwnUi = true;

            this.add_child(
                new St.Icon({
                    icon_name: 'edit-paste-symbolic',
                    style_class: 'system-status-icon',
                })
            );

            this._addAction('Show/Hide', () => activateAppAction('toggle'));
            this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
            this._addAction('Settings', () => activateAppAction('settings'));
            this._addAction('Check for Updates', () => activateAppAction('check-updates'));
            this._addAction('GitHub', () => this._openGitHub());
            this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
            this._addAction('Quit', () => activateAppAction('quit'));
        }

        _addAction(label, callback) {
            const item = new PopupMenu.PopupMenuItem(label);
            item.connect('activate', callback);
            this.menu.addMenuItem(item);
        }

        _openGitHub() {
            try {
                Gio.AppInfo.launch_default_for_uri(
                    GITHUB_URL,
                    global.create_app_launch_context(0, -1)
                );
            } catch (e) {
                console.warn(`[uti] Failed to open ${GITHUB_URL}: ${e.message}`);
            }
        }
    }
);

export default class UtiExtension extends Extension {
    enable() {
        this._service = new ShellService();
        this._service.enable();

        this._settings = this.getSettings();
        this._settings.connectObject(
            'changed::enable-tray-icon',
            () => this._syncIndicator(),
            this
        );
        this._syncIndicator();
    }

    disable() {
        this._settings.disconnectObject(this);
        this._settings = null;
        this._indicator?.destroy();
        this._indicator = null;
        this._service.disable();
        this._service = null;
    }

    _syncIndicator() {
        const enabled = this._settings.get_boolean('enable-tray-icon');
        if (enabled && !this._indicator) {
            this._indicator = new UtiIndicator();
            Main.panel.addToStatusArea(this.uuid, this._indicator);
        } else if (!enabled && this._indicator) {
            this._indicator.destroy();
            this._indicator = null;
        }
    }
}
